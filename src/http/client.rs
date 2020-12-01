use crate::constants;
use reqwest::{
    multipart::Part,
    Client,
    ClientBuilder,
    Response as ReqwestResponse,
};
use reqwest::{
    header::{AUTHORIZATION, USER_AGENT, CONTENT_TYPE, HeaderValue, HeaderMap as Headers},
    StatusCode,
    Url,
};
use crate::internal::prelude::*;
use crate::model::prelude::*;
use super::{
    ratelimiting::{Ratelimiter, RatelimitedRequest},
    request::Request,
    routing::RouteInfo,
    typing::Typing,
    AttachmentType,
    GuildPagination,
    HttpError,
};
use bytes::buf::Buf;
use serde::de::DeserializeOwned;
use serde_json::json;
use tracing::{debug, trace, instrument};
use std::{
    collections::BTreeMap,
    fmt,
    sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    fs::File,
};
use crate::http::routing::Route;

pub struct Http {
    pub(crate) client: Arc<Client>,
    pub ratelimiter: Ratelimiter,
    pub token: String,
}

impl fmt::Debug for Http {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Http")
            .field("client", &self.client)
            .field("ratelimiter", &self.ratelimiter)
            .finish()
    }
}

impl Http {
    pub fn new(client: Arc<Client>, token: &str) -> Self {
        let client2 = Arc::clone(&client);

        Http {
            client,
            ratelimiter: Ratelimiter::new(client2, token.to_string()),
            token: token.to_string(),
        }
    }

    pub fn new_with_token(token: &str) -> Self {
        let builder = configure_client_backend(Client::builder());
        let built = builder.build().expect("Cannot build reqwest::Client");

        let token = if token.trim().starts_with("Bot ") {
            token.to_string()
        } else {
            format!("Bot {}", token)
        };

        Self::new(Arc::new(built), &token)
    }

