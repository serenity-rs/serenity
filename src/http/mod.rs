//! The HTTP module which provides functions for performing requests to
//! endpoints in Discord's API.
//!
//! An important function of the REST API is ratelimiting. Requests to endpoints
//! are ratelimited to prevent spam, and once ratelimited Discord will stop
//! performing requests. The library implements protection to pre-emptively
//! ratelimit, to ensure that no wasted requests are made.
//!
//! The HTTP module comprises of two types of requests:
//!
//! - REST API requests, which require an authorization token;
//! - Other requests, which do not require an authorization token.
//!
//! The former require a [`Client`] to have logged in, while the latter may be
//! made regardless of any other usage of the library.
//!
//! If a request spuriously fails, it will be retried once.
//!
//! Note that you may want to perform requests through a [model]s'
//! instance methods where possible, as they each offer different
//! levels of a high-level interface to the HTTP module.
//!
//! [`Client`]: ../struct.Client.html
//! [model]: ../model/index.html

#[macro_use] mod macros;

pub mod constants;
pub mod ratelimiting;

mod error;
mod routing;
mod utils;

pub use hyper::StatusCode;
pub use self::error::{Error as HttpError, Result};
pub use self::routing::{Path, Route};

use futures::{Future, Stream, future};
use http_crate::uri::Uri;
use hyper::client::{Client as HyperClient, HttpConnector};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, CONTENT_LENGTH};
use hyper::{Body, Method, Request, Response};
use hyper::body::Payload;
use hyper_tls::HttpsConnector;
use model::prelude::*;
use parking_lot::Mutex;
use self::ratelimiting::RateLimiter;
use serde::de::DeserializeOwned;
use serde_json::{self, Number, Value};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;
use ::builder::*;
use ::{Error, utils as serenity_utils};

/// An method used for ratelimiting special routes.
///
/// This is needed because `hyper`'s `Method` enum does not derive Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LightMethod {
    /// Indicates that a route is for the `DELETE` method only.
    Delete,
    /// Indicates that a route is for the `GET` method only.
    Get,
    /// Indicates that a route is for the `PATCH` method only.
    Patch,
    /// Indicates that a route is for the `POST` method only.
    Post,
    /// Indicates that a route is for the `PUT` method only.
    Put,
}

