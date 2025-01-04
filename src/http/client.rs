#![allow(clippy::missing_errors_doc)]

use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::header::{HeaderMap as Headers, HeaderValue};
#[cfg(feature = "utils")]
use reqwest::Url;
use reqwest::{Client, ClientBuilder, Response as ReqwestResponse, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use tracing::{debug, instrument, warn};

use super::multipart::{Multipart, MultipartUpload};
use super::ratelimiting::Ratelimiter;
use super::request::Request;
use super::routing::Route;
use super::typing::Typing;
use super::{
    ErrorResponse,
    GuildPagination,
    HttpError,
    LightMethod,
    MessagePagination,
    UserPagination,
};
use crate::builder::{CreateAllowedMentions, CreateAttachment};
use crate::constants;
use crate::internal::prelude::*;
use crate::json::*;
use crate::model::prelude::*;

/// A builder for the underlying [`Http`] client that performs requests to Discord's HTTP API
///
/// If you do not need to use a proxy or do not need to disable the rate limiter, you can use
/// [`Http::new`] instead.
///
/// ## Example
///
/// Create an instance of [`Http`] with a proxy and rate limiter disabled
///
/// ```rust
/// # use serenity::http::HttpBuilder;
/// # fn run() {
/// let http =
///     HttpBuilder::new("token").proxy("http://127.0.0.1:3000").ratelimiter_disabled(true).build();
/// # }
/// ```
#[must_use]
pub struct HttpBuilder {
    client: Option<Client>,
    ratelimiter: Option<Ratelimiter>,
    ratelimiter_disabled: bool,
    token: SecretString,
    proxy: Option<String>,
    application_id: Option<ApplicationId>,
    default_allowed_mentions: Option<CreateAllowedMentions>,
}

impl HttpBuilder {
    /// Construct a new builder to call methods on for the HTTP construction. The `token` will
    /// automatically be prefixed "Bot " if not already.
    pub fn new(token: impl AsRef<str>) -> Self {
        Self {
            client: None,
            ratelimiter: None,
            ratelimiter_disabled: false,
            token: SecretString::new(parse_token(token)),
            proxy: None,
            application_id: None,
            default_allowed_mentions: None,
        }
    }

    /// Sets the application_id to use interactions.
    pub fn application_id(mut self, application_id: ApplicationId) -> Self {
        self.application_id = Some(application_id);
        self
    }

    /// Sets a token for the bot. If the token is not prefixed "Bot ", this method will
    /// automatically do so.
    pub fn token(mut self, token: impl AsRef<str>) -> Self {
        self.token = SecretString::new(parse_token(token));
        self
    }

    /// Sets the [`reqwest::Client`]. If one isn't provided, a default one will be used.
    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Sets the ratelimiter to be used. If one isn't provided, a default one will be used.
    pub fn ratelimiter(mut self, ratelimiter: Ratelimiter) -> Self {
        self.ratelimiter = Some(ratelimiter);
        self
    }

    /// Sets whether or not the ratelimiter is disabled. By default if this this not used, it is
    /// enabled. In most cases, this should be used in conjunction with [`Self::proxy`].
    ///
    /// **Note**: You should **not** disable the ratelimiter unless you have another form of rate
    /// limiting. Disabling the ratelimiter has the main purpose of delegating rate limiting to an
    /// API proxy via [`Self::proxy`] instead of the current process.
    pub fn ratelimiter_disabled(mut self, ratelimiter_disabled: bool) -> Self {
        self.ratelimiter_disabled = ratelimiter_disabled;
        self
    }

    /// Sets the proxy that Discord HTTP API requests will be passed to. This is mainly intended
    /// for something like [`twilight-http-proxy`] where multiple processes can make API requests
    /// while sharing a single ratelimiter.
    ///
    /// The proxy should be in the form of the protocol and hostname, e.g. `http://127.0.0.1:3000`
    /// or `http://myproxy.example`
    ///
    /// This will simply send HTTP API requests to the proxy instead of Discord API to allow the
    /// proxy to intercept, rate limit, and forward requests. This is different than a native
    /// proxy's behavior where it will tunnel requests that use TLS via [`HTTP CONNECT`] method
    /// (e.g. using [`reqwest::Proxy`]).
    ///
    /// [`twilight-http-proxy`]: https://github.com/twilight-rs/http-proxy
    /// [`HTTP CONNECT`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/CONNECT
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Sets the [`CreateAllowedMentions`] used by default for each request that would use it.
    ///
    /// This only takes effect if you are calling through the model or builder methods, not directly
    /// calling [`Http`] methods, as [`Http`] is simply used as a convenient storage for these.
    pub fn default_allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
        self.default_allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Use the given configuration to build the `Http` client.
    #[must_use]
    pub fn build(self) -> Http {
        let application_id = AtomicU64::new(self.application_id.map_or(0, ApplicationId::get));

        let client = self.client.unwrap_or_else(|| {
            let builder = configure_client_backend(Client::builder());
            builder.build().expect("Cannot build reqwest::Client")
        });

        let ratelimiter = (!self.ratelimiter_disabled).then(|| {
            self.ratelimiter
                .unwrap_or_else(|| Ratelimiter::new(client.clone(), self.token.expose_secret()))
        });

        Http {
            client,
            ratelimiter,
            proxy: self.proxy,
            token: self.token,
            application_id,
            default_allowed_mentions: self.default_allowed_mentions,
        }
    }
}

fn parse_token(token: impl AsRef<str>) -> String {
    let token = token.as_ref().trim();

    if token.starts_with("Bot ") || token.starts_with("Bearer ") {
        token.to_string()
    } else {
        format!("Bot {token}")
    }
}

fn reason_into_header(reason: &str) -> Headers {
    let mut headers = Headers::new();

    // "The X-Audit-Log-Reason header supports 1-512 URL-encoded UTF-8 characters."
    // https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object
    let header_value = match Cow::from(utf8_percent_encode(reason, NON_ALPHANUMERIC)) {
        Cow::Borrowed(value) => HeaderValue::from_str(value),
        Cow::Owned(value) => HeaderValue::try_from(value),
    }
    .expect("Invalid header value even after percent encode");

    headers.insert("X-Audit-Log-Reason", header_value);
    headers
}

/// **Note**: For all member functions that return a [`Result`], the Error kind will be either
/// [`Error::Http`] or [`Error::Json`].
#[derive(Debug)]
pub struct Http {
    pub(crate) client: Client,
    pub ratelimiter: Option<Ratelimiter>,
    pub proxy: Option<String>,
    token: SecretString,
    application_id: AtomicU64,
    pub default_allowed_mentions: Option<CreateAllowedMentions>,
}

impl Http {
    #[must_use]
    pub fn new(token: &str) -> Self {
        HttpBuilder::new(token).build()
    }

    pub fn application_id(&self) -> Option<ApplicationId> {
        let application_id = self.application_id.load(Ordering::Relaxed);
        NonZeroU64::new(application_id).map(ApplicationId::from)
    }

    fn try_application_id(&self) -> Result<ApplicationId> {
        self.application_id().ok_or_else(|| HttpError::ApplicationIdMissing.into())
    }

    pub fn set_application_id(&self, application_id: ApplicationId) {
        self.application_id.store(application_id.get(), Ordering::Relaxed);
    }

    pub fn token(&self) -> &str {
        self.token.expose_secret()
    }

    /// Adds a [`User`] to a [`Guild`] with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a guild member.
    pub async fn add_guild_member(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        map: &impl serde::Serialize,
    ) -> Result<Option<Member>> {
        let body = to_vec(map)?;

        let response = self
            .request(Request {
                body: Some(body),
                multipart: None,
                headers: None,
                method: LightMethod::Put,
                route: Route::GuildMember {
                    guild_id,
                    user_id,
                },
                params: None,
            })
            .await?;

        if response.status() == 204 {
            Ok(None)
        } else {
            Ok(Some(decode_resp(response).await?))
        }
    }

    /// Adds a single [`Role`] to a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role hierarchy.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn add_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Put,
            route: Route::GuildMemberRole {
                guild_id,
                role_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Bans a [`User`] from a [`Guild`], removing their messages sent in the last X number of
    /// days.
    ///
    /// Passing a `delete_message_days` of `0` is equivalent to not removing any messages. Up to
    /// `7` days' worth of messages may be deleted.
    ///
    /// **Note**: Requires that you have the [Ban Members] permission.
    ///
    /// [Ban Members]: Permissions::BAN_MEMBERS
    pub async fn ban_user(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        delete_message_days: u8,
        reason: Option<&str>,
    ) -> Result<()> {
        let delete_message_seconds = u32::from(delete_message_days) * 86400;

        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: reason.map(reason_into_header),
            method: LightMethod::Put,
            route: Route::GuildBan {
                guild_id,
                user_id,
            },
            params: Some(vec![("delete_message_seconds", delete_message_seconds.to_string())]),
        })
        .await
    }

    /// Bans multiple users from a [`Guild`], optionally removing their messages.
    ///
    /// See the [Discord docs](https://discord.com/developers/docs/resources/guild#bulk-guild-ban)
    /// for more information.
    pub async fn bulk_ban_users(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        reason: Option<&str>,
    ) -> Result<BulkBanResponse> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildBulkBan {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Broadcasts that the current user is typing in the given [`Channel`].
    ///
    /// This lasts for about 10 seconds, and will then need to be renewed to indicate that the
    /// current user is still typing.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a long-running
    /// command is still being processed.
    pub async fn broadcast_typing(&self, channel_id: ChannelId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::ChannelTyping {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
    ///
    /// Refer to the Discord's [docs] for information on what fields this requires.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// [docs]: https://discord.com/developers/docs/resources/guild#create-guild-channel
    /// [Manage Channels]: Permissions::MANAGE_CHANNELS
    pub async fn create_channel(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildChannels {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a stage instance.
    pub async fn create_stage_instance(
        &self,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<StageInstance> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::StageInstances,
            params: None,
        })
        .await
    }

    /// Creates a thread channel in the [`GuildChannel`] given its Id, with a base message Id.
    pub async fn create_thread_from_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelMessageThreads {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a thread channel not attached to a message in the [`GuildChannel`] given its Id.
    pub async fn create_thread(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelThreads {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Shortcut for [`Self::create_forum_post_with_attachments`]
    pub async fn create_forum_post(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        self.create_forum_post_with_attachments(channel_id, map, vec![], audit_log_reason).await
    }

    /// Creates a forum post channel in the [`GuildChannel`] given its Id.
    pub async fn create_forum_post_with_attachments(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        files: Vec<CreateAttachment>,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        self.fire(Request {
            body: None,
            multipart: Some(Multipart {
                upload: MultipartUpload::Attachments(files.into_iter().collect()),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            }),
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelForumPosts {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Creates an emoji in the given [`Guild`] with the given data.
    ///
    /// See [`Guild::create_emoji`] for required fields.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_emoji(
        &self,
        guild_id: GuildId,
        map: &Value,
        audit_log_reason: Option<&str>,
    ) -> Result<Emoji> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildEmojis {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates an application emoji with the given data.
    ///
    /// See [`Context::create_application_emoji`] for required fields.
    ///
    /// [`Context::create_application_emoji`]: crate::client::Context::create_application_emoji
    pub async fn create_application_emoji(&self, map: &impl serde::Serialize) -> Result<Emoji> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::Emojis {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Create a follow-up message for an Interaction.
    ///
    /// Functions the same as [`Self::execute_webhook`]
    pub async fn create_followup_message(
        &self,
        interaction_token: &str,
        map: &impl serde::Serialize,
        files: Vec<CreateAttachment>,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::WebhookFollowupMessages {
                application_id: self.try_application_id()?,
                token: interaction_token,
            },
            params: None,
        };

        if files.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(files),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Creates a new global command.
    ///
    /// New global commands will be available in all guilds after 1 hour.
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// **Note**: Creating a command with the same name as an existing command for your application
    /// will overwrite the old command.
    ///
    /// [docs]: https://discord.com/developers/docs/interactions/slash-commands#create-global-application-command
    pub async fn create_global_command(&self, map: &impl serde::Serialize) -> Result<Command> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::Commands {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Creates new global application commands.
    pub async fn create_global_commands(
        &self,
        map: &impl serde::Serialize,
    ) -> Result<Vec<Command>> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::Commands {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Creates new guild application commands.
    pub async fn create_guild_commands(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
    ) -> Result<Vec<Command>> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::GuildCommands {
                application_id: self.try_application_id()?,
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full [`Guild`] will be received
    /// over a [`Shard`], if at least one is running.
    ///
    /// **Note**: This endpoint is currently limited to 10 active guilds. The limits are raised for
    /// whitelisted [GameBridge] applications. See the [documentation on this endpoint] for more
    /// info.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region]:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let map = json!({
    ///     "name": "test",
    /// });
    ///
    /// let _result = http.create_guild(&map).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Shard`]: crate::gateway::Shard
    /// [GameBridge]: https://discord.com/developers/docs/topics/gamebridge
    /// [documentation on this endpoint]:
    /// https://discord.com/developers/docs/resources/guild#create-guild
    /// [whitelist]: https://discord.com/developers/docs/resources/guild#create-guild
    pub async fn create_guild(&self, map: &Value) -> Result<PartialGuild> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::Guilds,
            params: None,
        })
        .await
    }

    /// Creates a new guild command.
    ///
    /// New guild commands will be available in the guild immediately.
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/interactions/slash-commands#create-guild-application-command
    pub async fn create_guild_command(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
    ) -> Result<Command> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::GuildCommands {
                application_id: self.try_application_id()?,
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates an [`Integration`] for a [`Guild`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [docs]: https://discord.com/developers/docs/resources/guild#create-guild-integration
    pub async fn create_guild_integration(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
        map: &Value,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildIntegration {
                guild_id,
                integration_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a response to an [`Interaction`] from the gateway.
    ///
    /// Refer to Discord's [docs] for the object it takes.
    ///
    /// [`Interaction`]: crate::model::application::Interaction
    /// [docs]: https://discord.com/developers/docs/interactions/slash-commands#interaction-interaction-response
    pub async fn create_interaction_response(
        &self,
        interaction_id: InteractionId,
        interaction_token: &str,
        map: &impl serde::Serialize,
        files: Vec<CreateAttachment>,
    ) -> Result<()> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::InteractionResponse {
                interaction_id,
                token: interaction_token,
            },
            params: None,
        };

        if files.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(files),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.wind(204, request).await
    }

    /// Creates a [`RichInvite`] for the given [channel][`GuildChannel`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// All fields are optional.
    ///
    /// **Note**: Requires the [Create Instant Invite] permission.
    ///
    /// [Create Instant Invite]: Permissions::CREATE_INSTANT_INVITE
    /// [docs]: https://discord.com/developers/docs/resources/channel#create-channel-invite
    pub async fn create_invite(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<RichInvite> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelInvites {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a permission override for a member or a role in a channel.
    pub async fn create_permission(
        &self,
        channel_id: ChannelId,
        target_id: TargetId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        let body = to_vec(map)?;

        self.wind(204, Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Put,
            route: Route::ChannelPermission {
                channel_id,
                target_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a private channel with a user.
    pub async fn create_private_channel(&self, map: &Value) -> Result<PrivateChannel> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::UserMeDmChannels,
            params: None,
        })
        .await
    }

    /// Reacts to a message.
    pub async fn create_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::ChannelMessageReactionMe {
                channel_id,
                message_id,
                reaction: &reaction_type.as_data(),
            },
            params: None,
        })
        .await
    }
    /// Creates a role.
    pub async fn create_role(
        &self,
        guild_id: GuildId,
        body: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Role> {
        let mut value: Value = self
            .fire(Request {
                body: Some(to_vec(body)?),
                multipart: None,
                headers: audit_log_reason.map(reason_into_header),
                method: LightMethod::Post,
                route: Route::GuildRoles {
                    guild_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Creates a Guild Scheduled Event.
    ///
    /// Refer to Discord's docs for field information.
    ///
    /// **Note**: Requires the [Create Events] permission.
    ///
    /// [Create Events]: Permissions::CREATE_EVENTS
    pub async fn create_scheduled_event(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<ScheduledEvent> {
        let body = to_vec(map)?;
        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildScheduledEvents {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a sticker.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    pub async fn create_sticker(
        &self,
        guild_id: GuildId,
        map: impl IntoIterator<Item = (&'static str, String)>,
        file: CreateAttachment,
        audit_log_reason: Option<&str>,
    ) -> Result<Sticker> {
        self.fire(Request {
            body: None,
            multipart: Some(Multipart {
                upload: MultipartUpload::File(file),
                fields: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
                payload_json: None,
            }),
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildStickers {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Creates a test entitlement to a given SKU for a given guild or user. Discord will act as
    /// though that user/guild has entitlement in perpetuity to the SKU. As a result, the returned
    /// entitlement will have `starts_at` and `ends_at` both be `None`.
    pub async fn create_test_entitlement(
        &self,
        sku_id: SkuId,
        owner: EntitlementOwner,
    ) -> Result<Entitlement> {
        #[derive(serde::Serialize)]
        struct TestEntitlement {
            sku_id: SkuId,
            owner_id: u64,
            owner_type: u8,
        }

        let (owner_id, owner_type) = match owner {
            EntitlementOwner::Guild(id) => (id.get(), 1),
            EntitlementOwner::User(id) => (id.get(), 2),
        };

        let map = TestEntitlement {
            sku_id,
            owner_id,
            owner_type,
        };

        self.fire(Request {
            body: Some(to_vec(&map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::Entitlements {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Creates a webhook for the given [channel][`GuildChannel`]'s Id, passing in the given data.
    ///
    /// This method requires authentication.
    ///
    /// The Value is a map with the values of:
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default avatar (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100 characters long.
    ///
    /// # Examples
    ///
    /// Creating a webhook named `test`:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let channel_id = ChannelId::new(81384788765712384);
    /// let map = json!({"name": "test"});
    ///
    /// let webhook = http.create_webhook(channel_id, &map, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_webhook(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Webhook> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelWebhooks {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a private channel or a channel in a guild.
    pub async fn delete_channel(
        &self,
        channel_id: ChannelId,
        audit_log_reason: Option<&str>,
    ) -> Result<Channel> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::Channel {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a stage instance.
    pub async fn delete_stage_instance(
        &self,
        channel_id: ChannelId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::StageInstance {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes an emoji from a guild.
    ///
    /// See [`GuildId::delete_emoji`] for permissions requirements.
    pub async fn delete_emoji(
        &self,
        guild_id: GuildId,
        emoji_id: EmojiId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildEmoji {
                guild_id,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes an application emoji.
    pub async fn delete_application_emoji(&self, emoji_id: EmojiId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::Emoji {
                application_id: self.try_application_id()?,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a follow-up message for an interaction.
    pub async fn delete_followup_message(
        &self,
        interaction_token: &str,
        message_id: MessageId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::WebhookFollowupMessage {
                application_id: self.try_application_id()?,
                token: interaction_token,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a global command.
    pub async fn delete_global_command(&self, command_id: CommandId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::Command {
                application_id: self.try_application_id()?,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a guild, only if connected account owns it.
    pub async fn delete_guild(&self, guild_id: GuildId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::Guild {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a guild command.
    pub async fn delete_guild_command(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::GuildCommand {
                application_id: self.try_application_id()?,
                guild_id,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Removes an integration from a guild.
    pub async fn delete_guild_integration(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildIntegration {
                guild_id,
                integration_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes an invite by code.
    pub async fn delete_invite(
        &self,
        code: &str,
        audit_log_reason: Option<&str>,
    ) -> Result<Invite> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::Invite {
                code,
            },
            params: None,
        })
        .await
    }

    /// Deletes a message if created by us or we have specific permissions.
    pub async fn delete_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::ChannelMessage {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a bunch of messages, only works for bots.
    pub async fn delete_messages(
        &self,
        channel_id: ChannelId,
        map: &Value,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::ChannelMessagesBulkDelete {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes all of the [`Reaction`]s associated with a [`Message`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// use serenity::model::id::{ChannelId, MessageId};
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let channel_id = ChannelId::new(7);
    /// let message_id = MessageId::new(8);
    ///
    /// http.delete_message_reactions(channel_id, message_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_message_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelMessageReactions {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes all the reactions for a given emoji on a message.
    pub async fn delete_message_reaction_emoji(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelMessageReactionEmoji {
                channel_id,
                message_id,
                reaction: &reaction_type.as_data(),
            },
            params: None,
        })
        .await
    }

    /// Deletes the initial interaction response.
    pub async fn delete_original_interaction_response(
        &self,
        interaction_token: &str,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::WebhookOriginalInteractionResponse {
                application_id: self.try_application_id()?,
                token: interaction_token,
            },
            params: None,
        })
        .await
    }

    /// Deletes a permission override from a role or a member in a channel.
    pub async fn delete_permission(
        &self,
        channel_id: ChannelId,
        target_id: TargetId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::ChannelPermission {
                channel_id,
                target_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a user's reaction from a message.
    pub async fn delete_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: UserId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelMessageReaction {
                channel_id,
                message_id,
                user_id,
                reaction: &reaction_type.as_data(),
            },
            params: None,
        })
        .await
    }

    /// Deletes a reaction by the current user from a message.
    pub async fn delete_reaction_me(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelMessageReactionMe {
                channel_id,
                message_id,
                reaction: &reaction_type.as_data(),
            },
            params: None,
        })
        .await
    }

    /// Deletes a role from a server. Can't remove the default everyone role.
    pub async fn delete_role(
        &self,
        guild_id: GuildId,
        role_id: RoleId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildRole {
                guild_id,
                role_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a [Scheduled Event] from a server.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// [Scheduled Event]: crate::model::guild::ScheduledEvent
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn delete_scheduled_event(
        &self,
        guild_id: GuildId,
        event_id: ScheduledEventId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::GuildScheduledEvent {
                guild_id,
                event_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a sticker from a server.
    ///
    /// See [`GuildId::delete_sticker`] for permissions requirements.
    pub async fn delete_sticker(
        &self,
        guild_id: GuildId,
        sticker_id: StickerId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildSticker {
                guild_id,
                sticker_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a currently active test entitlement. Discord will act as though the corresponding
    /// user/guild *no longer has* an entitlement to the corresponding SKU.
    pub async fn delete_test_entitlement(&self, entitlement_id: EntitlementId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::Entitlement {
                application_id: self.try_application_id()?,
                entitlement_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a [`Webhook`] given its Id.
    ///
    /// This method requires authentication, whereas [`Self::delete_webhook_with_token`] does not.
    ///
    /// # Examples
    ///
    /// Deletes a webhook given its Id:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let webhook_id = WebhookId::new(245037420704169985);
    /// http.delete_webhook(webhook_id, None).await?;
    /// Ok(())
    /// # }
    /// ```
    pub async fn delete_webhook(
        &self,
        webhook_id: WebhookId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::Webhook {
                webhook_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a [`Webhook`] given its Id and unique token.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Deletes a webhook given its Id and unique token:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// http.delete_webhook_with_token(id, token, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_webhook_with_token(
        &self,
        webhook_id: WebhookId,
        token: &str,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::WebhookWithToken {
                webhook_id,
                token,
            },
            params: None,
        })
        .await
    }

    /// Changes channel information.
    pub async fn edit_channel(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::Channel {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a stage instance.
    pub async fn edit_stage_instance(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<StageInstance> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::StageInstance {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Changes guild emoji information.
    ///
    /// See [`GuildId::edit_emoji`] for permissions requirements.
    pub async fn edit_emoji(
        &self,
        guild_id: GuildId,
        emoji_id: EmojiId,
        map: &Value,
        audit_log_reason: Option<&str>,
    ) -> Result<Emoji> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildEmoji {
                guild_id,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    /// Changes application emoji information.
    ///
    /// See [`Context::edit_application_emoji`] for required fields.
    ///
    /// [`Context::edit_application_emoji`]: crate::client::Context::edit_application_emoji
    pub async fn edit_application_emoji(
        &self,
        emoji_id: EmojiId,
        map: &impl serde::Serialize,
    ) -> Result<Emoji> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::Emoji {
                application_id: self.try_application_id()?,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a follow-up message for an interaction.
    ///
    /// Refer to Discord's [docs] for Edit Webhook Message for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/resources/webhook#edit-webhook-message
    pub async fn edit_followup_message(
        &self,
        interaction_token: &str,
        message_id: MessageId,
        map: &impl serde::Serialize,
        new_attachments: Vec<CreateAttachment>,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::WebhookFollowupMessage {
                application_id: self.try_application_id()?,
                token: interaction_token,
                message_id,
            },
            params: None,
        };

        if new_attachments.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(new_attachments),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Get a follow-up message for an interaction.
    ///
    /// Refer to Discord's [docs] for Get Webhook Message for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/resources/webhook#get-webhook-message
    pub async fn get_followup_message(
        &self,
        interaction_token: &str,
        message_id: MessageId,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::WebhookFollowupMessage {
                application_id: self.try_application_id()?,
                token: interaction_token,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a global command.
    ///
    /// Updates will be available in all guilds after 1 hour.
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/interactions/slash-commands#edit-global-application-command
    pub async fn edit_global_command(
        &self,
        command_id: CommandId,
        map: &impl serde::Serialize,
    ) -> Result<Command> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::Command {
                application_id: self.try_application_id()?,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Changes guild information.
    pub async fn edit_guild(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<PartialGuild> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::Guild {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a guild command.
    ///
    /// Updates for guild commands will be available immediately.
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/interactions/slash-commands#edit-guild-application-command
    pub async fn edit_guild_command(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
        map: &impl serde::Serialize,
    ) -> Result<Command> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::GuildCommand {
                application_id: self.try_application_id()?,
                guild_id,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a guild command permissions.
    ///
    /// Updates for guild commands will be available immediately.
    ///
    /// Refer to Discord's [documentation] for field information.
    ///
    /// [documentation]: https://discord.com/developers/docs/interactions/application-commands#edit-application-command-permissions
    pub async fn edit_guild_command_permissions(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
        map: &impl serde::Serialize,
    ) -> Result<CommandPermissions> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::GuildCommandPermissions {
                application_id: self.try_application_id()?,
                guild_id,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Edits the positions of a guild's channels.
    pub async fn edit_guild_channel_positions(
        &self,
        guild_id: GuildId,
        value: &Value,
    ) -> Result<()> {
        let body = to_vec(value)?;

        self.wind(204, Request {
            body: Some(body),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::GuildChannels {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Edits the MFA level of a guild. Requires guild ownership.
    pub async fn edit_guild_mfa_level(
        &self,
        guild_id: GuildId,
        value: &Value,
        audit_log_reason: Option<&str>,
    ) -> Result<MfaLevel> {
        #[derive(Deserialize)]
        struct GuildMfaLevel {
            level: MfaLevel,
        }

        let body = to_vec(value)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildMfa {
                guild_id,
            },
            params: None,
        })
        .await
        .map(|mfa: GuildMfaLevel| mfa.level)
    }

    /// Edits a [`Guild`]'s widget.
    pub async fn edit_guild_widget(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildWidget> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildWidget {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a guild welcome screen.
    pub async fn edit_guild_welcome_screen(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildWelcomeScreen> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildWelcomeScreen {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Does specific actions to a member.
    pub async fn edit_member(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Member> {
        let body = to_vec(map)?;

        let mut value: Value = self
            .fire(Request {
                body: Some(body),
                multipart: None,
                headers: audit_log_reason.map(reason_into_header),
                method: LightMethod::Patch,
                route: Route::GuildMember {
                    guild_id,
                    user_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value::<Member>(value)
    }

    /// Edits a message by Id.
    ///
    /// **Note**: Only the author of a message can modify it.
    pub async fn edit_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        map: &impl serde::Serialize,
        new_attachments: Vec<CreateAttachment>,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::ChannelMessage {
                channel_id,
                message_id,
            },
            params: None,
        };

        if new_attachments.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(new_attachments),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Crossposts a message by Id.
    ///
    /// **Note**: Only available on news channels.
    pub async fn crosspost_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::ChannelMessageCrosspost {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Edits the current member for the provided [`Guild`] via its Id.
    pub async fn edit_member_me(
        &self,
        guild_id: GuildId,
        map: &JsonMap,
        audit_log_reason: Option<&str>,
    ) -> Result<Member> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildMemberMe {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Edits the current user's nickname for the provided [`Guild`] via its Id.
    ///
    /// Pass [`None`] to reset the nickname.
    pub async fn edit_nickname(
        &self,
        guild_id: GuildId,
        new_nickname: Option<&str>,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        let map = json!({ "nick": new_nickname });
        let body = to_vec(&map)?;

        self.wind(200, Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildMemberMe {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Follow a News Channel to send messages to a target channel.
    pub async fn follow_news_channel(
        &self,
        news_channel_id: ChannelId,
        target_channel_id: ChannelId,
    ) -> Result<FollowedChannel> {
        let map = json!({ "webhook_channel_id": target_channel_id });
        let body = to_vec(&map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::ChannelFollowNews {
                channel_id: news_channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets the initial interaction response.
    pub async fn get_original_interaction_response(
        &self,
        interaction_token: &str,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::WebhookOriginalInteractionResponse {
                application_id: self.try_application_id()?,
                token: interaction_token,
            },
            params: None,
        })
        .await
    }

    /// Edits the initial interaction response.
    ///
    /// Refer to Discord's [docs] for Edit Webhook Message for field information.
    ///
    /// [docs]: https://discord.com/developers/docs/resources/webhook#edit-webhook-message
    pub async fn edit_original_interaction_response(
        &self,
        interaction_token: &str,
        map: &impl serde::Serialize,
        new_attachments: Vec<CreateAttachment>,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::WebhookOriginalInteractionResponse {
                application_id: self.try_application_id()?,
                token: interaction_token,
            },
            params: None,
        };

        if new_attachments.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(new_attachments.into_iter().collect()),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Edits the current user's profile settings.
    pub async fn edit_profile(&self, map: &impl serde::Serialize) -> Result<CurrentUser> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::UserMe,
            params: None,
        })
        .await
    }

    /// Changes a role in a guild.
    pub async fn edit_role(
        &self,
        guild_id: GuildId,
        role_id: RoleId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Role> {
        let mut value: Value = self
            .fire(Request {
                body: Some(to_vec(map)?),
                multipart: None,
                headers: audit_log_reason.map(reason_into_header),
                method: LightMethod::Patch,
                route: Route::GuildRole {
                    guild_id,
                    role_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Changes the position of a role in a guild.
    pub async fn edit_role_position(
        &self,
        guild_id: GuildId,
        role_id: RoleId,
        position: u16,
        audit_log_reason: Option<&str>,
    ) -> Result<Vec<Role>> {
        let map = json!([{
            "id": role_id,
            "position": position,
        }]);
        let body = to_vec(&map)?;

        let mut value: Value = self
            .fire(Request {
                body: Some(body),
                multipart: None,
                headers: audit_log_reason.map(reason_into_header),
                method: LightMethod::Patch,
                route: Route::GuildRoles {
                    guild_id,
                },
                params: None,
            })
            .await?;

        if let Some(array) = value.as_array_mut() {
            for role in array {
                if let Some(map) = role.as_object_mut() {
                    map.insert("guild_id".to_string(), guild_id.get().into());
                }
            }
        }

        from_value(value)
    }

    /// Modifies a scheduled event.
    ///
    /// **Note**: Requires the [Manage Events] permission.
    ///
    /// [Manage Events]: Permissions::MANAGE_EVENTS
    pub async fn edit_scheduled_event(
        &self,
        guild_id: GuildId,
        event_id: ScheduledEventId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<ScheduledEvent> {
        let body = to_vec(map)?;
        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildScheduledEvent {
                guild_id,
                event_id,
            },
            params: None,
        })
        .await
    }

    /// Changes a sticker in a guild.
    ///
    /// See [`GuildId::edit_sticker`] for permissions requirements.
    pub async fn edit_sticker(
        &self,
        guild_id: GuildId,
        sticker_id: StickerId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Sticker> {
        let body = to_vec(&map)?;

        let mut value: Value = self
            .fire(Request {
                body: Some(body),
                multipart: None,
                headers: audit_log_reason.map(reason_into_header),
                method: LightMethod::Patch,
                route: Route::GuildSticker {
                    guild_id,
                    sticker_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Edits a thread channel in the [`GuildChannel`] given its Id.
    pub async fn edit_thread(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildChannel> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::Channel {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Changes another user's voice state in a stage channel.
    ///
    /// The Value is a map with values of:
    /// - **channel_id**: ID of the channel the user is currently in (**required**)
    /// - **suppress**: Bool which toggles user's suppressed state. Setting this to `false` will
    ///   invite the user to speak.
    ///
    /// # Example
    ///
    /// Suppress a user
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let guild_id = GuildId::new(187450744427773963);
    /// let user_id = UserId::new(150443906511667200);
    /// let map = json!({
    ///     "channel_id": "826929611849334784",
    ///     "suppress": true,
    /// });
    ///
    /// // Edit state for another user
    /// http.edit_voice_state(guild_id, user_id, &map).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edit_voice_state(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        map: &impl serde::Serialize,
    ) -> Result<()> {
        self.wind(204, Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::GuildVoiceStates {
                guild_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Changes the current user's voice state in a stage channel.
    ///
    /// The Value is a map with values of:
    ///
    /// - **channel_id**: ID of the channel the user is currently in (**required**)
    /// - **suppress**: Bool which toggles user's suppressed state. Setting this to `false` will
    ///   invite the user to speak.
    /// - **request_to_speak_timestamp**: ISO8601 timestamp to set the user's request to speak. This
    ///   can be any present or future time.
    ///
    /// # Example
    ///
    /// Unsuppress the current bot user
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let guild_id = GuildId::new(187450744427773963);
    /// let map = json!({
    ///     "channel_id": "826929611849334784",
    ///     "suppress": false,
    ///     "request_to_speak_timestamp": "2021-03-31T18:45:31.297561+00:00"
    /// });
    ///
    /// // Edit state for current user
    /// http.edit_voice_state_me(guild_id, &map).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edit_voice_state_me(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
    ) -> Result<()> {
        self.wind(204, Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::GuildVoiceStateMe {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Changes a voice channel's status.
    pub async fn edit_voice_status(
        &self,
        channel_id: ChannelId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        let body = to_vec(map)?;

        self.wind(204, Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Put,
            route: Route::ChannelVoiceStatus {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Edits a the webhook with the given data.
    ///
    /// The Value is a map with optional values of:
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default avatar (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100 characters long.
    ///
    /// Note that, unlike with [`Self::create_webhook`], _all_ values are optional.
    ///
    /// This method requires authentication, whereas [`Self::edit_webhook_with_token`] does not.
    ///
    /// # Examples
    ///
    /// Edit the image of a webhook given its Id and unique token:
    ///
    /// ```rust,no_run
    /// use serenity::builder::CreateAttachment;
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let image = CreateAttachment::path("./webhook_img.png").await?;
    /// let map = json!({
    ///     "avatar": image.to_base64(),
    /// });
    ///
    /// let edited = http.edit_webhook(id, &map, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edit_webhook(
        &self,
        webhook_id: WebhookId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Webhook> {
        self.fire(Request {
            body: Some(to_vec(map)?),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::Webhook {
                webhook_id,
            },
            params: None,
        })
        .await
    }

    /// Edits the webhook with the given data.
    ///
    /// Refer to the documentation for [`Self::edit_webhook`] for more information.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Edit the name of a webhook given its Id and unique token:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let map = json!({"name": "new name"});
    ///
    /// let edited = http.edit_webhook_with_token(id, token, &map, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edit_webhook_with_token(
        &self,
        webhook_id: WebhookId,
        token: &str,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Webhook> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::WebhookWithToken {
                webhook_id,
                token,
            },
            params: None,
        })
        .await
    }

    /// Executes a webhook, posting a [`Message`] in the webhook's associated [`Channel`].
    ///
    /// This method does _not_ require authentication.
    ///
    /// If `thread_id` is not `None`, then the message will be sent to the thread in the webhook's
    /// associated [`Channel`] with the corresponding Id, which will be automatically unarchived.
    ///
    /// If `wait` is `false`, this function will return `Ok(None)` on success. Otherwise, it will
    /// wait for server confirmation of the message having been sent, and return `Ok(Some(msg))`.
    /// From the [Discord docs]:
    ///
    /// > waits for server confirmation of message send before response, and returns the created
    /// > message body (defaults to false; when false a message that is not saved does not return
    /// > an error)
    ///
    /// The map can _optionally_ contain the following data:
    /// - `avatar_url`: Override the default avatar of the webhook with a URL.
    /// - `tts`: Whether this is a text-to-speech message (defaults to `false`).
    /// - `username`: Override the default username of the webhook.
    ///
    /// Additionally, _at least one_ of the following must be given:
    /// - `content`: The content of the message.
    /// - `embeds`: An array of rich embeds.
    ///
    /// **Note**: For embed objects, all fields are registered by Discord except for `height`,
    /// `provider`, `proxy_url`, `type` (it will always be `rich`), `video`, and `width`. The rest
    /// will be determined by Discord.
    ///
    /// # Examples
    ///
    /// Sending a webhook with message content of `test`:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    /// use serenity::json::json;
    /// use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let map = json!({"content": "test"});
    /// let files = vec![];
    ///
    /// let message = http.execute_webhook(id, None, token, true, files, &map).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [Discord docs]: https://discord.com/developers/docs/resources/webhook#execute-webhook-query-string-params
    pub async fn execute_webhook(
        &self,
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
        token: &str,
        wait: bool,
        files: Vec<CreateAttachment>,
        map: &impl serde::Serialize,
    ) -> Result<Option<Message>> {
        let mut params = vec![("wait", wait.to_string())];
        if let Some(thread_id) = thread_id {
            params.push(("thread_id", thread_id.to_string()));
        }

        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::WebhookWithToken {
                webhook_id,
                token,
            },
            params: Some(params),
        };

        if files.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(files.into_iter().collect()),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        let response = self.request(request).await?;

        Ok(if response.status() == StatusCode::NO_CONTENT {
            None
        } else {
            decode_resp(response).await?
        })
    }

    // Gets a webhook's message by Id
    pub async fn get_webhook_message(
        &self,
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
        token: &str,
        message_id: MessageId,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::WebhookMessage {
                webhook_id,
                token,
                message_id,
            },
            params: thread_id.map(|thread_id| vec![("thread_id", thread_id.to_string())]),
        })
        .await
    }

    /// Edits a webhook's message by Id.
    pub async fn edit_webhook_message(
        &self,
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
        token: &str,
        message_id: MessageId,
        map: &impl serde::Serialize,
        new_attachments: Vec<CreateAttachment>,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Patch,
            route: Route::WebhookMessage {
                webhook_id,
                token,
                message_id,
            },
            params: thread_id.map(|thread_id| vec![("thread_id", thread_id.to_string())]),
        };

        if new_attachments.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(new_attachments),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Deletes a webhook's message by Id.
    pub async fn delete_webhook_message(
        &self,
        webhook_id: WebhookId,
        thread_id: Option<ChannelId>,
        token: &str,
        message_id: MessageId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::WebhookMessage {
                webhook_id,
                token,
                message_id,
            },
            params: thread_id.map(|thread_id| vec![("thread_id", thread_id.to_string())]),
        })
        .await
    }

    /// Gets the active maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_active_maintenances(&self) -> Result<Vec<Maintenance>> {
        #[derive(Deserialize)]
        struct StatusResponse {
            #[serde(default)]
            scheduled_maintenances: Vec<Maintenance>,
        }

        let status: StatusResponse = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::StatusMaintenancesActive,
                params: None,
            })
            .await?;

        Ok(status.scheduled_maintenances)
    }

    /// Gets all the users that are banned in specific guild, with additional options for
    /// filtering.
    ///
    /// If `limit` is left unset, by default at most 1000 worths of data for banned users is
    /// returned.
    ///
    /// If `target` is set, then users will be filtered by Id, such that their Id comes before or
    /// after the provided [`UserId`] wrapped by the [`UserPagination`].
    ///
    /// [`UserId`]: crate::model::id::UserId
    pub async fn get_bans(
        &self,
        guild_id: GuildId,
        target: Option<UserPagination>,
        limit: Option<u8>,
    ) -> Result<Vec<Ban>> {
        let mut params = vec![];

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }

        if let Some(target) = target {
            match target {
                UserPagination::After(id) => params.push(("after", id.to_string())),
                UserPagination::Before(id) => params.push(("before", id.to_string())),
            }
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildBans {
                guild_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Gets all audit logs in a specific guild.
    pub async fn get_audit_logs(
        &self,
        guild_id: GuildId,
        action_type: Option<audit_log::Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> Result<AuditLogs> {
        let mut params = vec![];
        if let Some(action_type) = action_type {
            params.push(("action_type", action_type.num().to_string()));
        }
        if let Some(before) = before {
            params.push(("before", before.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(user_id) = user_id {
            params.push(("user_id", user_id.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildAuditLogs {
                guild_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Retrieves all auto moderation rules in a guild.
    ///
    /// This method requires `MANAGE_GUILD` permissions.
    pub async fn get_automod_rules(&self, guild_id: GuildId) -> Result<Vec<Rule>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildAutomodRules {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Retrieves an auto moderation rule in a guild.
    ///
    /// This method requires `MANAGE_GUILD` permissions.
    pub async fn get_automod_rule(&self, guild_id: GuildId, rule_id: RuleId) -> Result<Rule> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildAutomodRule {
                guild_id,
                rule_id,
            },
            params: None,
        })
        .await
    }

    /// Creates an auto moderation rule in a guild.
    ///
    /// This method requires `MANAGE_GUILD` permissions.
    pub async fn create_automod_rule(
        &self,
        guild_id: GuildId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Rule> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildAutomodRules {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Retrieves an auto moderation rule in a guild.
    ///
    /// This method requires `MANAGE_GUILD` permissions.
    pub async fn edit_automod_rule(
        &self,
        guild_id: GuildId,
        rule_id: RuleId,
        map: &impl serde::Serialize,
        audit_log_reason: Option<&str>,
    ) -> Result<Rule> {
        let body = to_vec(map)?;

        self.fire(Request {
            body: Some(body),
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Patch,
            route: Route::GuildAutomodRule {
                guild_id,
                rule_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes an auto moderation rule in a guild.
    ///
    /// This method requires `MANAGE_GUILD` permissions.
    pub async fn delete_automod_rule(
        &self,
        guild_id: GuildId,
        rule_id: RuleId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildAutomodRule {
                guild_id,
                rule_id,
            },
            params: None,
        })
        .await
    }

    /// Gets current bot gateway.
    pub async fn get_bot_gateway(&self) -> Result<BotGateway> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GatewayBot,
            params: None,
        })
        .await
    }

    /// Gets all invites for a channel.
    pub async fn get_channel_invites(&self, channel_id: ChannelId) -> Result<Vec<RichInvite>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelInvites {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all thread members for a thread.
    pub async fn get_channel_thread_members(
        &self,
        channel_id: ChannelId,
    ) -> Result<Vec<ThreadMember>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelThreadMembers {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all active threads from a guild.
    pub async fn get_guild_active_threads(&self, guild_id: GuildId) -> Result<ThreadsData> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildThreadsActive {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all archived public threads from a channel.
    pub async fn get_channel_archived_public_threads(
        &self,
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        let mut params = vec![];
        if let Some(before) = before {
            params.push(("before", before.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            method: LightMethod::Get,
            headers: None,
            route: Route::ChannelArchivedPublicThreads {
                channel_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Gets all archived private threads from a channel.
    pub async fn get_channel_archived_private_threads(
        &self,
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        let mut params = vec![];
        if let Some(before) = before {
            params.push(("before", before.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelArchivedPrivateThreads {
                channel_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Gets all archived private threads joined from a channel.
    pub async fn get_channel_joined_archived_private_threads(
        &self,
        channel_id: ChannelId,
        before: Option<u64>,
        limit: Option<u64>,
    ) -> Result<ThreadsData> {
        let mut params = vec![];
        if let Some(before) = before {
            params.push(("before", before.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelJoinedPrivateThreads {
                channel_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Joins a thread channel.
    pub async fn join_thread_channel(&self, channel_id: ChannelId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::ChannelThreadMemberMe {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Leaves a thread channel.
    pub async fn leave_thread_channel(&self, channel_id: ChannelId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelThreadMemberMe {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Adds a member to a thread channel.
    pub async fn add_thread_channel_member(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Put,
            route: Route::ChannelThreadMember {
                channel_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Removes a member from a thread channel.
    pub async fn remove_thread_channel_member(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::ChannelThreadMember {
                channel_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    pub async fn get_thread_channel_member(
        &self,
        channel_id: ChannelId,
        user_id: UserId,
        with_member: bool,
    ) -> Result<ThreadMember> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelThreadMember {
                channel_id,
                user_id,
            },
            params: Some(vec![("with_member", with_member.to_string())]),
        })
        .await
    }

    /// Retrieves the webhooks for the given [channel][`GuildChannel`]'s Id.
    ///
    /// This method requires authentication.
    ///
    /// # Examples
    ///
    /// Retrieve all of the webhooks owned by a channel:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let channel_id = ChannelId::new(81384788765712384);
    ///
    /// let webhooks = http.get_channel_webhooks(channel_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_channel_webhooks(&self, channel_id: ChannelId) -> Result<Vec<Webhook>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelWebhooks {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets channel information.
    pub async fn get_channel(&self, channel_id: ChannelId) -> Result<Channel> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Channel {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all channels in a guild.
    pub async fn get_channels(&self, guild_id: GuildId) -> Result<Vec<GuildChannel>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildChannels {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a stage instance.
    pub async fn get_stage_instance(&self, channel_id: ChannelId) -> Result<StageInstance> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::StageInstance {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Get a list of users that voted for this specific answer.
    pub async fn get_poll_answer_voters(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        answer_id: AnswerId,
        after: Option<UserId>,
        limit: Option<u8>,
    ) -> Result<Vec<User>> {
        #[derive(Deserialize)]
        struct VotersResponse {
            users: Vec<User>,
        }

        let mut params = Vec::with_capacity(2);
        if let Some(after) = after {
            params.push(("after", after.to_string()));
        }

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }

        let resp: VotersResponse = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::ChannelPollGetAnswerVoters {
                    channel_id,
                    message_id,
                    answer_id,
                },
                params: Some(params),
            })
            .await?;

        Ok(resp.users)
    }

    pub async fn expire_poll(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::ChannelPollExpire {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Gets information about the current application.
    ///
    /// **Note**: Only applications may use this endpoint.
    pub async fn get_current_application_info(&self) -> Result<CurrentApplicationInfo> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Oauth2ApplicationCurrent,
            params: None,
        })
        .await
    }

    /// Gets information about the user we're connected with.
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::UserMe,
            params: None,
        })
        .await
    }

    /// Gets all emojis of a guild.
    pub async fn get_emojis(&self, guild_id: GuildId) -> Result<Vec<Emoji>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildEmojis {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets information about an emoji in a guild.
    pub async fn get_emoji(&self, guild_id: GuildId, emoji_id: EmojiId) -> Result<Emoji> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildEmoji {
                guild_id,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all emojis for the current application.
    pub async fn get_application_emojis(&self) -> Result<Vec<Emoji>> {
        // Why, discord...
        #[derive(Deserialize)]
        struct ApplicationEmojis {
            items: Vec<Emoji>,
        }

        let result: ApplicationEmojis = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::Emojis {
                    application_id: self.try_application_id()?,
                },
                params: None,
            })
            .await?;

        Ok(result.items)
    }

    /// Gets information about an application emoji.
    pub async fn get_application_emoji(&self, emoji_id: EmojiId) -> Result<Emoji> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Emoji {
                application_id: self.try_application_id()?,
                emoji_id,
            },
            params: None,
        })
        .await
    }

    #[allow(clippy::too_many_arguments)]
    /// Gets all entitlements for the current app, active and expired.
    pub async fn get_entitlements(
        &self,
        user_id: Option<UserId>,
        sku_ids: Option<Vec<SkuId>>,
        before: Option<EntitlementId>,
        after: Option<EntitlementId>,
        limit: Option<u8>,
        guild_id: Option<GuildId>,
        exclude_ended: Option<bool>,
    ) -> Result<Vec<Entitlement>> {
        let mut params = vec![];
        if let Some(user_id) = user_id {
            params.push(("user_id", user_id.to_string()));
        }
        if let Some(sku_ids) = sku_ids {
            params.push((
                "sku_ids",
                sku_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(","),
            ));
        }
        if let Some(before) = before {
            params.push(("before", before.to_string()));
        }
        if let Some(after) = after {
            params.push(("after", after.to_string()));
        }
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(guild_id) = guild_id {
            params.push(("guild_id", guild_id.to_string()));
        }
        if let Some(exclude_ended) = exclude_ended {
            params.push(("exclude_ended", exclude_ended.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Entitlements {
                application_id: self.try_application_id()?,
            },
            params: Some(params),
        })
        .await
    }

    /// Gets current gateway.
    pub async fn get_gateway(&self) -> Result<Gateway> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Gateway,
            params: None,
        })
        .await
    }

    /// Fetches all of the global commands for your application.
    pub async fn get_global_commands(&self) -> Result<Vec<Command>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Commands {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Fetches all of the global commands for your application with localizations.
    pub async fn get_global_commands_with_localizations(&self) -> Result<Vec<Command>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Commands {
                application_id: self.try_application_id()?,
            },
            params: Some(vec![("with_localizations", true.to_string())]),
        })
        .await
    }

    /// Fetches a global commands for your application by its Id.
    pub async fn get_global_command(&self, command_id: CommandId) -> Result<Command> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Command {
                application_id: self.try_application_id()?,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Gets guild information.
    pub async fn get_guild(&self, guild_id: GuildId) -> Result<PartialGuild> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Guild {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets guild information with counts.
    pub async fn get_guild_with_counts(&self, guild_id: GuildId) -> Result<PartialGuild> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Guild {
                guild_id,
            },
            params: Some(vec![("with_counts", true.to_string())]),
        })
        .await
    }

    /// Fetches all of the guild commands for your application for a specific guild.
    pub async fn get_guild_commands(&self, guild_id: GuildId) -> Result<Vec<Command>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildCommands {
                application_id: self.try_application_id()?,
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Fetches all of the guild commands with localizations for your application for a specific
    /// guild.
    pub async fn get_guild_commands_with_localizations(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<Command>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildCommands {
                application_id: self.try_application_id()?,
                guild_id,
            },
            params: Some(vec![("with_localizations", true.to_string())]),
        })
        .await
    }

    /// Fetches a guild command by its Id.
    pub async fn get_guild_command(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<Command> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildCommand {
                application_id: self.try_application_id()?,
                guild_id,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Fetches all of the guild commands permissions for your application for a specific guild.
    pub async fn get_guild_commands_permissions(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<CommandPermissions>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildCommandsPermissions {
                application_id: self.try_application_id()?,
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gives the guild command permission for your application for a specific guild.
    pub async fn get_guild_command_permissions(
        &self,
        guild_id: GuildId,
        command_id: CommandId,
    ) -> Result<CommandPermissions> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildCommandPermissions {
                application_id: self.try_application_id()?,
                guild_id,
                command_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a guild widget information.
    // TODO: according to Discord, this returns different data; namely https://discord.com/developers/docs/resources/guild#guild-widget-object-guild-widget-structure.
    // Should investigate if this endpoint actually works
    pub async fn get_guild_widget(&self, guild_id: GuildId) -> Result<GuildWidget> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildWidget {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a guild preview.
    pub async fn get_guild_preview(&self, guild_id: GuildId) -> Result<GuildPreview> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildPreview {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a guild welcome screen information.
    pub async fn get_guild_welcome_screen(&self, guild_id: GuildId) -> Result<GuildWelcomeScreen> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildWelcomeScreen {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets integrations that a guild has.
    pub async fn get_guild_integrations(&self, guild_id: GuildId) -> Result<Vec<Integration>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildIntegrations {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets all invites to a guild.
    pub async fn get_guild_invites(&self, guild_id: GuildId) -> Result<Vec<RichInvite>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildInvites {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a guild's vanity URL if it has one.
    pub async fn get_guild_vanity_url(&self, guild_id: GuildId) -> Result<String> {
        #[derive(Deserialize)]
        struct GuildVanityUrl {
            code: String,
        }

        self.fire::<GuildVanityUrl>(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildVanityUrl {
                guild_id,
            },
            params: None,
        })
        .await
        .map(|x| x.code)
    }

    /// Gets the members of a guild. Optionally pass a `limit` and the Id of the user to offset the
    /// result by.
    pub async fn get_guild_members(
        &self,
        guild_id: GuildId,
        limit: Option<u64>,
        after: Option<u64>,
    ) -> Result<Vec<Member>> {
        if let Some(l) = limit {
            if !(1..=constants::MEMBER_FETCH_LIMIT).contains(&l) {
                return Err(Error::NotInRange("limit", l, 1, constants::MEMBER_FETCH_LIMIT));
            }
        }

        let mut params =
            vec![("limit", limit.unwrap_or(constants::MEMBER_FETCH_LIMIT).to_string())];
        if let Some(after) = after {
            params.push(("after", after.to_string()));
        }

        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildMembers {
                    guild_id,
                },
                params: Some(params),
            })
            .await?;

        if let Some(values) = value.as_array_mut() {
            for value in values {
                if let Some(element) = value.as_object_mut() {
                    element.insert("guild_id".to_string(), guild_id.get().into());
                }
            }
        }

        from_value(value)
    }

    /// Gets the amount of users that can be pruned.
    pub async fn get_guild_prune_count(&self, guild_id: GuildId, days: u8) -> Result<GuildPrune> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildPrune {
                guild_id,
            },
            params: Some(vec![("days", days.to_string())]),
        })
        .await
    }

    /// Gets regions that a guild can use. If a guild has the `VIP_REGIONS` feature enabled, then
    /// additional VIP-only regions are returned.
    pub async fn get_guild_regions(&self, guild_id: GuildId) -> Result<Vec<VoiceRegion>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildRegions {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Retrieves a specific role in a [`Guild`].
    pub async fn get_guild_role(&self, guild_id: GuildId, role_id: RoleId) -> Result<Role> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildRole {
                    guild_id,
                    role_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Retrieves a list of roles in a [`Guild`].
    pub async fn get_guild_roles(&self, guild_id: GuildId) -> Result<Vec<Role>> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildRoles {
                    guild_id,
                },
                params: None,
            })
            .await?;

        if let Some(array) = value.as_array_mut() {
            for sticker in array {
                if let Some(map) = sticker.as_object_mut() {
                    map.insert("guild_id".to_string(), guild_id.get().into());
                }
            }
        }

        from_value(value)
    }

    /// Gets a scheduled event by Id.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn get_scheduled_event(
        &self,
        guild_id: GuildId,
        event_id: ScheduledEventId,
        with_user_count: bool,
    ) -> Result<ScheduledEvent> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildScheduledEvent {
                guild_id,
                event_id,
            },
            params: Some(vec![("with_user_count", with_user_count.to_string())]),
        })
        .await
    }

    /// Gets a list of all scheduled events for the corresponding guild.
    ///
    /// **Note**: Requires the [View Channel] permission at the guild level.
    ///
    /// [View Channel]: Permissions::VIEW_CHANNEL
    pub async fn get_scheduled_events(
        &self,
        guild_id: GuildId,
        with_user_count: bool,
    ) -> Result<Vec<ScheduledEvent>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildScheduledEvents {
                guild_id,
            },
            params: Some(vec![("with_user_count", with_user_count.to_string())]),
        })
        .await
    }

    /// Gets a list of all interested users for the corresponding scheduled event, with additional
    /// options for filtering.
    ///
    /// If `limit` is left unset, by default at most 100 users are returned.
    ///
    /// If `target` is set, then users will be filtered by Id, such that their Id comes before or
    /// after the provided [`UserId`] wrapped by the [`UserPagination`].
    ///
    /// If `with_member` is set to `Some(true)`, then the [`member`] field of the user struct will
    /// be populated with [`Guild Member`] information, if the interested user is a member of the
    /// guild the event takes place in.
    ///
    /// **Note**: Requires the [View Channel] permission for the channel associated with the event.
    ///
    /// [`member`]: ScheduledEventUser::member
    /// [`Guild Member`]: crate::model::guild::Member
    pub async fn get_scheduled_event_users(
        &self,
        guild_id: GuildId,
        event_id: ScheduledEventId,
        limit: Option<u64>,
        target: Option<UserPagination>,
        with_member: Option<bool>,
    ) -> Result<Vec<ScheduledEventUser>> {
        let mut params = vec![];
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(with_member) = with_member {
            params.push(("with_member", with_member.to_string()));
        }
        if let Some(target) = target {
            match target {
                UserPagination::After(id) => params.push(("after", id.to_string())),
                UserPagination::Before(id) => params.push(("before", id.to_string())),
            }
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildScheduledEventUsers {
                guild_id,
                event_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Retrieves a list of stickers in a [`Guild`].
    pub async fn get_guild_stickers(&self, guild_id: GuildId) -> Result<Vec<Sticker>> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildStickers {
                    guild_id,
                },
                params: None,
            })
            .await?;

        if let Some(array) = value.as_array_mut() {
            for role in array {
                if let Some(map) = role.as_object_mut() {
                    map.insert("guild_id".to_string(), guild_id.get().into());
                }
            }
        }

        from_value(value)
    }

    /// Retrieves a single sticker in a [`Guild`].
    pub async fn get_guild_sticker(
        &self,
        guild_id: GuildId,
        sticker_id: StickerId,
    ) -> Result<Sticker> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildSticker {
                    guild_id,
                    sticker_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Retrieves the webhooks for the given [guild][`Guild`]'s Id.
    ///
    /// This method requires authentication.
    ///
    /// # Examples
    ///
    /// Retrieve all of the webhooks owned by a guild:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let guild_id = GuildId::new(81384788765712384);
    ///
    /// let webhooks = http.get_guild_webhooks(guild_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_guild_webhooks(&self, guild_id: GuildId) -> Result<Vec<Webhook>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::GuildWebhooks {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Gets a paginated list of the current user's guilds.
    ///
    /// The `limit` has a maximum value of 100.
    ///
    /// [Discord's documentation][docs]
    ///
    /// # Examples
    ///
    /// Get the first 10 guilds after a certain guild's Id:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// use serenity::http::GuildPagination;
    /// use serenity::model::id::GuildId;
    ///
    /// let guild_id = GuildId::new(81384788765712384);
    ///
    /// let guilds = http.get_guilds(Some(GuildPagination::After(guild_id)), Some(10)).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [docs]: https://discord.com/developers/docs/resources/user#get-current-user-guilds
    pub async fn get_guilds(
        &self,
        target: Option<GuildPagination>,
        limit: Option<u64>,
    ) -> Result<Vec<GuildInfo>> {
        let mut params = vec![];
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(target) = target {
            match target {
                GuildPagination::After(id) => params.push(("after", id.to_string())),
                GuildPagination::Before(id) => params.push(("before", id.to_string())),
            }
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::UserMeGuilds,
            params: Some(params),
        })
        .await
    }

    /// Returns a guild [`Member`] object for the current user.
    ///
    /// # Authorization
    ///
    /// This method only works for user tokens with the [`GuildsMembersRead`] OAuth2 scope.
    ///
    /// [`GuildsMembersRead`]: crate::model::application::Scope::GuildsMembersRead
    ///
    /// # Examples
    ///
    /// Get the member object for the current user within the specified guild.
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// use serenity::model::id::GuildId;
    ///
    /// let guild_id = GuildId::new(81384788765712384);
    ///
    /// let member = http.get_current_user_guild_member(guild_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See the [Discord Developer Portal documentation][docs] for more.
    ///
    /// [docs]: https://discord.com/developers/docs/resources/user#get-current-user-guild-member
    pub async fn get_current_user_guild_member(&self, guild_id: GuildId) -> Result<Member> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::UserMeGuildMember {
                    guild_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Gets information about a specific invite.
    ///
    /// # Arguments
    /// * `code` - The invite code.
    /// * `member_counts` - Whether to include information about the current number of members in
    ///   the server that the invite belongs to.
    /// * `expiration` - Whether to include information about when the invite expires.
    /// * `event_id` - An optional server event ID to include with the invite.
    ///
    /// More information about these arguments can be found on Discord's
    /// [API documentation](https://discord.com/developers/docs/resources/invite#get-invite).
    pub async fn get_invite(
        &self,
        code: &str,
        member_counts: bool,
        expiration: bool,
        event_id: Option<ScheduledEventId>,
    ) -> Result<Invite> {
        #[cfg(feature = "utils")]
        let code = crate::utils::parse_invite(code);

        let mut params = vec![
            ("member_counts", member_counts.to_string()),
            ("expiration", expiration.to_string()),
        ];
        if let Some(event_id) = event_id {
            params.push(("event_id", event_id.to_string()));
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Invite {
                code,
            },
            params: Some(params),
        })
        .await
    }

    /// Gets member of a guild.
    pub async fn get_member(&self, guild_id: GuildId, user_id: UserId) -> Result<Member> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildMember {
                    guild_id,
                    user_id,
                },
                params: None,
            })
            .await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), guild_id.get().into());
        }

        from_value(value)
    }

    /// Gets a message by an Id, bots only.
    pub async fn get_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<Message> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelMessage {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Gets X messages from a channel.
    pub async fn get_messages(
        &self,
        channel_id: ChannelId,
        target: Option<MessagePagination>,
        limit: Option<u8>,
    ) -> Result<Vec<Message>> {
        let mut params = vec![];
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(target) = target {
            match target {
                MessagePagination::After(id) => params.push(("after", id.to_string())),
                MessagePagination::Around(id) => params.push(("around", id.to_string())),
                MessagePagination::Before(id) => params.push(("before", id.to_string())),
            }
        }

        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelMessages {
                channel_id,
            },
            params: Some(params),
        })
        .await
    }

    /// Retrieves a specific [`StickerPack`] from it's [`StickerPackId`]
    pub async fn get_sticker_pack(&self, sticker_pack_id: StickerPackId) -> Result<StickerPack> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::StickerPack {
                sticker_pack_id,
            },
            params: None,
        })
        .await
    }

    /// Retrieves a list of all nitro sticker packs.
    pub async fn get_nitro_stickers(&self) -> Result<Vec<StickerPack>> {
        #[derive(Deserialize)]
        struct StickerPacks {
            sticker_packs: Vec<StickerPack>,
        }

        self.fire::<StickerPacks>(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::StickerPacks,
            params: None,
        })
        .await
        .map(|s| s.sticker_packs)
    }

    /// Gets all pins of a channel.
    pub async fn get_pins(&self, channel_id: ChannelId) -> Result<Vec<Message>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelPins {
                channel_id,
            },
            params: None,
        })
        .await
    }

    /// Gets user Ids based on their reaction to a message. This endpoint is dumb.
    pub async fn get_reaction_users(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType,
        limit: u8,
        after: Option<u64>,
    ) -> Result<Vec<User>> {
        let mut params = vec![("limit", limit.to_string())];
        if let Some(after) = after {
            params.push(("after", after.to_string()));
        }
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::ChannelMessageReactionEmoji {
                channel_id,
                message_id,
                reaction: &reaction_type.as_data(),
            },
            params: Some(params),
        })
        .await
    }

    /// Gets all SKUs for the current application.
    pub async fn get_skus(&self) -> Result<Vec<Sku>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Skus {
                application_id: self.try_application_id()?,
            },
            params: None,
        })
        .await
    }

    /// Gets a sticker.
    pub async fn get_sticker(&self, sticker_id: StickerId) -> Result<Sticker> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Sticker {
                sticker_id,
            },
            params: None,
        })
        .await
    }

    /// Gets the current unresolved incidents from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_unresolved_incidents(&self) -> Result<Vec<Incident>> {
        #[derive(Deserialize)]
        struct StatusResponse {
            #[serde(default)]
            incidents: Vec<Incident>,
        }

        let status: StatusResponse = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::StatusIncidentsUnresolved,
                params: None,
            })
            .await?;

        Ok(status.incidents)
    }

    /// Gets the upcoming (planned) maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_upcoming_maintenances(&self) -> Result<Vec<Maintenance>> {
        #[derive(Deserialize)]
        struct StatusResponse {
            #[serde(default)]
            scheduled_maintenances: Vec<Maintenance>,
        }

        let status: StatusResponse = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::StatusMaintenancesUpcoming,
                params: None,
            })
            .await?;

        Ok(status.scheduled_maintenances)
    }

    /// Gets a user by Id.
    pub async fn get_user(&self, user_id: UserId) -> Result<User> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::User {
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Gets the current user's third party connections.
    ///
    /// This method only works for user tokens with the [`Connections`] OAuth2 scope.
    ///
    /// [`Connections`]: crate::model::application::Scope::Connections
    pub async fn get_user_connections(&self) -> Result<Vec<Connection>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::UserMeConnections,
            params: None,
        })
        .await
    }

    /// Gets our DM channels.
    pub async fn get_user_dm_channels(&self) -> Result<Vec<PrivateChannel>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::UserMeDmChannels,
            params: None,
        })
        .await
    }

    /// Gets all voice regions.
    pub async fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::VoiceRegions,
            params: None,
        })
        .await
    }

    /// Retrieves a webhook given its Id.
    ///
    /// This method requires authentication, whereas [`Http::get_webhook_with_token`] and
    /// [`Http::get_webhook_from_url`] do not.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let webhook = http.get_webhook(id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_webhook(&self, webhook_id: WebhookId) -> Result<Webhook> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::Webhook {
                webhook_id,
            },
            params: None,
        })
        .await
    }

    /// Retrieves a webhook given its Id and unique token.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id and its unique token:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let id = WebhookId::new(245037420704169985);
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let webhook = http.get_webhook_with_token(id, token).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_webhook_with_token(
        &self,
        webhook_id: WebhookId,
        token: &str,
    ) -> Result<Webhook> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::WebhookWithToken {
                webhook_id,
                token,
            },
            params: None,
        })
        .await
    }

    /// Retrieves a webhook given its url.
    ///
    /// This method does _not_ require authentication
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by url:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// let url = "https://discord.com/api/webhooks/245037420704169985/ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let webhook = http.get_webhook_from_url(url).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "utils")]
    pub async fn get_webhook_from_url(&self, url: &str) -> Result<Webhook> {
        let url = Url::parse(url).map_err(HttpError::Url)?;
        let (webhook_id, token) =
            crate::utils::parse_webhook(&url).ok_or(HttpError::InvalidWebhook)?;
        self.fire(Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Get,
            route: Route::WebhookWithToken {
                webhook_id,
                token,
            },
            params: None,
        })
        .await
    }

    /// Kicks a member from a guild with a provided reason.
    pub async fn kick_member(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildMember {
                guild_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Leaves a guild.
    pub async fn leave_guild(&self, guild_id: GuildId) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Delete,
            route: Route::UserMeGuild {
                guild_id,
            },
            params: None,
        })
        .await
    }

    /// Sends a message to a channel.
    ///
    /// # Errors
    ///
    /// Returns an [`HttpError::UnsuccessfulRequest`] if the files are too large to send.
    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        files: Vec<CreateAttachment>,
        map: &impl serde::Serialize,
    ) -> Result<Message> {
        let mut request = Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::ChannelMessages {
                channel_id,
            },
            params: None,
        };

        if files.is_empty() {
            request.body = Some(to_vec(map)?);
        } else {
            request.multipart = Some(Multipart {
                upload: MultipartUpload::Attachments(files.into_iter().collect()),
                payload_json: Some(to_string(map)?),
                fields: vec![],
            });
        }

        self.fire(request).await
    }

    /// Pins a message in a channel.
    pub async fn pin_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Put,
            route: Route::ChannelPin {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Unbans a user from a guild.
    pub async fn remove_ban(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildBan {
                guild_id,
                user_id,
            },
            params: None,
        })
        .await
    }

    /// Deletes a single [`Role`] from a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role hierarchy.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    pub async fn remove_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::GuildMemberRole {
                guild_id,
                user_id,
                role_id,
            },
            params: None,
        })
        .await
    }

    /// Returns a list of [`Member`]s in a [`Guild`] whose username or nickname starts with a
    /// provided string.
    pub async fn search_guild_members(
        &self,
        guild_id: GuildId,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<Member>> {
        let mut value: Value = self
            .fire(Request {
                body: None,
                multipart: None,
                headers: None,
                method: LightMethod::Get,
                route: Route::GuildMembersSearch {
                    guild_id,
                },
                params: Some(vec![
                    ("query", query.to_string()),
                    ("limit", limit.unwrap_or(constants::MEMBER_FETCH_LIMIT).to_string()),
                ]),
            })
            .await?;

        if let Some(members) = value.as_array_mut() {
            for member in members {
                if let Some(map) = member.as_object_mut() {
                    map.insert("guild_id".to_string(), guild_id.get().into());
                }
            }
        }

        from_value(value)
    }

    /// Starts removing some members from a guild based on the last time they've been online.
    pub async fn start_guild_prune(
        &self,
        guild_id: GuildId,
        days: u8,
        audit_log_reason: Option<&str>,
    ) -> Result<GuildPrune> {
        self.fire(Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Post,
            route: Route::GuildPrune {
                guild_id,
            },
            params: Some(vec![("days", days.to_string())]),
        })
        .await
    }

    /// Starts syncing an integration with a guild.
    pub async fn start_integration_sync(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: None,
            method: LightMethod::Post,
            route: Route::GuildIntegrationSync {
                guild_id,
                integration_id,
            },
            params: None,
        })
        .await
    }

    /// Starts typing in the specified [`Channel`] for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called on
    /// the returned struct to stop typing. Note that on some clients, typing may persist for a few
    /// seconds after [`Typing::stop`] is called. Typing is also stopped when the struct is
    /// dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief
    /// period of time and then resume again until either [`Typing::stop`] is called or the struct
    /// is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a long-running
    /// command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use serenity::http::{Http, Typing};
    /// # use serenity::Result;
    /// # use serenity::model::prelude::*;
    /// #
    /// # fn long_process() {}
    /// # fn main() {
    /// # let http: Arc<Http> = unimplemented!();
    /// // Initiate typing (assuming http is `Arc<Http>`)
    /// let channel_id = ChannelId::new(7);
    /// let typing = http.start_typing(channel_id);
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// # }
    /// ```
    pub fn start_typing(self: &Arc<Self>, channel_id: ChannelId) -> Typing {
        Typing::start(Arc::clone(self), channel_id)
    }

    /// Unpins a message from a channel.
    pub async fn unpin_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        audit_log_reason: Option<&str>,
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            multipart: None,
            headers: audit_log_reason.map(reason_into_header),
            method: LightMethod::Delete,
            route: Route::ChannelPin {
                channel_id,
                message_id,
            },
            params: None,
        })
        .await
    }

    /// Fires off a request, deserializing the response reader via the given type bound.
    ///
    /// If you don't need to deserialize the response and want the response instance itself, use
    /// [`Self::request`].
    ///
    /// # Examples
    ///
    /// Create a new message and deserialize the response into a [`Message`]:
    ///
    /// ```rust,no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::http::Http;
    /// #
    /// # let http: Http = unimplemented!();
    /// use serenity::{
    ///     http::{LightMethod, Request, Route},
    ///     model::prelude::*,
    /// };
    ///
    /// let bytes = vec![
    ///     // payload bytes here
    /// ];
    /// let channel_id = ChannelId::new(381880193700069377);
    /// let route = Route::ChannelMessages { channel_id };
    ///
    /// let mut request = Request::new(route, LightMethod::Post).body(Some(bytes));
    ///
    /// let message = http.fire::<Message>(request).await?;
    ///
    /// println!("Message content: {}", message.content);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// If there is an error, it will be either [`Error::Http`] or [`Error::Json`].
    pub async fn fire<T: DeserializeOwned>(&self, req: Request<'_>) -> Result<T> {
        let response = self.request(req).await?;
        decode_resp(response).await
    }

    /// Performs a request, ratelimiting it if necessary.
    ///
    /// Returns the raw reqwest Response. Use [`Self::fire`] to deserialize the response into some
    /// type.
    ///
    /// # Examples
    ///
    /// Send a body of bytes over the create message endpoint:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// # use serenity::model::prelude::*;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// use serenity::http::{LightMethod, Request, Route};
    ///
    /// let bytes = vec![
    ///     // payload bytes here
    /// ];
    /// let channel_id = ChannelId::new(381880193700069377);
    /// let route = Route::ChannelMessages { channel_id };
    ///
    /// let mut request = Request::new(route, LightMethod::Post).body(Some(bytes));
    ///
    /// let response = http.request(request).await?;
    ///
    /// println!("Response successful?: {}", response.status().is_success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument]
    pub async fn request(&self, req: Request<'_>) -> Result<ReqwestResponse> {
        let method = req.method.reqwest_method();
        let response = if let Some(ratelimiter) = &self.ratelimiter {
            ratelimiter.perform(req).await?
        } else {
            let request = req.build(&self.client, self.token(), self.proxy.as_deref())?.build()?;
            self.client.execute(request).await?
        };

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::Http(HttpError::UnsuccessfulRequest(
                ErrorResponse::from_response(response, method).await,
            )))
        }
    }

    /// Performs a request and then verifies that the response status code is equal to the expected
    /// value.
    ///
    /// This is a function that performs a light amount of work and returns an empty tuple, so it's
    /// called "self.wind" to denote that it's lightweight.
    pub(super) async fn wind(&self, expected: u16, req: Request<'_>) -> Result<()> {
        let route = req.route;
        let method = req.method.reqwest_method();
        let response = self.request(req).await?;

        if response.status().is_success() {
            let response_status = response.status().as_u16();
            if response_status != expected {
                let route = route.path();
                warn!("Mismatched successful response status from {route}! Expected {expected} but got {response_status}");
            }

            return Ok(());
        }

        debug!("Unsuccessful response: {response:?}");
        Err(Error::Http(HttpError::UnsuccessfulRequest(
            ErrorResponse::from_response(response, method).await,
        )))
    }
}

#[cfg(not(feature = "native_tls_backend"))]
fn configure_client_backend(builder: ClientBuilder) -> ClientBuilder {
    builder.use_rustls_tls()
}

#[cfg(feature = "native_tls_backend")]
fn configure_client_backend(builder: ClientBuilder) -> ClientBuilder {
    builder.use_native_tls()
}

impl AsRef<Http> for Http {
    fn as_ref(&self) -> &Http {
        self
    }
}
