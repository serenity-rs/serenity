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

pub mod ratelimiting;

mod error;
mod routing;
mod utils;

pub use hyper::StatusCode;
pub use self::error::{Error as HttpError, Result};
pub use self::routing::{Path, Route};

use futures::{Future, Stream, future};
use hyper::{
    client::{Client as HyperClient, HttpConnector},
    header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Body,
    Error as HyperError,
    Method,
    Request, 
    Response,
};
use hyper_multipart_rfc7578::client::multipart::{Body as MultipartBody, Form};
use hyper_tls::HttpsConnector;
use mime::APPLICATION_JSON;
use model::prelude::*;
use self::ratelimiting::RateLimiter;
use serde::de::DeserializeOwned;
use serde_json::{self, Number, Value};
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::{Debug, Write};
use std::fs::File;
use std::io::Cursor;
use std::rc::Rc;
use std::str::FromStr;
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
    pub client: Rc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
    pub multiparter: Rc<HyperClient<HttpsConnector<HttpConnector>, MultipartBody>>,
    pub ratelimiter: Rc<RefCell<RateLimiter>>,
    pub token: Rc<String>,
}

impl Client {
    pub fn new(
        client: Rc<HyperClient<HttpsConnector<HttpConnector>, Body>>,
        token: Rc<String>,
    ) -> Result<Self> {
        let connector = HttpsConnector::new(4)?;

        let multiparter = Rc::new(HyperClient::builder()
            .keep_alive(true)
            .build::<_, MultipartBody>(connector));

        Ok(Self {
            ratelimiter: Rc::new(RefCell::new(RateLimiter::new())),
            client,
            multiparter,
            token,
        })
    }

    pub fn set_token(&mut self, token: Rc<String>) {
        self.token = token;
    }