impl LightMethod {
    pub fn hyper_method(&self) -> Method {
        match *self {
            LightMethod::Delete => Method::DELETE,
            LightMethod::Get => Method::GET,
            LightMethod::Patch => Method::PATCH,
            LightMethod::Post => Method::POST,
            LightMethod::Put => Method::PUT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    pub base: String,
    pub client: Arc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
    pub ratelimiter: Option<Arc<Mutex<RateLimiter>>>,
    pub token: Arc<String>,
}

impl Client {
    pub fn new(
        client: Arc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
        token: Arc<String>,
    ) -> Result<Self> {
        Ok(Self {
            base: self::constants::API_URI_VERSIONED.to_owned(),
            ratelimiter: Some(Arc::new(Mutex::new(RateLimiter::new()))),
            client,
            token,
        })
    }

    pub fn skip_ratelimiter(&mut self) {
        self.ratelimiter = None;
    }

    pub fn set_token(&mut self, token: Arc<String>) {
        self.token = token;
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self._set_base_url(base_url.into())
    }

    fn _set_base_url(&mut self, base_url: String) {
        self.base = base_url;
    }

    /// Adds a [`User`] as a recipient to a [`Group`].
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    ///
    /// [`Group`]: ../model/channel/struct.Group.html
    /// [`Group::add_recipient`]: ../model/channel/struct.Group.html#method.add_recipient
    /// [`User`]: ../model/user/struct.User.html
    pub fn add_group_recipient(&self, group_id: ChannelId, user_id: UserId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::AddGroupRecipient {
            group_id: group_id.0,
            user_id: user_id.0,
        }, None)
    }

    /// Adds a single [`Role`] to a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role
    /// hierarchy.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`Member`]: ../model/guild/struct.Member.html
    /// [`Role`]: ../model/guild/struct.Role.html
    /// [Manage Roles]: ../model/permissions/constant.MANAGE_ROLES.html
    pub fn add_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::AddMemberRole {
            guild_id: guild_id.0,
            user_id: user_id.0,
            role_id: role_id.0,
        }, None)
    }

    /// Bans a [`User`] from a [`Guild`], removing their messages sent in the
    /// last X number of days.
    ///
    /// Passing a `delete_message_days` of `0` is equivalent to not removing any
    /// messages. Up to `7` days' worth of messages may be deleted.
    ///
    /// **Note**: Requires that you have the [Ban Members] permission.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`User`]: ../model/user/struct.User.html
    /// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
    pub fn ban_user(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        delete_message_days: u8,
        reason: &str,
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::GuildBanUser {
            delete_message_days: Some(delete_message_days),
            reason: Some(reason),
            guild_id: guild_id.0,
            user_id: user_id.0,
        }, None)
    }

    /// Broadcasts that the current user is typing in the given [`Channel`].
    ///
    /// This lasts for about 10 seconds, and will then need to be renewed to
    /// indicate that the current user is still typing.
    ///
    /// This should rarely be used for bots, although it is a good indicator
    /// that a long-running command is still being processed.
    ///
    /// [`Channel`]: ../model/channel/enum.Channel.html
    pub fn broadcast_typing(&self, channel_id: ChannelId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::BroadcastTyping {
            channel_id: channel_id.0,
        }, None)
    }

    /// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
    ///
    /// Refer to the Discord's [docs] for information on what fields this
    /// requires.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
    /// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-channel
    /// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
    pub fn create_channel(
        &self,
        guild_id: GuildId,
        name: &str,
        kind: ChannelType,
        category_id: Option<ChannelId>,
    ) -> impl Future<Item = GuildChannel, Error = Error> + Send {
        self.post(Route::CreateChannel {
            guild_id: guild_id.0,
        }, Some(&json!({
            "name": name,
            "parent_id": category_id.map(|id| id.0),
            "type": kind as u8,
        })))
    }

    /// Creates an emoji in the given [`Guild`] with the given data.
    ///
    /// View the source code for [`Context::create_emoji`] to see what fields
    /// this requires.
    ///
    /// **Note**: Requires the [Manage Emojis] permission.
    ///
    /// [`Context::create_emoji`]: ../struct.Context.html#method.create_emoji
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [Manage Emojis]: ../model/permissions/constant.MANAGE_EMOJIS.html
    pub fn create_emoji(&self, guild_id: GuildId, name: &str, image: &str)
        -> impl Future<Item = Emoji, Error = Error> + Send {
        self.post(Route::CreateEmoji {
            guild_id: guild_id.0,
        }, Some(&json!({
            "image": image,
            "name": name,
        })))
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full
    /// [`Guild`] will be received over a [`Shard`], if at least one is running.
    ///
    /// **Note**: This endpoint is currently limited to 10 active guilds. The
    /// limits are raised for whitelisted [GameBridge] applications. See the
    /// [documentation on this endpoint] for more info.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region]:
    ///
    /// ```rust,ignore
    /// extern crate serde_json;
    ///
    /// use serde_json::builder::ObjectBuilder;
    /// use serde_json::Value;
    /// use serenity::http;
    ///
    /// let map = ObjectBuilder::new()
    ///     .insert("name", "test")
    ///     .insert("region", "us-west")
    ///     .build();
    ///
    /// let _result = http::create_guild(map);
    /// ```
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`PartialGuild`]: ../model/guild/struct.PartialGuild.html
    /// [`Shard`]: ../gateway/struct.Shard.html
    /// [GameBridge]: https://discordapp.com/developers/docs/topics/gamebridge
    /// [US West Region]: ../model/enum.Region.html#variant.UsWest
    /// [documentation on this endpoint]:
    /// https://discordapp.com/developers/docs/resources/guild#create-guild
    /// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
    pub fn create_guild(&self, map: &Value)
        -> impl Future<Item = PartialGuild, Error = Error> + Send {
        self.post(Route::CreateGuild, Some(map))
    }

    /// Creates an [`Integration`] for a [`Guild`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`Integration`]: ../model/guild/struct.Integration.html
    /// [Manage Guild]: ../model/permissions/constant.MANAGE_GUILD.html
    /// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-integration
    pub fn create_guild_integration(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
        kind: &str,
    ) -> impl Future<Item = (), Error = Error> + Send {
        let json = json!({
            "id": integration_id.0,
            "type": kind,
        });
        self.verify(Route::CreateGuildIntegration {
            guild_id: guild_id.0,
        }, Some(&json))
    }

    /// Creates a [`RichInvite`] for the given [channel][`GuildChannel`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// All fields are optional.
    ///
    /// **Note**: Requires the [Create Invite] permission.
    ///
    /// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
    /// [`RichInvite`]: ../model/guild/struct.RichInvite.html
    /// [Create Invite]: ../model/permissions/constant.CREATE_INVITE.html
    /// [docs]: https://discordapp.com/developers/docs/resources/channel#create-channel-invite
    pub fn create_invite<F>(&self, channel_id: ChannelId, f: F)
        -> impl Future<Item = RichInvite, Error = Error>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        let map = serenity_utils::vecmap_to_json_map(f(CreateInvite::default()).0);

        self.post(Route::CreateInvite {
            channel_id: channel_id.0
        }, Some(&Value::Object(map)))
    }

    /// Creates a permission override for a member or a role in a channel.
    pub fn create_permission(
        &self,
        channel_id: ChannelId,
        target: &PermissionOverwrite,
    ) -> impl Future<Item = (), Error = Error> + Send {
        let (target_id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };
        let map = json!({
            "allow": target.allow.bits(),
            "deny": target.deny.bits(),
            "type": kind,
        });

        self.verify(Route::CreatePermission {
            target_id: target_id,
            channel_id: channel_id.0,
        }, Some(&map))
    }

    /// Creates a private channel with a user.
    pub fn create_private_channel(&self, user_id: UserId)
        -> impl Future<Item = PrivateChannel, Error = Error> + Send {
        let map = json!({
            "recipient_id": user_id.0,
        });

        self.post(Route::CreatePrivateChannel, Some(&map))
    }

    /// Reacts to a message.
    pub fn create_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::CreateReaction {
            reaction: &utils::reaction_type_data(reaction_type),
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    /// Creates a role.
    pub fn create_role<F>(&self, guild_id: GuildId, f: F) -> impl Future<Item = Role, Error = Error>
        where F: FnOnce(EditRole) -> EditRole {
        let map = serenity_utils::vecmap_to_json_map(f(EditRole::default()).0);

        self.post(Route::CreateRole {
            guild_id: guild_id.0,
        }, Some(&Value::Object(map)))
    }

    /// Creates a webhook for the given [channel][`GuildChannel`]'s Id, passing
    /// in the given data.
    ///
    /// This method requires authentication.
    ///
    /// The Value is a map with the values of:
    ///
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default
    ///   avatar (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100
    ///   characters long.
    ///
    /// # Examples
    ///
    /// Creating a webhook named `test`:
    ///
    /// ```rust,ignore
    /// extern crate serde_json;
    /// extern crate serenity;
    ///
    /// use serde_json::builder::ObjectBuilder;
    /// use serenity::http;
    ///
    /// let channel_id = 81384788765712384;
    /// let map = ObjectBuilder::new().insert("name", "test").build();
    ///
    /// let webhook = http::create_webhook(channel_id, map)
    ///     .expect("Error creating");
    /// ```
    ///
    /// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
    pub fn create_webhook(&self, channel_id: ChannelId, map: &Value)
        -> impl Future<Item = Webhook, Error = Error> + Send {
        self.post(Route::CreateWebhook { channel_id: channel_id.0 }, Some(map))
    }

    /// Deletes a private channel or a channel in a guild.
    pub fn delete_channel(&self, channel_id: ChannelId) -> impl Future<Item = Channel, Error = Error> + Send {
        self.delete(Route::DeleteChannel { channel_id: channel_id.0 }, None)
    }

    /// Deletes an emoji from a server.
    pub fn delete_emoji(&self, guild_id: GuildId, emoji_id: EmojiId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.delete(Route::DeleteEmoji {
            guild_id: guild_id.0,
            emoji_id: emoji_id.0,
        }, None)
    }

    /// Deletes a guild, only if connected account owns it.
    pub fn delete_guild(&self, guild_id: GuildId)
        -> impl Future<Item = PartialGuild, Error = Error> + Send {
        self.delete(Route::DeleteGuild { guild_id: guild_id.0 }, None)
    }

    /// Remvoes an integration from a guild.
    pub fn delete_guild_integration(
        &self,
        guild_id: GuildId,
        integration_id: IntegrationId,
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteGuildIntegration {
            guild_id: guild_id.0,
            integration_id: integration_id.0,
        }, None)
    }

    /// Deletes an invite by code.
    pub fn delete_invite(&self, code: &str) -> impl Future<Item = Invite, Error = Error> + Send {
        self.delete(Route::DeleteInvite { code }, None)
    }

    /// Deletes a message if created by us or we have specific permissions.
    pub fn delete_message(&self, channel_id: ChannelId, message_id: MessageId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteMessage {
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    /// Deletes a bunch of messages, only works for bots.
    pub fn delete_messages<T, It>(&self, channel_id: ChannelId, message_ids: It)
        -> impl Future<Item = (), Error = Error>
        where T: AsRef<MessageId>, It: IntoIterator<Item=T> {
        let ids = message_ids
            .into_iter()
            .map(|id| id.as_ref().0)
            .collect::<Vec<u64>>();

        let map = json!({
            "messages": ids,
        });

        self.verify(Route::DeleteMessages { channel_id: channel_id.0 }, Some(&map))
    }

    /// Deletes all of the [`Reaction`]s associated with a [`Message`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serenity::http;
    /// use serenity::model::{ChannelId, MessageId};
    ///
    /// let channel_id = ChannelId(7);
    /// let message_id = MessageId(8);
    ///
    /// let _ = http::delete_message_reactions(channel_id.0, message_id.0)
    ///     .expect("Error deleting reactions");
    /// ```
    ///
    /// [`Message`]: ../model/channel/struct.Message.html
    /// [`Reaction`]: ../model/channel/struct.Reaction.html
    pub fn delete_message_reactions(&self, channel_id: ChannelId, message_id: MessageId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteMessageReactions {
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    /// Deletes a permission override from a role or a member in a channel.
    pub fn delete_permission(&self, channel_id: ChannelId, target_id: u64)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeletePermission { channel_id: channel_id.0, target_id }, None)
    }

    /// Deletes a reaction from a message if owned by us or
    /// we have specific permissions.
    pub fn delete_reaction(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        user_id: Option<UserId>,
        reaction_type: &ReactionType,
    ) -> impl Future<Item = (), Error = Error> + Send {
        let reaction_type = utils::reaction_type_data(reaction_type);
        let user = user_id
            .map(|uid| uid.to_string())
            .unwrap_or_else(|| "@me".to_string());

        self.verify(Route::DeleteReaction {
            reaction: &reaction_type,
            user: &user,
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    /// Deletes a role from a server. Can't remove the default everyone role.
    pub fn delete_role(&self, guild_id: GuildId, role_id: RoleId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteRole { guild_id: guild_id.0, role_id: role_id.0 }, None)
    }

    /// Deletes a [`Webhook`] given its Id.
    ///
    /// This method requires authentication, whereas
    /// [`delete_webhook_with_token`] does not.
    ///
    /// # Examples
    ///
    /// Deletes a webhook given its Id:
    ///
    /// ```rust,no_run
    /// use serenity::{Client, http};
    /// use std::env;
    ///
    /// // Due to the `delete_webhook` function requiring you to authenticate,
    /// // you must have set the token first.
    /// http::set_token(&env::var("DISCORD_TOKEN").unwrap());
    ///
    /// http::delete_webhook(245037420704169985)
    ///     .expect("Error deleting webhook");
    /// ```
    ///
    /// [`Webhook`]: ../model/webhook/struct.Webhook.html
    /// [`delete_webhook_with_token`]: fn.delete_webhook_with_token.html
    pub fn delete_webhook(&self, webhook_id: WebhookId) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteWebhook { webhook_id: webhook_id.0 }, None)
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
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// http::delete_webhook_with_token(id, token)
    ///     .expect("Error deleting webhook");
    /// ```
    ///
    /// [`Webhook`]: ../model/webhook/struct.Webhook.html
    pub fn delete_webhook_with_token(&self, webhook_id: WebhookId, token: &str)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteWebhookWithToken { webhook_id: webhook_id.0, token }, None)
    }

    /// Changes channel information.
    pub fn edit_channel<F>(&self, channel_id: ChannelId, f: F)
        -> impl Future<Item = GuildChannel, Error = Error>
        where F: FnOnce(EditChannel) -> EditChannel {
        let channel = f(EditChannel::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(channel));

        self.patch(Route::EditChannel { channel_id: channel_id.0 }, Some(&map))
    }

    /// Changes emoji information.
    pub fn edit_emoji(&self, guild_id: GuildId, emoji_id: EmojiId, name: &str)
        -> impl Future<Item = Emoji, Error = Error> + Send {
        let map = json!({
            "name": name,
        });

        self.patch(Route::EditEmoji {
            guild_id: guild_id.0,
            emoji_id: emoji_id.0,
        }, Some(&map))
    }

    /// Changes guild information.
    pub fn edit_guild<F>(&self, guild_id: GuildId, f: F)
        -> impl Future<Item = PartialGuild, Error = Error>
        where F: FnOnce(EditGuild) -> EditGuild {
        let guild = f(EditGuild::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(guild));

        self.patch(Route::EditGuild { guild_id: guild_id.0 }, Some(&map))
    }

    /// Edits the positions of a guild's channels.
    pub fn edit_guild_channel_positions<It>(&self, guild_id: GuildId, channels: It)
        -> impl Future<Item = (), Error = Error> + Send where It: IntoIterator<Item = (ChannelId, u64)> {
        let items = channels.into_iter().map(|(id, pos)| json!({
            "id": id,
            "position": pos,
        })).collect();

        let map = Value::Array(items);

        self.patch(Route::EditGuildChannels { guild_id: guild_id.0 }, Some(&map))
    }

    /// Edits a [`Guild`]'s embed setting.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    // todo
    pub fn edit_guild_embed(&self, guild_id: GuildId, map: &Value)
        -> impl Future<Item = GuildEmbed, Error = Error> + Send {
        self.patch(Route::EditGuildEmbed { guild_id: guild_id.0 }, Some(map))
    }

    /// Does specific actions to a member.
    pub fn edit_member<F>(&self, guild_id: GuildId, user_id: UserId, f: F)
        -> impl Future<Item = (), Error = Error> + Send where F: FnOnce(EditMember) -> EditMember {
        let member = f(EditMember::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(member));

        self.verify(Route::EditMember {
            guild_id: guild_id.0,
            user_id: user_id.0,
        }, Some(&map))
    }

    /// Edits a message by Id.
    ///
    /// **Note**: Only the author of a message can modify it.
    pub fn edit_message<F: FnOnce(EditMessage) -> EditMessage>(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        f: F,
    ) -> impl Future<Item = Message, Error = Error> + Send {
        let msg = f(EditMessage::default());
        let map = Value::Object(serenity_utils::vecmap_to_json_map(msg.0));

        self.patch(Route::EditMessage {
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, Some(&map))
    }

    /// Edits the current user's nickname for the provided [`Guild`] via its Id.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub fn edit_nickname(&self, guild_id: GuildId, new_nickname: Option<&str>)
        -> impl Future<Item = (), Error = Error> + Send {
        self.patch(Route::EditNickname { guild_id: guild_id.0 }, Some(&json!({
            "nick": new_nickname,
        })))
    }

    /// Edits the current user's profile settings.
    pub fn edit_profile(&self, map: &Value) -> impl Future<Item = CurrentUser, Error = Error> + Send {
        self.patch(Route::EditProfile, Some(map))
    }

    /// Changes a role in a guild.
    pub fn edit_role<F>(&self, guild_id: GuildId, role_id: RoleId, f: F)
        -> impl Future<Item = Role, Error = Error> + Send where F: FnOnce(EditRole) -> EditRole {
        let role = f(EditRole::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(role));

        self.patch(Route::EditRole {
            guild_id: guild_id.0,
            role_id: role_id.0,
        }, Some(&map))
    }

    /// Changes the position of a role in a guild.
    pub fn edit_role_position(
        &self,
        guild_id: GuildId,
        role_id: RoleId,
        position: u64,
    ) -> impl Future<Item = Vec<Role>, Error = Error> + Send {
        self.patch(Route::EditRolePosition { guild_id: guild_id.0 }, Some(&json!({
            "id": role_id.0,
            "position": position,
        })))
    }

    /// Edits a the webhook with the given data.
    ///
    /// The Value is a map with optional values of:
    ///
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default
    ///   avatar (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100
    ///   characters long.
    ///
    /// Note that, unlike with [`create_webhook`], _all_ values are optional.
    ///
    /// This method requires authentication, whereas [`edit_webhook_with_token`]
    /// does not.
    ///
    /// # Examples
    ///
    /// Edit the image of a webhook given its Id and unique token:
    ///
    /// ```rust,ignore
    /// extern crate serde_json;
    /// extern crate serenity;
    ///
    /// use serde_json::builder::ObjectBuilder;
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let image = serenity::utils::read_image("./webhook_img.png")
    ///     .expect("Error reading image");
    /// let map = ObjectBuilder::new().insert("avatar", image).build();
    ///
    /// let edited = http::edit_webhook_with_token(id, token, map)
    ///     .expect("Error editing webhook");
    /// ```
    ///
    /// [`create_webhook`]: fn.create_webhook.html
    /// [`edit_webhook_with_token`]: fn.edit_webhook_with_token.html
    // The tests are ignored, rather than no_run'd, due to rustdoc tests with
    // external crates being incredibly messy and misleading in the end user's
    // view.
    pub fn edit_webhook(
        &self,
        webhook_id: WebhookId,
        name: Option<&str>,
        avatar: Option<&str>,
    ) -> impl Future<Item = Webhook, Error = Error> + Send {
        let map = json!({
            "avatar": avatar,
            "name": name,
        });

        self.patch(Route::EditWebhook { webhook_id: webhook_id.0 }, Some(&map))
    }

    /// Edits the webhook with the given data.
    ///
    /// Refer to the documentation for [`edit_webhook`] for more information.
    ///
    /// This method does _not_ require authentication.
    ///
    /// # Examples
    ///
    /// Edit the name of a webhook given its Id and unique token:
    ///
    /// ```rust,ignore
    /// extern crate serde_json;
    /// extern crate serenity;
    ///
    /// use serde_json::builder::ObjectBuilder;
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let map = ObjectBuilder::new().insert("name", "new name").build();
    ///
    /// let edited = http::edit_webhook_with_token(id, token, map)
    ///     .expect("Error editing webhook");
    /// ```
    ///
    /// [`edit_webhook`]: fn.edit_webhook.html
    pub fn edit_webhook_with_token(
        &self,
        webhook_id: WebhookId,
        token: &str,
        name: Option<&str>,
        avatar: Option<&str>,
    ) -> impl Future<Item = Webhook, Error = Error> + Send {
        let map = json!({
            "avatar": avatar,
            "name": name,
        });
        let route = Route::EditWebhookWithToken {
            token,
            webhook_id: webhook_id.0,
        };

        self.patch(route, Some(&map))
    }

    /// Executes a webhook, posting a [`Message`] in the webhook's associated
    /// [`Channel`].
    ///
    /// This method does _not_ require authentication.
    ///
    /// Pass `true` to `wait` to wait for server confirmation of the message
    /// sending before receiving a response. From the [Discord docs]:
    ///
    /// > waits for server confirmation of message send before response, and
    /// > returns the created message body (defaults to false; when false a
    /// > message that is not saved does not return an error)
    ///
    /// The map can _optionally_ contain the following data:
    ///
    /// - `avatar_url`: Override the default avatar of the webhook with a URL.
    /// - `tts`: Whether this is a text-to-speech message (defaults to `false`).
    /// - `username`: Override the default username of the webhook.
    ///
    /// Additionally, _at least one_ of the following must be given:
    ///
    /// - `content`: The content of the message.
    /// - `embeds`: An array of rich embeds.
    ///
    /// **Note**: For embed objects, all fields are registered by Discord except
    /// for `height`, `provider`, `proxy_url`, `type` (it will always be
    /// `rich`), `video`, and `width`. The rest will be determined by Discord.
    ///
    /// # Examples
    ///
    /// Sending a webhook with message content of `test`:
    ///
    /// ```rust,ignore
    /// extern crate serde_json;
    /// extern crate serenity;
    ///
    /// use serde_json::builder::ObjectBuilder;
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let map = ObjectBuilder::new().insert("content", "test").build();
    ///
    /// let message = match http::execute_webhook(id, token, true, map) {
    ///     Ok(Some(message)) => message,
    ///     Ok(None) => {
    ///         println!("Expected a webhook message");
    ///
    ///         return;
    ///     },
    ///     Err(why) => {
    ///         println!("Error executing webhook: {:?}", why);
    ///
    ///         return;
    ///     },
    /// };
    /// ```
    ///
    /// [`Channel`]: ../model/channel/enum.Channel.html
    /// [`Message`]: ../model/channel/struct.Message.html
    /// [Discord docs]: https://discordapp.com/developers/docs/resources/webhook#querystring-params
    pub fn execute_webhook<F: FnOnce(ExecuteWebhook) -> ExecuteWebhook>(
        &self,
        webhook_id: WebhookId,
        token: &str,
        wait: bool,
        f: F,
    ) -> impl Future<Item = Option<Message>, Error = Error> + Send {
        let execution = f(ExecuteWebhook::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(execution));

        let route = Route::ExecuteWebhook {
            token,
            wait,
            webhook_id: webhook_id.0,
        };

        if wait {
            future::Either::A(self.post(route, Some(&map)))
        } else {
            future::Either::B(self.verify(route, Some(&map)).map(|_| None))
        }
    }

    /// Gets the active maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    // pub fn get_active_maintenances() -> impl Future<Item = Vec<Maintenance>, Error = Error> + Send {
    //     let client = request_client!();

    //     let response = retry(|| {
    //         client.get(status!("/scheduled-maintenances/active.json"))
    //     })?;

    //     let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    //     match map.remove("scheduled_maintenances") {
    //         Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
    //             .map_err(From::from),
    //         None => Ok(vec![]),
    //     }
    // }

    /// Gets all the users that are banned in specific guild.
    pub fn get_bans(&self, guild_id: GuildId) -> impl Future<Item = Vec<Ban>, Error = Error> + Send {
        self.get(Route::GetBans { guild_id: guild_id.0 })
    }

    /// Gets all audit logs in a specific guild.
    pub fn get_audit_logs(
        &self,
        guild_id: GuildId,
        action_type: Option<Action>,
        user_id: Option<UserId>,
        before: Option<AuditLogEntryId>,
        limit: Option<u8>,
    ) -> impl Future<Item = AuditLogs, Error = Error> + Send {
        self.get(Route::GetAuditLogs {
            guild_id: guild_id.0,
            action_type: action_type.map(|action| action.num()),
            user_id: user_id.map(|id| id.0),
            before: before.map(|id| id.0),
            limit,
        })
    }

    /// Gets current bot gateway.
    pub fn get_bot_gateway(&self) -> impl Future<Item = BotGateway, Error = Error> + Send {
        self.get(Route::GetBotGateway)
    }

    /// Gets all invites for a channel.
    pub fn get_channel_invites(&self, channel_id: ChannelId)
        -> impl Future<Item = Vec<RichInvite>, Error = Error> + Send {
        self.get(Route::GetChannelInvites { channel_id: channel_id.0 })
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
    /// use serenity::http;
    ///
    /// let channel_id = 81384788765712384;
    ///
    /// let webhooks = http::get_channel_webhooks(channel_id)
    ///     .expect("Error getting channel webhooks");
    /// ```
    ///
    /// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
    pub fn get_channel_webhooks(&self, channel_id: ChannelId)
        -> impl Future<Item = Vec<Webhook>, Error = Error> + Send {
        self.get(Route::GetChannelWebhooks { channel_id: channel_id.0 })
    }

    /// Gets channel information.
    pub fn get_channel(&self, channel_id: ChannelId) -> impl Future<Item = Channel, Error = Error> + Send {
        self.get(Route::GetChannel { channel_id: channel_id.0 })
    }

    /// Gets all channels in a guild.
    pub fn get_channels(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<GuildChannel>, Error = Error> + Send {
        self.get(Route::GetChannels { guild_id: guild_id.0 })
    }

    /// Gets information about the current application.
    ///
    /// **Note**: Only applications may use this endpoint.
    pub fn get_current_application_info(&self)
        -> impl Future<Item = CurrentApplicationInfo, Error = Error> + Send {
        self.get(Route::GetCurrentApplicationInfo)
    }

    /// Gets information about the user we're connected with.
    pub fn get_current_user(&self) -> impl Future<Item = CurrentUser, Error = Error> + Send {
        self.get(Route::GetCurrentUser)
    }

    /// Gets current gateway.
    pub fn get_gateway(&self) -> impl Future<Item = Gateway, Error = Error> + Send {
        self.get(Route::GetGateway)
    }

    /// Gets guild information.
    pub fn get_guild(&self, guild_id: GuildId) -> impl Future<Item = PartialGuild, Error = Error> + Send {
        self.get(Route::GetGuild { guild_id: guild_id.0 })
    }

    /// Gets a guild embed information.
    pub fn get_guild_embed(&self, guild_id: GuildId)
        -> impl Future<Item = GuildEmbed, Error = Error> + Send {
        self.get(Route::GetGuildEmbed { guild_id: guild_id.0 })
    }

    /// Gets integrations that a guild has.
    pub fn get_guild_integrations(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<Integration>, Error = Error> + Send {
        self.get(Route::GetGuildIntegrations { guild_id: guild_id.0 })
    }

    /// Gets all invites to a guild.
    pub fn get_guild_invites(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<RichInvite>, Error = Error> + Send {
        self.get(Route::GetGuildInvites { guild_id: guild_id.0 })
    }

    /// Gets the members of a guild. Optionally pass a `limit` and the Id of the
    /// user to offset the result by.
    pub fn get_guild_members(
        &self,
        guild_id: GuildId,
        limit: Option<u64>,
        after: Option<u64>
    ) -> impl Future<Item = Vec<Member>, Error = Error> + Send {
        self.get(Route::GetGuildMembers { after, guild_id: guild_id.0, limit })
            .and_then(move |mut v: Value| {
                if let Some(values) = v.as_array_mut() {
                    let num = Value::Number(Number::from(guild_id.0));

                    for value in values {
                        if let Some(element) = value.as_object_mut() {
                            element.insert("guild_id".to_string(), num.clone());
                        }
                    }
                }

                serde_json::from_value::<Vec<Member>>(v).map_err(From::from)
            })
    }

    /// Gets the amount of users that can be pruned.
    pub fn get_guild_prune_count(&self, guild_id: GuildId, days: u16)
        -> impl Future<Item = GuildPrune, Error = Error> + Send {
        self.get(Route::GetGuildPruneCount {
            days: days as u64,
            guild_id: guild_id.0,
        })
    }

    /// Gets regions that a guild can use. If a guild has
    /// [`Feature::VipRegions`] enabled, then additional VIP-only regions are
    /// returned.
    ///
    /// [`Feature::VipRegions`]: ../model/enum.Feature.html#variant.VipRegions
    pub fn get_guild_regions(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<VoiceRegion>, Error = Error> + Send {
        self.get(Route::GetGuildRegions { guild_id: guild_id.0 })
    }

    /// Retrieves a list of roles in a [`Guild`].
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub fn get_guild_roles(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<Role>, Error = Error> + Send {
        self.get(Route::GetGuildRoles { guild_id: guild_id.0 })
    }

    /// Gets a guild's vanity URL if it has one.
    pub fn get_guild_vanity_url(&self, guild_id: GuildId)
        -> impl Future<Item = String, Error = Error> + Send {
        #[derive(Deserialize)]
        struct GuildVanityUrl {
            code: String,
        }

        self.get::<GuildVanityUrl>(
            Route::GetGuildVanityUrl { guild_id: guild_id.0 },
        ).map(|resp| {
            resp.code
        })
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
    /// use serenity::http;
    ///
    /// let guild_id = 81384788765712384;
    ///
    /// let webhooks = http::get_guild_webhooks(guild_id)
    ///     .expect("Error getting guild webhooks");
    /// ```
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub fn get_guild_webhooks(&self, guild_id: GuildId)
        -> impl Future<Item = Vec<Webhook>, Error = Error> + Send {
        self.get(Route::GetGuildWebhooks { guild_id: guild_id.0 })
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
    /// use serenity::http::{GuildPagination, get_guilds};
    /// use serenity::model::GuildId;
    ///
    /// let guild_id = GuildId(81384788765712384);
    ///
    /// let guilds = get_guilds(&GuildPagination::After(guild_id), 10).unwrap();
    /// ```
    ///
    /// [docs]: https://discordapp.com/developers/docs/resources/user#get-current-user-guilds
    pub fn get_guilds(&self, target: &GuildPagination, limit: u64)
        -> impl Future<Item = Vec<GuildInfo>, Error = Error> + Send {
        let (after, before) = match *target {
            GuildPagination::After(v) => (Some(v.0), None),
            GuildPagination::Before(v) => (None, Some(v.0)),
        };

        self.get(Route::GetGuilds { after, before, limit })
    }

    /// Gets information about a specific invite.
    pub fn get_invite<'a>(&self, code: &'a str, stats: bool)
        -> impl Future<Item = Invite, Error = Error> + Send {
        self.get(Route::GetInvite { code, stats })
    }

    /// Gets member of a guild.
    pub fn get_member(&self, guild_id: GuildId, user_id: UserId)
        -> impl Future<Item = Member, Error = Error> + Send {
        self.get::<Value>(Route::GetMember {
            guild_id: guild_id.0,
            user_id: user_id.0,
        })
            .and_then(move |mut v| {
                if let Some(map) = v.as_object_mut() {
                    map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id.0)));
                }

                serde_json::from_value(v).map_err(From::from)
            })
    }

    /// Gets a message by an Id, bots only.
    pub fn get_message(&self, channel_id: ChannelId, message_id: MessageId)
        -> impl Future<Item = Message, Error = Error> + Send {
        self.get(Route::GetMessage {
            channel_id: channel_id.0,
            message_id: message_id.0,
        })
    }

    /// Gets X messages from a channel.
    pub fn get_messages<F: FnOnce(GetMessages) -> GetMessages>(
        &self,
        channel_id: ChannelId,
        f: F,
    ) -> impl Future<Item = Vec<Message>, Error = Error> + Send {
        let builder = f(GetMessages::default());
        let mut query = format!("?limit={}", builder.limit);

        use crate::builder::get_messages::MessagePagination;
        if let Some(pagination) = builder.message_pagination {
            match pagination {
                MessagePagination::After(message) => write!(query, "&after={}", message),
                MessagePagination::Around(message) => write!(query, "&around={}", message),
                MessagePagination::Before(message) => write!(query, "&before={}", message),
            }.unwrap() // query is a String, so this won't fail
        }

        self.get(Route::GetMessages {
            channel_id: channel_id.0,
            query,
        })
    }

    /// Gets all pins of a channel.
    pub fn get_pins(&self, channel_id: ChannelId) -> impl Future<Item = Vec<Message>, Error = Error> + Send {
        self.get(Route::GetPins { channel_id: channel_id.0 })
    }

    /// Gets user Ids based on their reaction to a message.
    pub fn get_reaction_users(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>
    ) -> impl Future<Item = Vec<User>, Error = Error> + Send {
        let reaction = utils::reaction_type_data(reaction_type);
        self.get(Route::GetReactionUsers {
            limit: limit.unwrap_or(50),
            after: after.map(|id| id.0),
            channel_id: channel_id.0,
            message_id: message_id.0,
            reaction,
        })
    }

    /// Gets the current unresolved incidents from Discord's Status API.
    ///
    /// Does not require authentication.
    pub fn get_unresolved_incidents(&self) -> impl Future<Item = Vec<Incident>, Error = Error> + Send {
        self.get::<HashMap<String, Value>>(Route::GetUnresolvedIncidents)
            .and_then(|mut map| {
                match map.remove("incidents") {
                    Some(v) => serde_json::from_value(v).map_err(From::from),
                    None => Ok(vec![]),
                }
            })
    }

    /// Gets the upcoming (planned) maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    pub fn get_upcoming_maintenances(&self) -> impl Future<Item = Vec<Maintenance>, Error = Error> + Send {
        self.get::<HashMap<String, Value>>(Route::StatusMaintenancesUpcoming)
            .and_then(|mut map| {
                match map.remove("scheduled_maintenances") {
                    Some(v) => serde_json::from_value(v).map_err(From::from),
                    None => Ok(vec![]),
                }
            })
    }

    /// Gets a user by Id.
    pub fn get_user(&self, user_id: UserId) -> impl Future<Item = User, Error = Error> + Send {
        self.get(Route::GetUser { user_id: user_id.0 })
    }

    /// Gets our DM channels.
    pub fn get_user_dm_channels(&self)
        -> impl Future<Item = Vec<PrivateChannel>, Error = Error> + Send
    {
        self.get(Route::GetUserDmChannels)
    }

    /// Gets all voice regions.
    pub fn get_voice_regions(&self) -> impl Future<Item = Vec<VoiceRegion>, Error = Error> + Send {
        self.get(Route::GetVoiceRegions)
    }

    /// Retrieves a webhook given its Id.
    ///
    /// This method requires authentication, whereas [`get_webhook_with_token`] does
    /// not.
    ///
    /// # Examples
    ///
    /// Retrieve a webhook by Id:
    ///
    /// ```rust,no_run
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let webhook = http::get_webhook(id).expect("Error getting webhook");
    /// ```
    ///
    /// [`get_webhook_with_token`]: fn.get_webhook_with_token.html
    pub fn get_webhook(&self, webhook_id: WebhookId)
        -> impl Future<Item = Webhook, Error = Error> + Send
    {
        self.get(Route::GetWebhook { webhook_id: webhook_id.0 })
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
    /// use serenity::http;
    ///
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let webhook = http::get_webhook_with_token(id, token)
    ///     .expect("Error getting webhook");
    /// ```
    pub fn get_webhook_with_token<'a>(
        &self,
        webhook_id: WebhookId,
        token: &'a str,
    ) -> impl Future<Item = Webhook, Error = Error> + Send {
        self.get(Route::GetWebhookWithToken {
            token,
            webhook_id: webhook_id.0,
        })
    }

    /// Kicks a member from a guild.
    pub fn kick_member(&self, guild_id: GuildId, user_id: UserId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::KickMember {
            guild_id: guild_id.0,
            user_id: user_id.0,
        }, None)
    }

    /// Leaves a group DM.
    pub fn leave_group(&self, group_id: ChannelId) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::DeleteChannel {
            channel_id: group_id.0,
        }, None)
    }

    /// Leaves a guild.
    pub fn leave_guild(&self, guild_id: GuildId) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::LeaveGuild {
            guild_id: guild_id.0,
        }, None)
    }

    /// Deletes a user from group DM.
    pub fn remove_group_recipient(&self, group_id: ChannelId, user_id: UserId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::RemoveGroupRecipient {
            group_id: group_id.0,
            user_id: user_id.0,
        }, None)
    }

    /*
    /// Sends file(s) to a channel.
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    /// if the file is too large to send.
    ///
    /// [`HttpError::InvalidRequest`]: enum.HttpError.html#variant.InvalidRequest
    pub fn send_files<F, T, It>(
        &self,
        channel_id: ChannelId,
        files: It,
        f: F,
    ) -> Box<Future<Item = Message, Error = Error>>
        where F: FnOnce(CreateMessage) -> CreateMessage,
              T: Into<AttachmentType>,
              It: IntoIterator<Item = T> {
        let msg = f(CreateMessage::default());
        let map = serenity_utils::vecmap_to_json_map(msg.0);

        let uri = try_uri!(Path::channel_messages(channel_id).as_ref());
        let mut form = Form::default();
        let mut file_num = "0".to_string();

        for file in files {
            match file.into() {
                AttachmentType::Bytes((mut bytes, filename)) => {
                    form.add_reader_file(
                        file_num.to_owned(),
                        Cursor::new(bytes),
                        filename,
                    );
                },
                AttachmentType::File((mut f, filename)) => {
                    form.add_reader_file(
                        file_num.to_owned(),
                        f,
                        filename,
                    );
                },
            }

            unsafe {
                let vec = file_num.as_mut_vec();
                vec[0] += 1;
            }
        }

        for (k, v) in map.into_iter() {
            match v {
                Value::Bool(false) => form.add_text(k, "false"),
                Value::Bool(true) => form.add_text(k, "true"),
                Value::Number(inner) => form.add_text(k, inner.to_string()),
                Value::String(inner) => form.add_text(k, inner),
                Value::Object(inner) => form.add_text(k, ftry!(serde_json::to_string(&inner))),
                _ => continue,
            }
        }

        let mut request = ftry!(Request::get(uri).body(()));
        form.set_body(&mut request);

        let client = Arc::clone(&self.multiparter);

        let done = client.request(request)
            .from_err()
            .and_then(verify_status)
            .and_then(|res| res.body().concat2().map_err(From::from))
            .and_then(|body| serde_json::from_slice(&body).map_err(From::from));

        Box::new(done)
    }
    */

    /// Sends a message to a channel.
    pub fn send_message<'s, F>(&'s self, channel_id: ChannelId, f: F)
        -> impl Future<Item = Message, Error = Error> + 's
    where
        F: FnOnce(CreateMessage) -> CreateMessage
    {
        let msg = f(CreateMessage::default());
        let map = Value::Object(serenity_utils::vecmap_to_json_map(msg.data));
        let reactions = msg.reactions;

        self.post(Route::CreateMessage {
            channel_id: channel_id.0
        }, Some(&map)).and_then(move |msg: Message| {
            if let Some(reactions) = reactions {
                let msg_id = msg.id;
                future::Either::A(futures::stream::iter_ok(reactions).for_each(move |reaction| {
                    self.create_reaction(channel_id, msg_id, &reaction)
                }).map(|()| msg))
            } else {
                future::Either::B(futures::future::ok(msg))
            }
        })
    }

    /// Pins a message in a channel.
    pub fn pin_message(&self, channel_id: ChannelId, message_id: MessageId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::PinMessage {
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    /// Unbans a user from a guild.
    pub fn remove_ban(&self, guild_id: GuildId, user_id: UserId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::RemoveBan {
            guild_id: guild_id.0,
            user_id: user_id.0,
        }, None)
    }

    /// Deletes a single [`Role`] from a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role
    /// hierarchy.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [`Member`]: ../model/guild/struct.Member.html
    /// [`Role`]: ../model/guild/struct.Role.html
    /// [Manage Roles]: ../model/permissions/constant.MANAGE_ROLES.html
    pub fn remove_member_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.verify(
            Route::RemoveMemberRole {
                guild_id: guild_id.0,
                role_id: role_id.0,
                user_id: user_id.0,
            },
            None,
        )
    }

    /// Starts removing some members from a guild based on the last time they've been online.
    pub fn start_guild_prune(&self, guild_id: GuildId, days: u16)
        -> impl Future<Item = GuildPrune, Error = Error> + Send {
        self.post(Route::StartGuildPrune {
            days: days as u64,
            guild_id: guild_id.0,
        }, None)
    }

    /// Starts syncing an integration with a guild.
    pub fn start_integration_sync(&self, guild_id: GuildId, integration_id: IntegrationId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(
            Route::StartIntegrationSync {
                guild_id: guild_id.0,
                integration_id: integration_id.0,
            },
            None,
        )
    }

    /// Unpins a message from a channel.
    pub fn unpin_message(&self, channel_id: ChannelId, message_id: MessageId)
        -> impl Future<Item = (), Error = Error> + Send {
        self.verify(Route::UnpinMessage {
            channel_id: channel_id.0,
            message_id: message_id.0,
        }, None)
    }

    fn delete<'a, T: DeserializeOwned + 'static + Send>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = T, Error = Error> + Send {
        self.request(route, map)
    }

    fn get<'a, T: DeserializeOwned + 'static + Send>(&self, route: Route<'a>)
        -> impl Future<Item = T, Error = Error> + Send {
        self.request(route, None)
    }

    fn patch<'a, T: DeserializeOwned + 'static + Send>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = T, Error = Error> + Send {
        self.request(route, map)
    }

    fn post<'a, T: DeserializeOwned + 'static + Send>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = T, Error = Error> + Send {
        self.request(route, map)
    }

    fn request<'a, T: DeserializeOwned + 'static + Send>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = T, Error = Error> + Send {
        self.request_common(route, map)
            .and_then(|res| res.into_body().concat2().map_err(From::from))
            .and_then(|body| serde_json::from_slice(&body).map_err(From::from))
    }

    fn verify<'a>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = (), Error = Error> + Send {
        self.request_common(route, map).map(|_| ())
    }

    fn request_common<'a>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> impl Future<Item = Response<Body>, Error = Error> + Send {
        let (method, path, url) = route.deconstruct();

        let uri = match Uri::from_str(&format!("{}/{}", self.base, url.as_ref())) {
            Ok(uri) => uri,
            Err(why) => return future::Either::A(future::err(Error::Http(HttpError::InvalidUri(why)))),
        };

        let body: Body = match map {
            Some(value) => match serde_json::to_vec(value) {
                Ok(body) => body,
                Err(why) => return future::Either::A(future::err(Error::Json(why))),
            },
            None => vec![],
        }.into();

        let request = match Request::builder()
            .method(method.hyper_method())
            .header(AUTHORIZATION, &self.token()[..])
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_LENGTH, body.content_length().unwrap())
            .uri(uri)
            .body(body) {
            Ok(request) => request,
            Err(why) => return future::Either::A(future::err(Error::HttpCrate(why))),
        };

        let client = Arc::clone(&self.client);

        let ratelimit = match self.ratelimiter.as_ref() {
            Some(ratelimiter) => future::Either::A(ratelimiter.lock().take(&path)),
            None => future::Either::B(future::ok(())),
        };

        future::Either::B(ratelimit
            .and_then(move |_| client.request(request).map_err(From::from))
            .from_err()
            .and_then(verify_status))
    }

    fn token(&self) -> String {
        let pointer = Arc::into_raw(Arc::clone(&self.token));
        let token = unsafe {
            (*pointer).clone()
        };

        unsafe {
            drop(Arc::from_raw(pointer));
        }

        token
    }
}