    /// Adds a single [`Role`] to a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role
    /// hierarchy.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`Member`]: ../../model/guild/struct.Member.html
    /// [`Role`]: ../../model/guild/struct.Role.html
    /// [Manage Roles]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    pub async fn add_member_role(&self, guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::AddMemberRole { guild_id, role_id, user_id },
        }).await
    }

    /// Bans a [`User`] from a [`Guild`], removing their messages sent in the last
    /// X number of days.
    ///
    /// Passing a `delete_message_days` of `0` is equivalent to not removing any
    /// messages. Up to `7` days' worth of messages may be deleted.
    ///
    /// **Note**: Requires that you have the [Ban Members] permission.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`User`]: ../../model/user/struct.User.html
    /// [Ban Members]: ../../model/permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
    pub async fn ban_user(&self, guild_id: u64, user_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::GuildBanUser {
                delete_message_days: Some(delete_message_days),
                reason: Some(reason),
                guild_id,
                user_id,
            },
        }).await
    }

    /// Broadcasts that the current user is typing in the given [`Channel`].
    ///
    /// This lasts for about 10 seconds, and will then need to be renewed to
    /// indicate that the current user is still typing.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a
    /// long-running command is still being processed.
    ///
    /// [`Channel`]: ../../model/channel/enum.Channel.html
    pub async fn broadcast_typing(&self, channel_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::BroadcastTyping { channel_id },
        }).await
    }

    /// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
    ///
    /// Refer to the Discord's [docs] for information on what fields this requires.
    ///
    /// **Note**: Requires the [Manage Channels] permission.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
    /// [docs]: https://discord.com/developers/docs/resources/guild#create-guild-channel
    /// [Manage Channels]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
    pub async fn create_channel(&self, guild_id: u64, map: &JsonMap) -> Result<GuildChannel> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreateChannel { guild_id },
        }).await
    }

    /// Creates an emoji in the given [`Guild`] with the given data.
    ///
    /// View the source code for [`Guild`]'s [`create_emoji`] method to see what
    /// fields this requires.
    ///
    /// **Note**: Requires the [Manage Emojis] permission.
    ///
    /// [`create_emoji`]: ../../model/guild/struct.Guild.html#method.create_emoji
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [Manage Emojis]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_EMOJIS
    pub async fn create_emoji(&self, guild_id: u64, map: &Value) -> Result<Emoji> {
        self.fire(Request {
            body: Some(map.to_string().as_bytes()),
            headers: None,
            route: RouteInfo::CreateEmoji { guild_id },
        }).await
    }

    /// Creates a guild with the data provided.
    ///
    /// Only a [`PartialGuild`] will be immediately returned, and a full [`Guild`]
    /// will be received over a [`Shard`], if at least one is running.
    ///
    /// **Note**: This endpoint is currently limited to 10 active guilds. The
    /// limits are raised for whitelisted [GameBridge] applications. See the
    /// [documentation on this endpoint] for more info.
    ///
    /// # Examples
    ///
    /// Create a guild called `"test"` in the [US West region]:
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #    let http = Http::default();
    /// let map = json!({
    ///     "name": "test",
    ///     "region": "us-west",
    /// });
    ///
    /// let _result = http.create_guild(&map).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`PartialGuild`]: ../../model/guild/struct.PartialGuild.html
    /// [`Shard`]: ../../gateway/struct.Shard.html
    /// [GameBridge]: https://discord.com/developers/docs/topics/gamebridge
    /// [US West Region]: ../../model/guild/enum.Region.html#variant.UsWest
    /// [documentation on this endpoint]:
    /// https://discord.com/developers/docs/resources/guild#create-guild
    /// [whitelist]: https://discord.com/developers/docs/resources/guild#create-guild
    pub async fn create_guild(&self, map: &Value) -> Result<PartialGuild> {
        self.fire(Request {
            body: Some(map.to_string().as_bytes()),
            headers: None,
            route: RouteInfo::CreateGuild,
        }).await
    }

    /// Creates an [`Integration`] for a [`Guild`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// **Note**: Requires the [Manage Guild] permission.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`Integration`]: ../../model/guild/struct.Integration.html
    /// [Manage Guild]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_GUILD
    /// [docs]: https://discord.com/developers/docs/resources/guild#create-guild-integration
    pub async fn create_guild_integration(&self, guild_id: u64, integration_id: u64, map: &Value) -> Result<()> {
        self.wind(204, Request {
            body: Some(map.to_string().as_bytes()),
            headers: None,
            route: RouteInfo::CreateGuildIntegration { guild_id, integration_id },
        }).await
    }

    /// Creates a [`RichInvite`] for the given [channel][`GuildChannel`].
    ///
    /// Refer to Discord's [docs] for field information.
    ///
    /// All fields are optional.
    ///
    /// **Note**: Requires the [Create Invite] permission.
    ///
    /// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
    /// [`RichInvite`]: ../../model/invite/struct.RichInvite.html
    /// [Create Invite]: ../../model/permissions/struct.Permissions.html#associatedconstant.CREATE_INVITE
    /// [docs]: https://discord.com/developers/docs/resources/channel#create-channel-invite
    pub async fn create_invite(&self, channel_id: u64, map: &JsonMap) -> Result<RichInvite> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreateInvite { channel_id },
        }).await
    }

    /// Creates a permission override for a member or a role in a channel.
    pub async fn create_permission(&self, channel_id: u64, target_id: u64, map: &Value) -> Result<()> {
        let body = serde_json::to_vec(map)?;

        self.wind(204, Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreatePermission { channel_id, target_id },
        }).await
    }

    /// Creates a private channel with a user.
    pub async fn create_private_channel(&self, map: &Value) -> Result<PrivateChannel> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreatePrivateChannel,
        }).await
    }

    /// Reacts to a message.
    pub async fn create_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        reaction_type: &ReactionType
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::CreateReaction {
                reaction: &reaction_type.as_data(),
                channel_id,
                message_id,
            },
        }).await
    }

    /// Creates a role.
    pub async fn create_role(&self, guild_id: u64, map: &JsonMap) -> Result<Role> {
        let body = serde_json::to_vec(map)?;
        let mut value = self.request(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreateRole { guild_id },
        }).await?.json::<Value>().await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
        }

        serde_json::from_value(value).map_err(From::from)
    }

    /// Creates a webhook for the given [channel][`GuildChannel`]'s Id, passing in
    /// the given data.
    ///
    /// This method requires authentication.
    ///
    /// The Value is a map with the values of:
    ///
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default avatar
    ///   (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100 characters
    ///   long.
    ///
    /// # Examples
    ///
    /// Creating a webhook named `test`:
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #    let http = Http::default();
    /// let channel_id = 81384788765712384;
    /// let map = json!({"name": "test"});
    ///
    /// let webhook = http.create_webhook(channel_id, &map).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
    pub async fn create_webhook(&self, channel_id: u64, map: &Value) -> Result<Webhook> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreateWebhook { channel_id },
        }).await
    }

    /// Deletes a private channel or a channel in a guild.
    pub async fn delete_channel(&self, channel_id: u64) -> Result<Channel> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteChannel { channel_id },
        }).await
    }

    /// Deletes an emoji from a server.
    pub async fn delete_emoji(&self, guild_id: u64, emoji_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteEmoji { guild_id, emoji_id },
        }).await
    }

    /// Deletes a guild, only if connected account owns it.
    pub async fn delete_guild(&self, guild_id: u64) -> Result<PartialGuild> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteGuild { guild_id },
        }).await
    }

    /// Removes an integration from a guild.
    pub async fn delete_guild_integration(&self, guild_id: u64, integration_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteGuildIntegration { guild_id, integration_id },
        }).await
    }

    /// Deletes an invite by code.
    pub async fn delete_invite(&self, code: &str) -> Result<Invite> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteInvite { code },
        }).await
    }

    /// Deletes a message if created by us or we have
    /// specific permissions.
    pub async fn delete_message(&self, channel_id: u64, message_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteMessage { channel_id, message_id },
        }).await
    }

    /// Deletes a bunch of messages, only works for bots.
    pub async fn delete_messages(&self, channel_id: u64, map: &Value) -> Result<()> {
        self.wind(204, Request {
            body: Some(map.to_string().as_bytes()),
            headers: None,
            route: RouteInfo::DeleteMessages { channel_id },
        }).await
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
    /// # let http = Http::default();
    /// let channel_id = ChannelId(7);
    /// let message_id = MessageId(8);
    ///
    /// http.delete_message_reactions(channel_id.0, message_id.0).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Message`]: ../../model/channel/struct.Message.html
    /// [`Reaction`]: ../../model/channel/struct.Reaction.html
    pub async fn delete_message_reactions(&self, channel_id: u64, message_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteMessageReactions { channel_id, message_id },
        }).await
    }

    /// Deletes all the reactions for a given emoji on a message.
    pub async fn delete_message_reaction_emoji(
        &self,
        channel_id: u64,
        message_id: u64,
        reaction_type: &ReactionType
    ) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteMessageReactionEmoji {
                reaction: &reaction_type.as_data(),
                channel_id,
                message_id,
            },
        }).await
    }

    /// Deletes a permission override from a role or a member in a channel.
    pub async fn delete_permission(&self, channel_id: u64, target_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeletePermission { channel_id, target_id },
        }).await
    }

    /// Deletes a reaction from a message if owned by us or
    /// we have specific permissions.
    pub async fn delete_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        user_id: Option<u64>,
        reaction_type: &ReactionType
    ) -> Result<()> {
        let user = user_id
            .map(|uid| uid.to_string())
            .unwrap_or_else(|| "@me".to_string());

        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteReaction {
                reaction: &reaction_type.as_data(),
                user: &user,
                channel_id,
                message_id,
            },
        }).await
    }

    /// Deletes a role from a server. Can't remove the default everyone role.
    pub async fn delete_role(&self, guild_id: u64, role_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteRole { guild_id, role_id },
        }).await
    }

    /// Deletes a [`Webhook`] given its Id.
    ///
    /// This method requires authentication, whereas [`delete_webhook_with_token`]
    /// does not.
    ///
    /// # Examples
    ///
    /// Deletes a webhook given its Id:
    ///
    /// ```rust,no_run
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// // Due to the `delete_webhook` function requiring you to authenticate, you
    /// // must have set the token first.
    /// let http = Http::default();
    ///
    /// http.delete_webhook(245037420704169985).await?;
    ///       Ok(())
    /// # }
    /// ```
    ///
    /// [`Webhook`]: ../../model/webhook/struct.Webhook.html
    /// [`delete_webhook_with_token`]: fn.delete_webhook_with_token.html
    pub async fn delete_webhook(&self, webhook_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteWebhook { webhook_id },
        }).await
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
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// http.delete_webhook_with_token(id, token).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Webhook`]: ../../model/webhook/struct.Webhook.html
    pub async fn delete_webhook_with_token(&self, webhook_id: u64, token: &str) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::DeleteWebhookWithToken { token, webhook_id },
        }).await
    }

    /// Changes channel information.
    pub async fn edit_channel(&self, channel_id: u64, map: &JsonMap) -> Result<GuildChannel> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditChannel {channel_id },
        }).await
    }

    /// Changes emoji information.
    pub async fn edit_emoji(&self, guild_id: u64, emoji_id: u64, map: &Value) -> Result<Emoji> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditEmoji { guild_id, emoji_id },
        }).await
    }

    /// Changes guild information.
    pub async fn edit_guild(&self, guild_id: u64, map: &JsonMap) -> Result<PartialGuild> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditGuild { guild_id },
        }).await
    }

    /// Edits the positions of a guild's channels.
    pub async fn edit_guild_channel_positions(
        &self,
        guild_id: u64,
        value: &Value
    ) -> Result<()> {
        let body = serde_json::to_vec(value)?;

        self.wind(204, Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditGuildChannels { guild_id },
        }).await
    }

    /// Edits a [`Guild`]'s embed setting.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    pub async fn edit_guild_embed(&self, guild_id: u64, map: &Value) -> Result<GuildEmbed> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditGuildEmbed { guild_id },
        }).await
    }

    /// Does specific actions to a member.
    pub async fn edit_member(&self, guild_id: u64, user_id: u64, map: &JsonMap) -> Result<()> {
        let body = serde_json::to_vec(map)?;

        self.wind(204, Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditMember { guild_id, user_id },
        }).await
    }

    /// Edits a message by Id.
    ///
    /// **Note**: Only the author of a message can modify it.
    pub async fn edit_message(&self, channel_id: u64, message_id: u64, map: &Value) -> Result<Message> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditMessage { channel_id, message_id },
        }).await
    }

    /// Edits the current user's nickname for the provided [`Guild`] via its Id.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    pub async fn edit_nickname(&self, guild_id: u64, new_nickname: Option<&str>) -> Result<()> {
        let map = json!({ "nick": new_nickname });
        let body = serde_json::to_vec(&map)?;

        self.wind(200, Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditNickname { guild_id },
        }).await
    }

    /// Edits the current user's profile settings.
    pub async fn edit_profile(&self, map: &JsonMap) -> Result<CurrentUser> {
        let body = serde_json::to_vec(map)?;

        let request = self.request(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditProfile,
        }).await?;

        Ok(request.json::<CurrentUser>().await?)
    }

    /// Changes a role in a guild.
    pub async fn edit_role(&self, guild_id: u64, role_id: u64, map: &JsonMap) -> Result<Role> {
        let body = serde_json::to_vec(&map)?;
        let mut value = self.request(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditRole { guild_id, role_id },
        }).await?.json::<Value>().await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
        }

        serde_json::from_value(value).map_err(From::from)
    }

    /// Changes the position of a role in a guild.
    pub async fn edit_role_position(&self, guild_id: u64, role_id: u64, position: u64) -> Result<Vec<Role>> {
        let body = serde_json::to_vec(&json!([{
            "id": role_id,
            "position": position,
        }]))?;

        let mut value = self.request(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditRolePosition { guild_id },
        }).await?.json::<Value>().await?;

        if let Some(array) = value.as_array_mut() {
            for role in array {
                if let Some(map) = role.as_object_mut() {
                    map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                }
            }
        }

        serde_json::from_value(value).map_err(From::from)
    }

    /// Edits a the webhook with the given data.
    ///
    /// The Value is a map with optional values of:
    ///
    /// - **avatar**: base64-encoded 128x128 image for the webhook's default avatar
    ///   (_optional_);
    /// - **name**: the name of the webhook, limited to between 2 and 100 characters
    ///   long.
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
    /// ```rust,no_run
    /// use serde_json::json;
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let id = 245037420704169985;
    /// let image = serenity::utils::read_image("./webhook_img.png")?;
    /// let map = json!({
    ///     "avatar": image,
    /// });
    ///
    /// let edited = http.edit_webhook(id, &map).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`create_webhook`]: fn.create_webhook.html
    /// [`edit_webhook_with_token`]: fn.edit_webhook_with_token.html
    pub async fn edit_webhook(&self, webhook_id: u64, map: &Value) -> Result<Webhook> {
        self.fire(Request {
            body: Some(map.to_string().as_bytes()),
            headers: None,
            route: RouteInfo::EditWebhook { webhook_id },
        }).await
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
    /// ```rust,no_run
    /// use serde_json::json;
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let value = json!({"name": "new name"});
    /// let map = value.as_object().unwrap();
    ///
    /// let edited = http.edit_webhook_with_token(id, token, map).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`edit_webhook`]: fn.edit_webhook.html
    pub async fn edit_webhook_with_token(&self, webhook_id: u64, token: &str, map: &JsonMap) -> Result<Webhook> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::EditWebhookWithToken { token, webhook_id },
        }).await
    }

    /// Executes a webhook, posting a [`Message`] in the webhook's associated
    /// [`Channel`].
    ///
    /// This method does _not_ require authentication.
    ///
    /// Pass `true` to `wait` to wait for server confirmation of the message sending
    /// before receiving a response. From the [Discord docs]:
    ///
    /// > waits for server confirmation of message send before response, and returns
    /// > the created message body (defaults to false; when false a message that is
    /// > not saved does not return an error)
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
    /// **Note**: For embed objects, all fields are registered by Discord except for
    /// `height`, `provider`, `proxy_url`, `type` (it will always be `rich`),
    /// `video`, and `width`. The rest will be determined by Discord.
    ///
    /// # Examples
    ///
    /// Sending a webhook with message content of `test`:
    ///
    /// ```rust,no_run
    /// use serde_json::json;
    /// use serenity::http::Http;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    /// let value = json!({"content": "test"});
    /// let map = value.as_object().unwrap();
    ///
    /// let message = http.execute_webhook(id, token, true, map).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Channel`]: ../../model/channel/enum.Channel.html
    /// [`Message`]: ../../model/channel/struct.Message.html
    /// [Discord docs]: https://discord.com/developers/docs/resources/webhook#querystring-params
    pub async fn execute_webhook(
        &self,
        webhook_id: u64,
        token: &str,
        wait: bool,
        map: &JsonMap
    ) -> Result<Option<Message>> {
        let body = serde_json::to_vec(map)?;

        let mut headers = Headers::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(&"application/json"));

        let response = self.request(Request {
            body: Some(&body),
            headers: Some(headers),
            route: RouteInfo::ExecuteWebhook { token, wait, webhook_id },
        }).await?;

        if response.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        response
            .json::<Message>()
            .await
            .map(Some)
            .map_err(From::from)
    }

    /// Send file(s) over a webhook.
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(ErrorResponse)`][`HttpError::UnsuccessfulRequest`]
    /// if the files are too large to send.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: enum.HttpError.html#variant.UnsuccessfulRequest
    pub async fn execute_webhook_with_files<'a, T, It: IntoIterator<Item=T>>(
        &self,
        webhook_id: u64,
        token: &str,
        wait: bool,
        files: It,
        map: JsonMap
    ) -> Result<Option<Message>>
        where
            T: Into<AttachmentType<'a>>,
    {
        let mut multipart = reqwest::multipart::Form::new();

        for (file_num, file) in files.into_iter().enumerate() {
            match file.into() {
                AttachmentType::Bytes { data, filename } => {
                    multipart = multipart
                        .part(file_num.to_string(), Part::bytes(data.into_owned())
                            .file_name(filename));
                },
                AttachmentType::File { file, filename } => {
                    let mut buf = Vec::new();
                    file.try_clone().await?.read_to_end(&mut buf).await?;

                    multipart = multipart
                        .part(file_num.to_string(),
                              Part::stream(buf)
                                  .file_name(filename));
                },
                AttachmentType::Path(path) => {
                    let filename = path
                        .file_name()
                        .map(|filename| filename.to_string_lossy().into_owned());
                    let mut file = File::open(path).await?;
                    let mut buf = vec![];
                    file.read_to_end(&mut buf).await?;

                    let part = match filename {
                        Some(filename) => Part::bytes(buf).file_name(filename),
                        None => Part::bytes(buf),
                    };

                    multipart = multipart.part(file_num.to_string(), part);
                },
                AttachmentType::Image(url) => {
                    let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;
                    let filename = url.path_segments()
                        .and_then(|segments| segments.last().map(ToString::to_string))
                        .ok_or_else(|| Error::Url(url.to_string()))?;
                    let response = self.client.get(url).send().await?;
                    let mut bytes = response.bytes().await?;
                    let mut picture: Vec<u8> = vec![0; bytes.len()];
                    bytes.copy_to_slice(&mut picture[..]);
                    multipart = multipart
                        .part(file_num.to_string(), Part::bytes(picture)
                            .file_name(filename.to_string()));
                },
            }
        }

        multipart = multipart.text("payload_json", serde_json::to_string(&map)?);

        let response = self.client
            .post(&Route::webhook_with_token_optioned(webhook_id, token, wait))
            .multipart(multipart)
            .header(CONTENT_TYPE, HeaderValue::from_static(&"multipart/form-data"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpError::from_response(response).await.into());
        }

        response
            .json::<Message>()
            .await
            .map(Some)
            .map_err(From::from)
    }

    /// Gets the active maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_active_maintenances(&self) -> Result<Vec<Maintenance>> {
        let response = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetActiveMaintenance,
        }).await?;

        let mut map: BTreeMap<String, Value> = response
            .json::<BTreeMap<String, Value>>()
            .await?;

        match map.remove("scheduled_maintenances") {
            Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
                .map_err(From::from),
            None => Ok(vec![]),
        }
    }

    /// Gets all the users that are banned in specific guild.
    pub async fn get_bans(&self, guild_id: u64) -> Result<Vec<Ban>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetBans { guild_id },
        }).await
    }

    /// Gets all audit logs in a specific guild.
    pub async fn get_audit_logs(
        &self,
        guild_id: u64,
        action_type: Option<u8>,
        user_id: Option<u64>,
        before: Option<u64>,
        limit: Option<u8>
    ) -> Result<AuditLogs> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetAuditLogs {
                action_type,
                before,
                guild_id,
                limit,
                user_id,
            },
        }).await
    }

    /// Gets current bot gateway.
    pub async fn get_bot_gateway(&self) -> Result<BotGateway> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetBotGateway,
        }).await
    }

    /// Gets all invites for a channel.
    pub async fn get_channel_invites(&self, channel_id: u64) -> Result<Vec<RichInvite>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetChannelInvites { channel_id },
        }).await
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
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http = Http::default();
    /// let channel_id = 81384788765712384;
    ///
    /// let webhooks = http.get_channel_webhooks(channel_id).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
    pub async fn get_channel_webhooks(&self, channel_id: u64) -> Result<Vec<Webhook>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetChannelWebhooks { channel_id },
        }).await
    }

    /// Gets channel information.
    pub async fn get_channel(&self, channel_id: u64) -> Result<Channel> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetChannel { channel_id },
        }).await
    }

    /// Gets all channels in a guild.
    pub async fn get_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetChannels { guild_id },
        }).await
    }

    /// Gets information about the current application.
    ///
    /// **Note**: Only applications may use this endpoint.
    pub async fn get_current_application_info(&self) -> Result<CurrentApplicationInfo> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetCurrentApplicationInfo,
        }).await
    }

    /// Gets information about the user we're connected with.
    pub async fn get_current_user(&self) -> Result<CurrentUser> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetCurrentUser,
        }).await
    }

    /// Gets all emojis of a guild.
    pub async fn get_emojis(&self, guild_id: u64) -> Result<Vec<Emoji>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetEmojis { guild_id },
        }).await
    }

    /// Gets information about an emoji in a guild.
    pub async fn get_emoji(&self, guild_id: u64, emoji_id: u64) -> Result<Emoji> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetEmoji { guild_id, emoji_id },
        }).await
    }

    /// Gets current gateway.
    pub async fn get_gateway(&self) -> Result<Gateway> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGateway,
        }).await
    }

    /// Gets guild information.
    pub async fn get_guild(&self, guild_id: u64) -> Result<PartialGuild> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuild { guild_id },
        }).await
    }

    /// Gets a guild embed information.
    pub async fn get_guild_embed(&self, guild_id: u64) -> Result<GuildEmbed> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildEmbed { guild_id },
        }).await
    }

    /// Gets integrations that a guild has.
    pub async fn get_guild_integrations(&self, guild_id: u64) -> Result<Vec<Integration>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildIntegrations { guild_id },
        }).await
    }

    /// Gets all invites to a guild.
    pub async fn get_guild_invites(&self, guild_id: u64) -> Result<Vec<RichInvite>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildInvites { guild_id },
        }).await
    }

    /// Gets a guild's vanity URL if it has one.
    pub async fn get_guild_vanity_url(&self, guild_id: u64) -> Result<String> {
        #[derive(Deserialize)]
        struct GuildVanityUrl {
            code: String,
        }

        self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildVanityUrl { guild_id },
        })
            .await?
        .json::<GuildVanityUrl>()
            .await
            .map(|x| x.code)
            .map_err(From::from)
    }

    /// Gets the members of a guild. Optionally pass a `limit` and the Id of the
    /// user to offset the result by.
    pub async fn get_guild_members(
        &self,
        guild_id: u64,
        limit: Option<u64>,
        after: Option<u64>
    ) -> Result<Vec<Member>> {
        let mut value = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildMembers { after, guild_id, limit },
        }).await?.json::<Value>().await?;

        if let Some(values) = value.as_array_mut() {
            let num = Value::Number(Number::from(guild_id));

            for value in values {
                if let Some(element) = value.as_object_mut() {
                    element.insert("guild_id".to_string(), num.clone());
                }
            }
        }

        serde_json::from_value::<Vec<Member>>(value).map_err(From::from)
    }

    /// Gets the amount of users that can be pruned.
    pub async fn get_guild_prune_count(&self, guild_id: u64, map: &Value) -> Result<GuildPrune> {
        // Note for 0.6.x: turn this into a function parameter.
        #[derive(Deserialize)]
        struct GetGuildPruneCountRequest {
            days: u64,
        }

        let req = serde_json::from_value::<GetGuildPruneCountRequest>(map.clone())?;

        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildPruneCount {
                days: req.days,
                guild_id,
            },
        }).await
    }

    /// Gets regions that a guild can use. If a guild has the `VIP_REGIONS` feature
    /// enabled, then additional VIP-only regions are returned.
    pub async fn get_guild_regions(&self, guild_id: u64) -> Result<Vec<VoiceRegion>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildRegions { guild_id },
        }).await
    }

    /// Retrieves a list of roles in a [`Guild`].
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    pub async fn get_guild_roles(&self, guild_id: u64) -> Result<Vec<Role>> {
        let mut value = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildRoles { guild_id },
        }).await?.json::<Value>().await?;

        if let Some(array) = value.as_array_mut() {
            for role in array {
                if let Some(map) = role.as_object_mut() {
                    map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                }
            }
        }

        serde_json::from_value(value).map_err(From::from)
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
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let guild_id = 81384788765712384;
    ///
    /// let webhooks = http.get_guild_webhooks(guild_id).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    pub async fn get_guild_webhooks(&self, guild_id: u64) -> Result<Vec<Webhook>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuildWebhooks { guild_id },
        }).await
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
    /// #     let http = Http::default();
    /// use serenity::{http::GuildPagination, model::id::GuildId};
    ///
    /// let guild_id = GuildId(81384788765712384);
    ///
    /// let guilds = http.get_guilds(&GuildPagination::After(guild_id), 10).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [docs]: https://discord.com/developers/docs/resources/user#get-current-user-guilds
    pub async fn get_guilds(&self, target: &GuildPagination, limit: u64) -> Result<Vec<GuildInfo>> {
        let (after, before) = match *target {
            GuildPagination::After(id) => (Some(id.0), None),
            GuildPagination::Before(id) => (None, Some(id.0)),
        };

        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetGuilds { after, before, limit },
        }).await
    }

    /// Gets information about a specific invite.
    pub async fn get_invite(&self, mut code: &str, stats: bool) -> Result<Invite> {
        #[cfg(feature = "utils")]
        {
            code = crate::utils::parse_invite(code);
        }

        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetInvite { code, stats },
        }).await
    }

    /// Gets member of a guild.
    pub async fn get_member(&self, guild_id: u64, user_id: u64) -> Result<Member> {
        let mut value = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetMember { guild_id, user_id },
        }).await?.json::<Value>().await?;

        if let Some(map) = value.as_object_mut() {
            map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
        }

        serde_json::from_value::<Member>(value).map_err(From::from)
    }

    /// Gets a message by an Id, bots only.
    pub async fn get_message(&self, channel_id: u64, message_id: u64) -> Result<Message> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetMessage { channel_id, message_id },
        }).await
    }

    /// Gets X messages from a channel.
    pub async fn get_messages(&self, channel_id: u64, query: &str) -> Result<Vec<Message>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetMessages {
                query: query.to_owned(),
                channel_id,
            },
        }).await
    }

    /// Gets all pins of a channel.
    pub async fn get_pins(&self, channel_id: u64) -> Result<Vec<Message>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetPins { channel_id },
        }).await
    }

    /// Gets user Ids based on their reaction to a message. This endpoint is dumb.
    pub async fn get_reaction_users(
        &self,
        channel_id: u64,
        message_id: u64,
        reaction_type: &ReactionType,
        limit: u8,
        after: Option<u64>
    ) -> Result<Vec<User>> {
        let reaction = reaction_type.as_data();

        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetReactionUsers {
                after,
                channel_id,
                limit,
                message_id,
                reaction,
            },
        }).await
    }

    /// Gets the current unresolved incidents from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_unresolved_incidents(&self) -> Result<Vec<Incident>> {
        let response = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetUnresolvedIncidents,
        }).await?;

        let mut map = response.json::<BTreeMap<String, Value>>().await?;

        match map.remove("incidents") {
            Some(v) => serde_json::from_value::<Vec<Incident>>(v)
                .map_err(From::from),
            None => Ok(vec![]),
        }
    }

    /// Gets the upcoming (planned) maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    pub async fn get_upcoming_maintenances(&self) -> Result<Vec<Maintenance>> {
        let response = self.request(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetUpcomingMaintenances,
        }).await?;

        let mut map = response.json::<BTreeMap<String, Value>>().await?;

        match map.remove("scheduled_maintenances") {
            Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
                .map_err(From::from),
            None => Ok(vec![]),
        }
    }

    /// Gets a user by Id.
    pub async fn get_user(&self, user_id: u64) -> Result<User> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetUser { user_id },
        }).await
    }

    /// Gets our DM channels.
    pub async fn get_user_dm_channels(&self) -> Result<Vec<PrivateChannel>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetUserDmChannels,
        }).await
    }

    /// Gets all voice regions.
    pub async fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetVoiceRegions,
        }).await
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
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let id = 245037420704169985;
    /// let webhook = http.get_webhook(id).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`get_webhook_with_token`]: fn.get_webhook_with_token.html
    pub async fn get_webhook(&self, webhook_id: u64) -> Result<Webhook> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetWebhook { webhook_id },
        }).await
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
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// let id = 245037420704169985;
    /// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
    ///
    /// let webhook = http.get_webhook_with_token(id, token).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn get_webhook_with_token(&self, webhook_id: u64, token: &str) -> Result<Webhook> {
        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::GetWebhookWithToken { token, webhook_id },
        }).await
    }

    /// Kicks a member from a guild.
    pub async fn kick_member(&self, guild_id: u64, user_id: u64) -> Result<()> {
        self.kick_member_with_reason(guild_id, user_id, "").await
    }

    /// Kicks a member from a guild with a provided reason.
    pub async fn kick_member_with_reason(&self, guild_id: u64, user_id: u64, reason: &str) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::KickMember {
                guild_id,
                user_id,
                reason,
            },
        }).await
    }

    /// Leaves a guild.
    pub async fn leave_guild(&self, guild_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::LeaveGuild { guild_id },
        }).await
    }

    /// Sends file(s) to a channel.
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(ErrorResponse)`][`HttpError::UnsuccessfulRequest`]
    /// if the files are too large to send.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: enum.HttpError.html#variant.UnsuccessfulRequest
    pub async fn send_files<'a, T, It: IntoIterator<Item=T>>(&self, channel_id: u64, files: It, map: JsonMap) -> Result<Message>
        where T: Into<AttachmentType<'a>> {
        let uri = api!("/channels/{}/messages", channel_id);
        let url = match Url::parse(&uri) {
            Ok(url) => url,
            Err(_) => return Err(Error::Url(uri)),
        };

        let mut multipart = reqwest::multipart::Form::new();

        for (file_num, file) in files.into_iter().enumerate() {
            match file.into() {
                AttachmentType::Bytes{ data, filename } => {
                    multipart = multipart
                        .part(file_num.to_string(), Part::bytes(data.into_owned())
                            .file_name(filename));
                },
                AttachmentType::File{ file, filename } => {
                    let mut buf = Vec::new();
                    file.try_clone().await?.read_to_end(&mut buf).await?;

                    multipart = multipart
                        .part(file_num.to_string(),
                            Part::stream(buf)
                                .file_name(filename));
                },
                AttachmentType::Path(path) => {
                    let filename = path
                        .file_name()
                        .map(|filename| filename.to_string_lossy().into_owned());
                    let mut file = File::open(path).await?;
                    let mut buf = vec![];
                    file.read_to_end(&mut buf).await?;

                    let part = match filename {
                        Some(filename) => Part::bytes(buf).file_name(filename),
                        None => Part::bytes(buf),
                    };

                    multipart = multipart.part(file_num.to_string(), part);
                },
                AttachmentType::Image(url) => {
                    let url = Url::parse(url).map_err(|_| Error::Url(url.to_string()))?;
                    let filename = url.path_segments()
                      .and_then(|segments| segments.last().map(ToString::to_string))
                      .ok_or_else(|| Error::Url(url.to_string()))?;
                    let response = self.client.get(url).send().await?;
                    let mut bytes = response.bytes().await?;
                    let mut picture: Vec<u8> = vec![0; bytes.len()];
                    bytes.copy_to_slice(&mut picture[..]);
                    multipart = multipart
                        .part(file_num.to_string(), Part::bytes(picture)
                            .file_name(filename.to_string()));
                },
            }
        }

        multipart = multipart.text("payload_json", serde_json::to_string(&map)?);

        let response = self.client
            .post(url)
            .header(AUTHORIZATION, HeaderValue::from_str(&self.token)?)
            .header(USER_AGENT, HeaderValue::from_static(&constants::USER_AGENT))
            .multipart(multipart)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(HttpError::from_response(response).await.into());
        }

        response
            .json::<Message>()
            .await
            .map_err(From::from)
    }

    /// Sends a message to a channel.
    pub async fn send_message(&self, channel_id: u64, map: &Value) -> Result<Message> {
        let body = serde_json::to_vec(map)?;

        self.fire(Request {
            body: Some(&body),
            headers: None,
            route: RouteInfo::CreateMessage { channel_id },
        }).await
    }

    /// Pins a message in a channel.
    pub async fn pin_message(&self, channel_id: u64, message_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::PinMessage { channel_id, message_id },
        }).await
    }

    /// Unbans a user from a guild.
    pub async fn remove_ban(&self, guild_id: u64, user_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::RemoveBan { guild_id, user_id },
        }).await
    }

    /// Deletes a single [`Role`] from a [`Member`] in a [`Guild`].
    ///
    /// **Note**: Requires the [Manage Roles] permission and respect of role
    /// hierarchy.
    ///
    /// [`Guild`]: ../../model/guild/struct.Guild.html
    /// [`Member`]: ../../model/guild/struct.Member.html
    /// [`Role`]: ../../model/guild/struct.Role.html
    /// [Manage Roles]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_ROLES
    pub async fn remove_member_role(&self, guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::RemoveMemberRole { guild_id, user_id, role_id },
        }).await
    }

    /// Starts removing some members from a guild based on the last time they've been online.
    pub async fn start_guild_prune(&self, guild_id: u64, map: &Value) -> Result<GuildPrune> {
        // Note for 0.6.x: turn this into a function parameter.
        #[derive(Deserialize)]
        struct StartGuildPruneRequest {
            days: u64,
        }

        let req = serde_json::from_value::<StartGuildPruneRequest>(map.clone())?;

        self.fire(Request {
            body: None,
            headers: None,
            route: RouteInfo::StartGuildPrune {
                days: req.days,
                guild_id,
            },
        }).await
    }

    /// Starts syncing an integration with a guild.
    pub async fn start_integration_sync(&self, guild_id: u64, integration_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::StartIntegrationSync { guild_id, integration_id },
        }).await
    }

    /// Starts typing in the specified [`Channel`] for an indefinite period of time.
    ///
    /// Returns [`Typing`] that is used to trigger the typing. [`Typing::stop`] must be called
    /// on the returned struct to stop typing. Note that on some clients, typing may persist
    /// for a few seconds after `stop` is called.
    /// Typing is also stopped when the struct is dropped.
    ///
    /// If a message is sent while typing is triggered, the user will stop typing for a brief period
    /// of time and then resume again until either `stop` is called or the struct is dropped.
    ///
    /// This should rarely be used for bots, although it is a good indicator that a
    /// long-running command is still being processed.
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// # use serenity::{http::{Http, Typing}, Result};
    /// # use std::sync::Arc;
    /// #
    /// # fn long_process() {}
    /// # fn main() -> Result<()> {
    /// # let http = Arc::new(Http::default());
    /// // Initiate typing (assuming http is `Arc<Http>`)
    /// let typing = http.start_typing(7)?;
    ///
    /// // Run some long-running process
    /// long_process();
    ///
    /// // Stop typing
    /// typing.stop();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Channel`]: ../../model/channel/enum.Channel.html
    /// [`Typing`]: ../typing/struct.Typing.html
    /// [`Typing::stop`]: ../typing/struct.Typing.html#method.stop
    pub fn start_typing(self: &Arc<Self>, channel_id: u64) -> Result<Typing> {
        Typing::start(self.clone(), channel_id)
    }

    /// Unpins a message from a channel.
    pub async fn unpin_message(&self, channel_id: u64, message_id: u64) -> Result<()> {
        self.wind(204, Request {
            body: None,
            headers: None,
            route: RouteInfo::UnpinMessage { channel_id, message_id },
        }).await
    }

    /// Fires off a request, deserializing the response reader via the given type
    /// bound.
    ///
    /// If you don't need to deserialize the response and want the response instance
    /// itself, use [`request`].
    ///
    /// # Examples
    ///
    /// Create a new message via the [`RouteInfo::CreateMessage`] endpoint and
    /// deserialize the response into a [`Message`]:
    ///
    /// ```rust,no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # use serenity::http::Http;
    /// #
    /// # let http = Http::default();
    /// use serenity::{
    ///     http::{
    ///         routing::RouteInfo,
    ///         request::RequestBuilder,
    ///     },
    ///     model::channel::Message,
    /// };
    ///
    /// let bytes = vec![
    ///     // payload bytes here
    /// ];
    /// let channel_id = 381880193700069377;
    /// let route_info = RouteInfo::CreateMessage { channel_id };
    ///
    /// let mut request = RequestBuilder::new(route_info);
    /// request.body(Some(&bytes));
    ///
    /// let message = http.fire::<Message>(request.build()).await?;
    ///
    /// println!("Message content: {}", message.content);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`request`]: fn.request.html
    pub async fn fire<T: DeserializeOwned>(&self, req: Request<'_>) -> Result<T> {
        let response = self.request(req).await?;

        response
            .json::<T>()
            .await
            .map_err(From::from)
    }

    /// Performs a request, ratelimiting it if necessary.
    ///
    /// Returns the raw reqwest Response. Use [`fire`] to deserialize the response
    /// into some type.
    ///
    /// # Examples
    ///
    /// Send a body of bytes over the [`RouteInfo::CreateMessage`] endpoint:
    ///
    /// ```rust,no_run
    /// # use serenity::http::Http;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Http::default();
    /// use serenity::http::{
    ///     request::RequestBuilder,
    ///     routing::RouteInfo,
    /// };
    ///
    /// let bytes = vec![
    ///     // payload bytes here
    /// ];
    /// let channel_id = 381880193700069377;
    /// let route_info = RouteInfo::CreateMessage { channel_id };
    ///
    /// let mut request = RequestBuilder::new(route_info);
    /// request.body(Some(&bytes));
    ///
    /// let response = http.request(request.build()).await?;
    ///
    /// println!("Response successful?: {}", response.status().is_success());
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`fire`]: fn.fire.html
    #[instrument]
    pub async fn request(&self, req: Request<'_>) -> Result<ReqwestResponse> {
        let ratelimiting_req = RatelimitedRequest::from(req);
        let response = self
            .ratelimiter
            .perform(ratelimiting_req)
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Error::Http(Box::new(HttpError::from_response(response).await)))
        }
    }

    /// Performs a request and then verifies that the response status code is equal
    /// to the expected value.
    ///
    /// This is a function that performs a light amount of work and returns an
    /// empty tuple, so it's called "self.wind" to denote that it's lightweight.
    pub(super) async fn wind(&self, expected: u16, req: Request<'_>) -> Result<()> {
        let response = self.request(req).await?;

        if response.status().as_u16() == expected {
            return Ok(());
        }

        debug!("Expected {}, got {}", expected, response.status());
        trace!("Unsuccessful response: {:?}", response);

        Err(Error::Http(Box::new(HttpError::from_response(response).await)))
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
    fn as_ref(&self) -> &Http { &self }
}

impl Default for Http {
    fn default() -> Self {
        let built = Client::builder().build().expect("Cannot build Reqwest::Client.");
        let client = Arc::new(built);
        let client2 = Arc::clone(&client);

        Self {
            client,
            ratelimiter: Ratelimiter::new(client2, ""),
            token: "".to_string(),
        }
    }
}