    /// Adds a [`User`] as a recipient to a [`Group`].
    ///
    /// **Note**: Groups have a limit of 10 recipients, including the current
    /// user.
    ///
    /// [`Group`]: ../model/channel/struct.Group.html
    /// [`Group::add_recipient`]: ../model/channel/struct.Group.html#method.add_recipient
    /// [`User`]: ../model/user/struct.User.html
    pub fn add_group_recipient(&self, group_id: u64, user_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::AddGroupRecipient { group_id, user_id }, None)
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
        guild_id: u64,
        user_id: u64,
        role_id: u64
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::AddMemberRole { guild_id, user_id, role_id }, None)
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
        guild_id: u64,
        user_id: u64,
        delete_message_days: u8,
        reason: &str,
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::GuildBanUser {
            delete_message_days: Some(delete_message_days),
            reason: Some(reason),
            guild_id,
            user_id,
        }, None)
    }

    /// Ban zeyla from a [`Guild`], removing her messages sent in the last X number
    /// of days.
    ///
    /// Passing a `delete_message_days` of `0` is equivalent to not removing any
    /// messages. Up to `7` days' worth of messages may be deleted.
    ///
    /// **Note**: Requires that you have the [Ban Members] permission.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    /// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
    pub fn ban_zeyla(
        &self,
        guild_id: u64,
        delete_message_days: u8,
        reason: &str,
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::GuildBanUser {
            delete_message_days: Some(delete_message_days),
            reason: Some(reason),
            guild_id,
            user_id: 114941315417899012,
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
    pub fn broadcast_typing(&self, channel_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::BroadcastTyping { channel_id }, None)
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
        guild_id: u64,
        name: &str,
        kind: ChannelType,
        category_id: Option<u64>,
    ) -> impl Future<Item = GuildChannel, Error = Error> {
        self.post(Route::CreateChannel { guild_id }, Some(&json!({
            "name": name,
            "parent_id": category_id,
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
    pub fn create_emoji(&self, guild_id: u64, name: &str, image: &str)
        -> impl Future<Item = Emoji, Error = Error> {
        self.post(Route::CreateEmoji { guild_id }, Some(&json!({
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
        -> impl Future<Item = PartialGuild, Error = Error> {
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
        guild_id: u64,
        integration_id: u64,
        kind: &str,
    ) -> impl Future<Item = (), Error = Error> {
        let json = json!({
            "id": integration_id,
            "type": kind,
        });
        self.verify(Route::CreateGuildIntegration {
            guild_id,
            integration_id,
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
    pub fn create_invite<F>(&self, channel_id: u64, f: F)
        -> impl Future<Item = RichInvite, Error = Error>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        let map = serenity_utils::vecmap_to_json_map(f(CreateInvite::default()).0);

        self.post(Route::CreateInvite { channel_id }, Some(&Value::Object(map)))
    }

    /// Creates a permission override for a member or a role in a channel.
    pub fn create_permission(
        &self,
        channel_id: u64,
        target: &PermissionOverwrite,
    ) -> impl Future<Item = (), Error = Error> {
        let (id, kind) = match target.kind {
            PermissionOverwriteType::Member(id) => (id.0, "member"),
            PermissionOverwriteType::Role(id) => (id.0, "role"),
        };
        let map = json!({
            "allow": target.allow.bits(),
            "deny": target.deny.bits(),
            "id": id,
            "type": kind,
        });

        self.verify(Route::CreatePermission {
            target_id: id,
            channel_id,
        }, Some(&map))
    }

    /// Creates a private channel with a user.
    pub fn create_private_channel(&self, user_id: u64)
        -> impl Future<Item = PrivateChannel, Error = Error> {
        let map = json!({
            "recipient_id": user_id,
        });

        self.post(Route::CreatePrivateChannel, Some(&map))
    }

    /// Reacts to a message.
    pub fn create_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        reaction_type: &ReactionType
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::CreateReaction {
            reaction: &utils::reaction_type_data(reaction_type),
            channel_id,
            message_id,
        }, None)
    }

    /// Creates a role.
    pub fn create_role<F>(&self, guild_id: u64, f: F) -> impl Future<Item = Role, Error = Error>
        where F: FnOnce(EditRole) -> EditRole {
        let map = serenity_utils::vecmap_to_json_map(f(EditRole::default()).0);

        self.post(Route::CreateRole { guild_id }, Some(&Value::Object(map)))
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
    pub fn create_webhook(&self, channel_id: u64, map: &Value)
        -> impl Future<Item = Webhook, Error = Error> {
        self.post(Route::CreateWebhook { channel_id }, Some(map))
    }

    /// Deletes a private channel or a channel in a guild.
    pub fn delete_channel(&self, channel_id: u64) -> impl Future<Item = Channel, Error = Error> {
        self.delete(Route::DeleteChannel { channel_id }, None)
    }

    /// Deletes an emoji from a server.
    pub fn delete_emoji(&self, guild_id: u64, emoji_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.delete(Route::DeleteEmoji { guild_id, emoji_id }, None)
    }

    /// Deletes a guild, only if connected account owns it.
    pub fn delete_guild(&self, guild_id: u64)
        -> impl Future<Item = PartialGuild, Error = Error> {
        self.delete(Route::DeleteGuild { guild_id }, None)
    }

    /// Remvoes an integration from a guild.
    pub fn delete_guild_integration(
        &self,
        guild_id: u64,
        integration_id: u64,
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteGuildIntegration {
            guild_id,
            integration_id,
        }, None)
    }

    /// Deletes an invite by code.
    pub fn delete_invite(&self, code: &str) -> impl Future<Item = Invite, Error = Error> {
        self.delete(Route::DeleteInvite { code }, None)
    }

    /// Deletes a message if created by us or we have specific permissions.
    pub fn delete_message(&self, channel_id: u64, message_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteMessage { channel_id, message_id }, None)
    }

    /// Deletes a bunch of messages, only works for bots.
    pub fn delete_messages<T, It>(&self, channel_id: u64, message_ids: It)
        -> impl Future<Item = (), Error = Error>
        where T: AsRef<MessageId>, It: IntoIterator<Item=T> {
        let ids = message_ids
            .into_iter()
            .map(|id| id.as_ref().0)
            .collect::<Vec<u64>>();

        let map = json!({
            "messages": ids,
        });

        self.verify(Route::DeleteMessages { channel_id }, Some(&map))
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
    pub fn delete_message_reactions(&self, channel_id: u64, message_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteMessageReactions {
            channel_id,
            message_id,
        }, None)
    }

    /// Deletes a permission override from a role or a member in a channel.
    pub fn delete_permission(&self, channel_id: u64, target_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeletePermission { channel_id, target_id }, None)
    }

    /// Deletes a reaction from a message if owned by us or
    /// we have specific permissions.
    pub fn delete_reaction(
        &self,
        channel_id: u64,
        message_id: u64,
        user_id: Option<u64>,
        reaction_type: &ReactionType,
    ) -> impl Future<Item = (), Error = Error> {
        let reaction_type = utils::reaction_type_data(reaction_type);
        let user = user_id
            .map(|uid| uid.to_string())
            .unwrap_or_else(|| "@me".to_string());

        self.verify(Route::DeleteReaction {
            reaction: &reaction_type,
            user: &user,
            channel_id,
            message_id,
        }, None)
    }

    /// Deletes a role from a server. Can't remove the default everyone role.
    pub fn delete_role(&self, guild_id: u64, role_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteRole { guild_id, role_id }, None)
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
    pub fn delete_webhook(&self, webhook_id: u64) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteWebhook { webhook_id }, None)
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
    pub fn delete_webhook_with_token(&self, webhook_id: u64, token: &str)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteWebhookWithToken { webhook_id, token }, None)
    }

    /// Changes channel information.
    pub fn edit_channel<F>(&self, channel_id: u64, f: F)
        -> impl Future<Item = GuildChannel, Error = Error>
        where F: FnOnce(EditChannel) -> EditChannel {
        let channel = f(EditChannel::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(channel));

        self.patch(Route::EditChannel { channel_id }, Some(&map))
    }

    /// Changes emoji information.
    pub fn edit_emoji(&self, guild_id: u64, emoji_id: u64, name: &str)
        -> impl Future<Item = Emoji, Error = Error> {
        let map = json!({
            "name": name,
        });

        self.patch(Route::EditEmoji { guild_id, emoji_id }, Some(&map))
    }

    /// Changes guild information.
    pub fn edit_guild<F>(&self, guild_id: u64, f: F)
        -> impl Future<Item = PartialGuild, Error = Error>
        where F: FnOnce(EditGuild) -> EditGuild {
        let guild = f(EditGuild::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(guild));

        self.patch(Route::EditGuild { guild_id }, Some(&map))
    }

    /// Edits the positions of a guild's channels.
    pub fn edit_guild_channel_positions<It>(&self, guild_id: u64, channels: It)
        -> impl Future<Item = (), Error = Error> where It: IntoIterator<Item = (ChannelId, u64)> {
        let items = channels.into_iter().map(|(id, pos)| json!({
            "id": id,
            "position": pos,
        })).collect();

        let map = Value::Array(items);

        self.patch(Route::EditGuildChannels { guild_id }, Some(&map))
    }

    /// Edits a [`Guild`]'s embed setting.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    // todo
    pub fn edit_guild_embed(&self, guild_id: u64, map: &Value)
        -> impl Future<Item = GuildEmbed, Error = Error> {
        self.patch(Route::EditGuildEmbed { guild_id }, Some(map))
    }

    /// Does specific actions to a member.
    pub fn edit_member<F>(&self, guild_id: u64, user_id: u64, f: F)
        -> impl Future<Item = (), Error = Error> where F: FnOnce(EditMember) -> EditMember {
        let member = f(EditMember::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(member));

        self.verify(Route::EditMember { guild_id, user_id }, Some(&map))
    }

    /// Edits a message by Id.
    ///
    /// **Note**: Only the author of a message can modify it.
    pub fn edit_message<F: FnOnce(EditMessage) -> EditMessage>(
        &self,
        channel_id: u64,
        message_id: u64,
        f: F,
    ) -> impl Future<Item = Message, Error = Error> {
        let msg = f(EditMessage::default());
        let map = Value::Object(serenity_utils::vecmap_to_json_map(msg.0));

        self.patch(Route::EditMessage { channel_id, message_id }, Some(&map))
    }

    /// Edits the current user's nickname for the provided [`Guild`] via its Id.
    ///
    /// Pass `None` to reset the nickname.
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub fn edit_nickname(&self, guild_id: u64, new_nickname: Option<&str>)
        -> impl Future<Item = (), Error = Error> {
        self.patch(Route::EditNickname { guild_id }, Some(&json!({
            "nick": new_nickname,
        })))
    }

    /// Edits the current user's profile settings.
    pub fn edit_profile(&self, map: &Value) -> impl Future<Item = CurrentUser, Error = Error> {
        self.patch(Route::EditProfile, Some(map))
    }

    /// Changes a role in a guild.
    pub fn edit_role<F>(&self, guild_id: u64, role_id: u64, f: F)
        -> impl Future<Item = Role, Error = Error> where F: FnOnce(EditRole) -> EditRole {
        let role = f(EditRole::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(role));

        self.patch(Route::EditRole { guild_id, role_id }, Some(&map))
    }

    /// Changes the position of a role in a guild.
    pub fn edit_role_position(
        &self,
        guild_id: u64,
        role_id: u64,
        position: u64,
    ) -> impl Future<Item = Vec<Role>, Error = Error> {
        self.patch(Route::EditRole { guild_id, role_id }, Some(&json!({
            "id": role_id,
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
        webhook_id: u64,
        name: Option<&str>,
        avatar: Option<&str>,
    ) -> impl Future<Item = Webhook, Error = Error> {
        let map = json!({
            "avatar": avatar,
            "name": name,
        });

        self.patch(Route::EditWebhook { webhook_id }, Some(&map))
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
        webhook_id: u64,
        token: &str,
        name: Option<&str>,
        avatar: Option<&str>,
    ) -> impl Future<Item = Webhook, Error = Error> {
        let map = json!({
            "avatar": avatar,
            "name": name,
        });
        let route = Route::EditWebhookWithToken {
            token,
            webhook_id,
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
        webhook_id: u64,
        token: &str,
        wait: bool,
        f: F,
    ) -> impl Future<Item = Option<Message>, Error = Error> {
        let execution = f(ExecuteWebhook::default()).0;
        let map = Value::Object(serenity_utils::vecmap_to_json_map(execution));

        let route = Route::ExecuteWebhook {
            token,
            wait,
            webhook_id,
        };

        if wait {
            self.post(route, Some(&map))
        } else {
            Box::new(self.verify(route, Some(&map)).map(|_| None))
        }
    }

    /// Gets the active maintenances from Discord's Status API.
    ///
    /// Does not require authentication.
    // pub fn get_active_maintenances() -> impl Future<Item = Vec<Maintenance>, Error = Error> {
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
    pub fn get_bans(&self, guild_id: u64) -> impl Future<Item = Vec<Ban>, Error = Error> {
        self.get(Route::GetBans { guild_id })
    }

    /// Gets all audit logs in a specific guild.
    pub fn get_audit_logs(
        &self,
        guild_id: u64,
        action_type: Option<u8>,
        user_id: Option<u64>,
        before: Option<u64>,
        limit: Option<u8>,
    ) -> impl Future<Item = AuditLogs, Error = Error> {
        self.get(Route::GetAuditLogs {
            action_type,
            before,
            guild_id,
            limit,
            user_id,
        })
    }

    /// Gets current bot gateway.
    pub fn get_bot_gateway(&self) -> impl Future<Item = BotGateway, Error = Error> {
        self.get(Route::GetBotGateway)
    }

    /// Gets all invites for a channel.
    pub fn get_channel_invites(&self, channel_id: u64)
        -> impl Future<Item = Vec<RichInvite>, Error = Error> {
        self.get(Route::GetChannelInvites { channel_id })
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
    pub fn get_channel_webhooks(&self, channel_id: u64)
        -> impl Future<Item = Vec<Webhook>, Error = Error> {
        self.get(Route::GetChannelWebhooks { channel_id })
    }

    /// Gets channel information.
    pub fn get_channel(&self, channel_id: u64) -> impl Future<Item = Channel, Error = Error> {
        self.get(Route::GetChannel { channel_id })
    }

    /// Gets all channels in a guild.
    pub fn get_channels(&self, guild_id: u64)
        -> impl Future<Item = Vec<GuildChannel>, Error = Error> {
        self.get(Route::GetChannels { guild_id })
    }

    /// Gets information about the current application.
    ///
    /// **Note**: Only applications may use this endpoint.
    pub fn get_current_application_info(&self)
        -> impl Future<Item = CurrentApplicationInfo, Error = Error> {
        self.get(Route::GetCurrentApplicationInfo)
    }

    /// Gets information about the user we're connected with.
    pub fn get_current_user(&self, ) -> impl Future<Item = CurrentUser, Error = Error> {
        self.get(Route::GetCurrentUser)
    }

    /// Gets current gateway.
    pub fn get_gateway(&self) -> impl Future<Item = Gateway, Error = Error> {
        self.get(Route::GetGateway)
    }

    /// Gets guild information.
    pub fn get_guild(&self, guild_id: u64) -> impl Future<Item = PartialGuild, Error = Error> {
        self.get(Route::GetGuild { guild_id })
    }

    /// Gets a guild embed information.
    pub fn get_guild_embed(&self, guild_id: u64)
        -> impl Future<Item = GuildEmbed, Error = Error> {
        self.get(Route::GetGuildEmbed { guild_id })
    }

    /// Gets integrations that a guild has.
    pub fn get_guild_integrations(&self, guild_id: u64)
        -> impl Future<Item = Vec<Integration>, Error = Error> {
        self.get(Route::GetGuildIntegrations { guild_id })
    }

    /// Gets all invites to a guild.
    pub fn get_guild_invites(&self, guild_id: u64)
        -> impl Future<Item = Vec<RichInvite>, Error = Error> {
        self.get(Route::GetGuildInvites { guild_id })
    }

    /// Gets the members of a guild. Optionally pass a `limit` and the Id of the
    /// user to offset the result by.
    pub fn get_guild_members(
        &self,
        guild_id: u64,
        limit: Option<u64>,
        after: Option<u64>
    ) -> impl Future<Item = Vec<Member>, Error = Error> {
        let done = self.get(Route::GetGuildMembers { after, guild_id, limit })
            .and_then(move |mut v: Value| {
                if let Some(values) = v.as_array_mut() {
                    let num = Value::Number(Number::from(guild_id));

                    for value in values {
                        if let Some(element) = value.as_object_mut() {
                            element.insert("guild_id".to_string(), num.clone());
                        }
                    }
                }

                serde_json::from_value::<Vec<Member>>(v).map_err(From::from)
            });

        Box::new(done)
    }

    /// Gets the amount of users that can be pruned.
    pub fn get_guild_prune_count(&self, guild_id: u64, days: u16)
        -> impl Future<Item = GuildPrune, Error = Error> {
        self.get(Route::GetGuildPruneCount {
            days: days as u64,
            guild_id,
        })
    }

    /// Gets regions that a guild can use. If a guild has
    /// [`Feature::VipRegions`] enabled, then additional VIP-only regions are
    /// returned.
    ///
    /// [`Feature::VipRegions`]: ../model/enum.Feature.html#variant.VipRegions
    pub fn get_guild_regions(&self, guild_id: u64)
        -> impl Future<Item = Vec<VoiceRegion>, Error = Error> {
        self.get(Route::GetGuildRegions { guild_id })
    }

    /// Retrieves a list of roles in a [`Guild`].
    ///
    /// [`Guild`]: ../model/guild/struct.Guild.html
    pub fn get_guild_roles(&self, guild_id: u64)
        -> impl Future<Item = Vec<Role>, Error = Error> {
        self.get(Route::GetGuildRoles { guild_id })
    }

    /// Gets a guild's vanity URL if it has one.
    pub fn get_guild_vanity_url(&self, guild_id: u64)
        -> impl Future<Item = String, Error = Error> {
        #[derive(Deserialize)]
        struct GuildVanityUrl {
            code: String,
        }

        let done = self.get::<GuildVanityUrl>(
            Route::GetGuildVanityUrl { guild_id },
        ).map(|resp| {
            resp.code
        });

        Box::new(done)
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
    pub fn get_guild_webhooks(&self, guild_id: u64)
        -> impl Future<Item = Vec<Webhook>, Error = Error> {
        self.get(Route::GetGuildWebhooks { guild_id })
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
        -> impl Future<Item = Vec<GuildInfo>, Error = Error> {
        let (after, before) = match *target {
            GuildPagination::After(v) => (Some(v.0), None),
            GuildPagination::Before(v) => (None, Some(v.0)),
        };

        self.get(Route::GetGuilds { after, before, limit })
    }

    /// Gets information about a specific invite.
    pub fn get_invite<'a>(&self, code: &'a str, stats: bool)
        -> Box<Future<Item = Invite, Error = Error> + 'a> {
        self.get(Route::GetInvite { code, stats })
    }

    /// Gets member of a guild.
    pub fn get_member(&self, guild_id: u64, user_id: u64)
        -> impl Future<Item = Member, Error = Error> {
        let done = self.get::<Value>(Route::GetMember { guild_id, user_id })
            .and_then(move |mut v| {
                if let Some(map) = v.as_object_mut() {
                    map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                }

                serde_json::from_value(v).map_err(From::from)
            });

        Box::new(done)
    }

    /// Gets a message by an Id, bots only.
    pub fn get_message(&self, channel_id: u64, message_id: u64)
        -> impl Future<Item = Message, Error = Error> {
        self.get(Route::GetMessage { channel_id, message_id })
    }

    /// Gets X messages from a channel.
    pub fn get_messages<'a, F: FnOnce(GetMessages) -> GetMessages>(
        &'a self,
        channel_id: u64,
        f: F,
    ) -> Box<Future<Item = Vec<Message>, Error = Error> + 'a> {
        let mut map = f(GetMessages::default()).0;

        let limit = map.remove(&"limit").unwrap_or(50);
        let mut query = format!("?limit={}", limit);

        if let Some(after) = map.remove(&"after") {
            ftry!(write!(query, "&after={}", after));
        }

        if let Some(around) = map.remove(&"around") {
            ftry!(write!(query, "&around={}", around));
        }

        if let Some(before) = map.remove(&"before") {
            ftry!(write!(query, "&before={}", before));
        }

        self.get(Route::GetMessages {
            channel_id,
            query,
        })
    }

    /// Gets all pins of a channel.
    pub fn get_pins(&self, channel_id: u64) -> impl Future<Item = Vec<Message>, Error = Error> {
        self.get(Route::GetPins { channel_id })
    }

    /// Gets user Ids based on their reaction to a message.
    pub fn get_reaction_users(
        &self,
        channel_id: u64,
        message_id: u64,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<u64>
    ) -> impl Future<Item = Vec<User>, Error = Error> {
        let reaction = utils::reaction_type_data(reaction_type);
        self.get(Route::GetReactionUsers {
            limit: limit.unwrap_or(50),
            after,
            channel_id,
            message_id,
            reaction,
        })
    }

    // /// Gets the current unresolved incidents from Discord's Status API.
    // ///
    // /// Does not require authentication.
    // pub fn get_unresolved_incidents(&self) -> impl Future<Item = Vec<Incident>, Error = Error> {
    //     let client = request_client!();

    //     let response = retry(|| client.get(status!("/incidents/unresolved.json")))?;

    //     let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    //     match map.remove("incidents") {
    //         Some(v) => serde_json::from_value::<Vec<Incident>>(v)
    //             .map_err(From::from),
    //         None => Ok(vec![]),
    //     }
    // }

    // /// Gets the upcoming (planned) maintenances from Discord's Status API.
    // ///
    // /// Does not require authentication.
    // pub fn get_upcoming_maintenances(&self) -> impl Future<Item = Vec<Maintenance>, Error = Error> {
    //     let client = request_client!();

    //     let response = retry(|| {
    //         client.get(status!("/scheduled-maintenances/upcoming.json"))
    //     })?;

    //     let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    //     match map.remove("scheduled_maintenances") {
    //         Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
    //             .map_err(From::from),
    //         None => Ok(vec![]),
    //     }
    // }

    /// Gets a user by Id.
    pub fn get_user(&self, user_id: u64) -> impl Future<Item = User, Error = Error> {
        self.get(Route::GetUser { user_id })
    }

    /// Gets our DM channels.
    pub fn get_user_dm_channels(&self)
        -> impl Future<Item = Vec<PrivateChannel>, Error = Error> {
        self.get(Route::GetUserDmChannels)
    }

    /// Gets all voice regions.
    pub fn get_voice_regions(&self) -> impl Future<Item = Vec<VoiceRegion>, Error = Error> {
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
    pub fn get_webhook(&self, webhook_id: u64) -> impl Future<Item = Webhook, Error = Error> {
        self.get(Route::GetWebhook { webhook_id })
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
        webhook_id: u64,
        token: &'a str,
    ) -> Box<Future<Item = Webhook, Error = Error> + 'a> {
        self.get(Route::GetWebhookWithToken { token, webhook_id })
    }

    /// Kicks a member from a guild.
    pub fn kick_member(&self, guild_id: u64, user_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::KickMember { guild_id, user_id }, None)
    }

    /// Leaves a group DM.
    pub fn leave_group(&self, group_id: u64) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::DeleteChannel {
            channel_id: group_id,
        }, None)
    }

    /// Leaves a guild.
    pub fn leave_guild(&self, guild_id: u64) -> impl Future<Item = (), Error = Error> {
        self.verify(Route::LeaveGuild { guild_id }, None)
    }

    /// Deletes a user from group DM.
    pub fn remove_group_recipient(&self, group_id: u64, user_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::RemoveGroupRecipient { group_id, user_id }, None)
    }

    // /// Sends file(s) to a channel.
    // ///
    // /// # Errors
    // ///
    // /// Returns an
    // /// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
    // /// if the file is too large to send.
    // ///
    // /// [`HttpError::InvalidRequest`]: enum.HttpError.html#variant.InvalidRequest
    pub fn send_files<F, T, It>(
        &self,
        channel_id: u64,
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

        let mut request = ftry!(Request::get(uri).body(Body::empty()));
        form.set_body(&mut request);

        let client = Rc::clone(&self.multiparter);

        let done = client.request(request)
            .from_err()
            .and_then(verify_status)
            .and_then(|res| res.body().concat2().map_err(From::from))
            .and_then(|body| serde_json::from_slice(&body).map_err(From::from));

        Box::new(done)
    }

    /// Sends a message to a channel.
    pub fn send_message<F>(&self, channel_id: u64, f: F) -> impl Future<Item = Message, Error = Error>
        where F: FnOnce(CreateMessage) -> CreateMessage {
        let msg = f(CreateMessage::default());
        let map = Value::Object(serenity_utils::vecmap_to_json_map(msg.0));

        self.post(Route::CreateMessage { channel_id }, Some(&map))
    }

    /// Pins a message in a channel.
    pub fn pin_message(&self, channel_id: u64, message_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::PinMessage { channel_id, message_id }, None)
    }

    /// Unbans a user from a guild.
    pub fn remove_ban(&self, guild_id: u64, user_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::RemoveBan { guild_id, user_id }, None)
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
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> impl Future<Item = (), Error = Error> {
        self.verify(
            Route::RemoveMemberRole { guild_id, role_id, user_id },
            None,
        )
    }

    /// Starts removing some members from a guild based on the last time they've been online.
    pub fn start_guild_prune(&self, guild_id: u64, days: u16)
        -> impl Future<Item = GuildPrune, Error = Error> {
        self.post(Route::StartGuildPrune {
            days: days as u64,
            guild_id,
        }, None)
    }

    /// Starts syncing an integration with a guild.
    pub fn start_integration_sync(&self, guild_id: u64, integration_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(
            Route::StartIntegrationSync { guild_id, integration_id },
            None,
        )
    }

    /// Unpins a message from a channel.
    pub fn unpin_message(&self, channel_id: u64, message_id: u64)
        -> impl Future<Item = (), Error = Error> {
        self.verify(Route::UnpinMessage { channel_id, message_id }, None)
    }

    fn delete<'a, T: DeserializeOwned + 'static>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> Box<Future<Item = T, Error = Error>> {
        self.request(route, map)
    }

    fn get<'a, T: DeserializeOwned + 'static>(&self, route: Route<'a>)
        -> Box<Future<Item = T, Error = Error> + 'a> {
        self.request(route, None)
    }

    fn patch<'a, T: DeserializeOwned + 'static>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> Box<Future<Item = T, Error = Error>> {
        self.request(route, map)
    }

    fn post<'a, T: DeserializeOwned + 'static>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> Box<Future<Item = T, Error = Error>> {
        self.request(route, map)
    }

    fn build_request(
        &self,
        method: LightMethod,
        url: Cow<str>,
        map: Option<&Value>
    ) -> Result<Request<Body>> {
        let built_uri = url.as_ref();

        let body = if let Some(value) = map {
            serde_json::to_string(value)?
        } else {
            String::new()
        };

        Request::builder()
            .uri(built_uri)
            .method(method.hyper_method())
            .header(AUTHORIZATION, HeaderValue::from_str(&self.token())
                .expect("Token being sent must be a valid HTTP-encodable string."))
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body(Body::from(body))
            .map_err(|e| HttpError::Http(e))
    }

    fn request<'a, T: DeserializeOwned + 'static>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> Box<Future<Item = T, Error = Error>> {
        let (method, path, url) = route.deconstruct();

        let request = ftry!(self.build_request(method, url, map));

        let client = Rc::clone(&self.client);

        Box::new(ftry!(self.ratelimiter.try_borrow_mut()).take(&path)
            .and_then(move |_| client.request(request).map_err(From::from))
            .from_err()
            .and_then(verify_status)
            .and_then(|res| res.body().concat2().map_err(From::from))
            .and_then(|body| serde_json::from_slice(&body).map_err(From::from)))
    }

    fn verify<'a>(
        &self,
        route: Route<'a>,
        map: Option<&Value>,
    ) -> Box<Future<Item = (), Error = Error>> {
        let (method, path, url) = route.deconstruct();

        let request = ftry!(self.build_request(method, url, map));

        let client = Rc::clone(&self.client);

        Box::new(ftry!(self.ratelimiter.try_borrow_mut()).take(&path)
            .and_then(move |_| client.request(request).map_err(From::from))
            .map_err(From::from)
            .and_then(verify_status)
            .map(|_| ()))
    }

    fn token(&self) -> String {
        let pointer = Rc::into_raw(Rc::clone(&self.token));
        let token = unsafe {
            (*pointer).clone()
        };

        unsafe {
            drop(Rc::from_raw(pointer));
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
fn verify_status<T: Debug>(response: Response<T>) ->
    impl Future<Item = Response<T>, Error = Error> {
    if response.status().is_success() {
        future::ok(response)
    } else {
        future::err(Error::Http(HttpError::InvalidRequest(format!("{:?}", response))))
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