/// Verifies the status of the response according to the method used to create
/// the accompanying request.
///
/// If the status code is correct (a 200 is expected and the response status is
/// also 200), then the future resolves. Otherwise, a leaf future is returned
/// with an error as the `Error` type.
///
/// # Errors
///
/// Returns [`Error::InvalidRequest`] if the response status code is unexpected.
///
/// [`Error::InvalidRequest`]: enum.Error.html#variant.InvalidRequest
fn verify_status(response: Response<Body>) ->
    impl Future<Item = Response<Body>, Error = Error> + Send {
    if response.status().is_success() {
        future::ok(response)
    } else {
        let (parts, _) = response.into_parts();
        let resp = Response::from_parts(parts, ());
        future::err(Error::Http(HttpError::InvalidRequest(resp)))
    }
}

/// Enum that allows a user to pass a `Path` or a `File` type to `send_files`.
#[derive(Debug)]
pub enum AttachmentType {
    /// Indicates that the `AttachmentType` is a byte slice with a filename.
    Bytes((Vec<u8>, String)),
    /// Indicates that the `AttachmentType` is a `File`
    File((File, String)),
}

impl From<(Vec<u8>, String)> for AttachmentType {
    fn from(params: (Vec<u8>, String)) -> AttachmentType {
        AttachmentType::Bytes(params)
    }
}

impl From<(File, String)> for AttachmentType {
    fn from(f: (File, String)) -> AttachmentType {
        AttachmentType::File((f.0, f.1))
    }
}

/// Representation of the method of a query to send for the [`get_guilds`]
/// function.
///
/// [`get_guilds`]: fn.get_guilds.html
pub enum GuildPagination {
    /// The Id to get the guilds after.
    After(GuildId),
    /// The Id to get the guilds before.
    Before(GuildId),
}
