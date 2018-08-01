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

pub mod ratelimiting;

mod error;

pub use self::error::Error as HttpError;
pub use hyper::status::{StatusClass, StatusCode};

use constants;
use hyper::{
    client::{
        Client as HyperClient,
        Request,
        RequestBuilder,
        Response as HyperResponse
    },
    header::ContentType,
    method::Method,
    mime::{Mime, SubLevel, TopLevel},
    net::HttpsConnector,
    header,
    Error as HyperError,
    Result as HyperResult,
    Url
};
use hyper_native_tls::NativeTlsClient;
use internal::prelude::*;
use model::prelude::*;
use multipart::client::Multipart;
use parking_lot::Mutex;
use self::ratelimiting::Route;
use serde_json;
use std::{
    collections::BTreeMap,
    default::Default,
    fmt::Write as FmtWrite,
    fs::File,
    io::ErrorKind as IoErrorKind,
    path::{Path, PathBuf},
    sync::Arc
};

/// An method used for ratelimiting special routes.
///
/// This is needed because `hyper`'s `Method` enum does not derive Copy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LightMethod {
    /// Indicates that a route is for "any" method.
    Any,
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

lazy_static! {
    static ref TOKEN: Arc<Mutex<String>> = Arc::new(Mutex::new(String::default()));
}

/// Sets the token to be used across all requests which require authentication.
///
/// If you are using the client module, you don't need to use this. If you're
/// using serenity solely for HTTP, you need to use this.
///
/// # Examples
///
/// Setting the token from an environment variable:
///
/// ```rust,no_run
/// # use std::error::Error;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// #
/// use serenity::http;
/// use std::env;
///
/// http::set_token(&env::var("DISCORD_TOKEN")?);
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
pub fn set_token(token: &str) { TOKEN.lock().clone_from(&token.to_string()); }

/// Adds a [`User`] as a recipient to a [`Group`].
///
/// **Note**: Groups have a limit of 10 recipients, including the current user.
///
/// [`Group`]: ../model/channel/struct.Group.html
/// [`Group::add_recipient`]: ../model/channel/struct.Group.html#method.add_recipient
/// [`User`]: ../model/user/struct.User.html
pub fn add_group_recipient(group_id: u64, user_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::None,
            put,
            "/channels/{}/recipients/{}",
            group_id,
            user_id
        ),
    )
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
pub fn add_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdMembersIdRolesId(guild_id),
            put,
            "/guilds/{}/members/{}/roles/{}",
            guild_id,
            user_id,
            role_id
        ),
    )
}

/// Bans a [`User`] from a [`Guild`], removing their messages sent in the last
/// X number of days.
///
/// Passing a `delete_message_days` of `0` is equivalent to not removing any
/// messages. Up to `7` days' worth of messages may be deleted.
///
/// **Note**: Requires that you have the [Ban Members] permission.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
/// [`User`]: ../model/user/struct.User.html
/// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
pub fn ban_user(guild_id: u64, user_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdBansUserId(guild_id),
            put,
            "/guilds/{}/bans/{}?delete_message_days={}&reason={}",
            guild_id,
            user_id,
            delete_message_days,
            reason
        ),
    )
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
pub fn ban_zeyla(guild_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
    ban_user(guild_id, 114_941_315_417_899_012, delete_message_days, reason)
}

/// Ban luna from a [`Guild`], removing her messages sent in the last X number
/// of days.
///
/// Passing a `delete_message_days` of `0` is equivalent to not removing any
/// messages. Up to `7` days' worth of messages may be deleted.
///
/// **Note**: Requires that you have the [Ban Members] permission.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
/// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
pub fn ban_luna(guild_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
    ban_user(guild_id, 180_731_582_049_550_336, delete_message_days, reason)
}

/// Ban the serenity servermoms from a [`Guild`], removing their messages
/// sent in the last X number of days.
///
/// Passing a `delete_message_days` of `0` is equivalent to not removing any
/// messages. Up to `7` days' worth of messages may be deleted.
///
/// **Note**: Requires that you have the [Ban Members] permission.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
/// [Ban Members]: ../model/permissions/constant.BAN_MEMBERS.html
pub fn ban_servermoms(guild_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
    ban_zeyla(guild_id, delete_message_days, reason)?;
    ban_luna(guild_id, delete_message_days, reason)
}

/// Broadcasts that the current user is typing in the given [`Channel`].
///
/// This lasts for about 10 seconds, and will then need to be renewed to
/// indicate that the current user is still typing.
///
/// This should rarely be used for bots, although it is a good indicator that a
/// long-running command is still being processed.
///
/// [`Channel`]: ../model/channel/enum.Channel.html
pub fn broadcast_typing(channel_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdTyping(channel_id),
            post,
            "/channels/{}/typing",
            channel_id
        ),
    )
}

/// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
///
/// Refer to the Discord's [docs] for information on what fields this requires.
///
/// **Note**: Requires the [Manage Channels] permission.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
/// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
/// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-channel
/// [Manage Channels]: ../model/permissions/constant.MANAGE_CHANNELS.html
pub fn create_channel(guild_id: u64, map: &Value) -> Result<GuildChannel> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdChannels(guild_id),
        post(body),
        "/guilds/{}/channels",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildChannel>(response)
        .map_err(From::from)
}

/// Creates an emoji in the given [`Guild`] with the given data.
///
/// View the source code for [`Context::create_emoji`] to see what fields this
/// requires.
///
/// **Note**: Requires the [Manage Emojis] permission.
///
/// [`Context::create_emoji`]: ../struct.Context.html#method.create_emoji
/// [`Guild`]: ../model/guild/struct.Guild.html
/// [Manage Emojis]: ../model/permissions/constant.MANAGE_EMOJIS.html
pub fn create_emoji(guild_id: u64, map: &Value) -> Result<Emoji> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdEmojis(guild_id),
        post(body),
        "/guilds/{}/emojis",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Emoji>(response)
        .map_err(From::from)
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
/// [US West Region]: ../model/guild/enum.Region.html#variant.UsWest
/// [documentation on this endpoint]:
/// https://discordapp.com/developers/docs/resources/guild#create-guild
/// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
pub fn create_guild(map: &Value) -> Result<PartialGuild> {
    let body = map.to_string();
    let response = request!(Route::Guilds, post(body), "/guilds");

    serde_json::from_reader::<HyperResponse, PartialGuild>(response)
        .map_err(From::from)
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
pub fn create_guild_integration(guild_id: u64, integration_id: u64, map: &Value) -> Result<()> {
    let body = map.to_string();

    verify(
        204,
        request!(
            Route::GuildsIdIntegrations(guild_id),
            post(body),
            "/guilds/{}/integrations/{}",
            guild_id,
            integration_id
        ),
    )
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
pub fn create_invite(channel_id: u64, map: &JsonMap) -> Result<RichInvite> {
    let body = serde_json::to_string(map)?;
    let response = request!(
        Route::ChannelsIdInvites(channel_id),
        post(body),
        "/channels/{}/invites",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, RichInvite>(response)
        .map_err(From::from)
}

/// Creates a permission override for a member or a role in a channel.
pub fn create_permission(channel_id: u64, target_id: u64, map: &Value) -> Result<()> {
    let body = map.to_string();

    verify(
        204,
        request!(
            Route::ChannelsIdPermissionsOverwriteId(channel_id),
            put(body),
            "/channels/{}/permissions/{}",
            channel_id,
            target_id
        ),
    )
}

/// Creates a private channel with a user.
pub fn create_private_channel(map: &Value) -> Result<PrivateChannel> {
    let body = map.to_string();
    let response = request!(Route::UsersMeChannels, post(body), "/users/@me/channels");

    serde_json::from_reader::<HyperResponse, PrivateChannel>(response)
        .map_err(From::from)
}

/// Reacts to a message.
pub fn create_reaction(channel_id: u64,
                       message_id: u64,
                       reaction_type: &ReactionType)
                       -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
            put,
            "/channels/{}/messages/{}/reactions/{}/@me",
            channel_id,
            message_id,
            reaction_type.as_data()
        ),
    )
}

/// Creates a role.
pub fn create_role(guild_id: u64, map: &JsonMap) -> Result<Role> {
    let body = serde_json::to_string(map)?;
    let response = request!(
        Route::GuildsIdRoles(guild_id),
        post(body),
        "/guilds/{}/roles",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Role>(response)
        .map_err(From::from)
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
/// let webhook = http::create_webhook(channel_id, map).expect("Error creating");
/// ```
///
/// [`GuildChannel`]: ../model/channel/struct.GuildChannel.html
pub fn create_webhook(channel_id: u64, map: &Value) -> Result<Webhook> {
    let body = map.to_string();
    let response = request!(
        Route::ChannelsIdWebhooks(channel_id),
        post(body),
        "/channels/{}/webhooks",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Webhook>(response)
        .map_err(From::from)
}

/// Deletes a private channel or a channel in a guild.
pub fn delete_channel(channel_id: u64) -> Result<Channel> {
    let response = request!(
        Route::ChannelsId(channel_id),
        delete,
        "/channels/{}",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Channel>(response)
        .map_err(From::from)
}

/// Deletes an emoji from a server.
pub fn delete_emoji(guild_id: u64, emoji_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdEmojisId(guild_id),
            delete,
            "/guilds/{}/emojis/{}",
            guild_id,
            emoji_id
        ),
    )
}

/// Deletes a guild, only if connected account owns it.
pub fn delete_guild(guild_id: u64) -> Result<PartialGuild> {
    let response = request!(Route::GuildsId(guild_id), delete, "/guilds/{}", guild_id);

    serde_json::from_reader::<HyperResponse, PartialGuild>(response)
        .map_err(From::from)
}

/// Remvoes an integration from a guild.
pub fn delete_guild_integration(guild_id: u64, integration_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdIntegrationsId(guild_id),
            delete,
            "/guilds/{}/integrations/{}",
            guild_id,
            integration_id
        ),
    )
}

/// Deletes an invite by code.
pub fn delete_invite(code: &str) -> Result<Invite> {
    let response = request!(Route::InvitesCode, delete, "/invites/{}", code);

    serde_json::from_reader::<HyperResponse, Invite>(response)
        .map_err(From::from)
}

/// Deletes a message if created by us or we have
/// specific permissions.
pub fn delete_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdMessagesId(LightMethod::Delete, channel_id),
            delete,
            "/channels/{}/messages/{}",
            channel_id,
            message_id
        ),
    )
}

/// Deletes a bunch of messages, only works for bots.
pub fn delete_messages(channel_id: u64, map: &Value) -> Result<()> {
    let body = map.to_string();

    verify(
        204,
        request!(
            Route::ChannelsIdMessagesBulkDelete(channel_id),
            post(body),
            "/channels/{}/messages/bulk-delete",
            channel_id
        ),
    )
}

/// Deletes all of the [`Reaction`]s associated with a [`Message`].
///
/// # Examples
///
/// ```rust,no_run
/// use serenity::http;
/// use serenity::model::id::{ChannelId, MessageId};
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
pub fn delete_message_reactions(channel_id: u64, message_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdMessagesIdReactions(channel_id),
            delete,
            "/channels/{}/messages/{}/reactions",
            channel_id,
            message_id
        ),
    )
}

/// Deletes a permission override from a role or a member in a channel.
pub fn delete_permission(channel_id: u64, target_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdPermissionsOverwriteId(channel_id),
            delete,
            "/channels/{}/permissions/{}",
            channel_id,
            target_id
        ),
    )
}

/// Deletes a reaction from a message if owned by us or
/// we have specific permissions.
pub fn delete_reaction(channel_id: u64,
                       message_id: u64,
                       user_id: Option<u64>,
                       reaction_type: &ReactionType)
                       -> Result<()> {
    let user = user_id
        .map(|uid| uid.to_string())
        .unwrap_or_else(|| "@me".to_string());

    verify(
        204,
        request!(
            Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
            delete,
            "/channels/{}/messages/{}/reactions/{}/{}",
            channel_id,
            message_id,
            reaction_type.as_data(),
            user
        ),
    )
}

/// Deletes a role from a server. Can't remove the default everyone role.
pub fn delete_role(guild_id: u64, role_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdRolesId(guild_id),
            delete,
            "/guilds/{}/roles/{}",
            guild_id,
            role_id
        ),
    )
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
/// use serenity::{Client, http};
/// use std::env;
///
/// // Due to the `delete_webhook` function requiring you to authenticate, you
/// // must have set the token first.
/// http::set_token(&env::var("DISCORD_TOKEN").unwrap());
///
/// http::delete_webhook(245037420704169985).expect("Error deleting webhook");
/// ```
///
/// [`Webhook`]: ../model/webhook/struct.Webhook.html
/// [`delete_webhook_with_token`]: fn.delete_webhook_with_token.html
pub fn delete_webhook(webhook_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::WebhooksId(webhook_id),
            delete,
            "/webhooks/{}",
            webhook_id,
        ),
    )
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
/// http::delete_webhook_with_token(id, token).expect("Error deleting webhook");
/// ```
///
/// [`Webhook`]: ../model/webhook/struct.Webhook.html
pub fn delete_webhook_with_token(webhook_id: u64, token: &str) -> Result<()> {
    let client = request_client!();

    verify(
        204,
        retry(|| {
            client
                .delete(&format!(api!("/webhooks/{}/{}"), webhook_id, token))
        }).map_err(Error::Hyper)?,
    )
}

/// Changes channel information.
pub fn edit_channel(channel_id: u64, map: &JsonMap) -> Result<GuildChannel> {
    let body = serde_json::to_string(map)?;
    let response = request!(
        Route::ChannelsId(channel_id),
        patch(body),
        "/channels/{}",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, GuildChannel>(response)
        .map_err(From::from)
}

/// Changes emoji information.
pub fn edit_emoji(guild_id: u64, emoji_id: u64, map: &Value) -> Result<Emoji> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdEmojisId(guild_id),
        patch(body),
        "/guilds/{}/emojis/{}",
        guild_id,
        emoji_id
    );

    serde_json::from_reader::<HyperResponse, Emoji>(response)
        .map_err(From::from)
}

/// Changes guild information.
pub fn edit_guild(guild_id: u64, map: &JsonMap) -> Result<PartialGuild> {
    let body = serde_json::to_string(map)?;
    let response = request!(
        Route::GuildsId(guild_id),
        patch(body),
        "/guilds/{}",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, PartialGuild>(response)
        .map_err(From::from)
}

/// Edits the positions of a guild's channels.
pub fn edit_guild_channel_positions(guild_id: u64, value: &Value)
    -> Result<()> {
    let body = serde_json::to_string(value)?;

    verify(
        204,
        request!(
            Route::GuildsIdChannels(guild_id),
            patch(body),
            "/guilds/{}/channels",
            guild_id,
        ),
    )
}

/// Edits a [`Guild`]'s embed setting.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
pub fn edit_guild_embed(guild_id: u64, map: &Value) -> Result<GuildEmbed> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdEmbed(guild_id),
        patch(body),
        "/guilds/{}/embed",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildEmbed>(response)
        .map_err(From::from)
}

/// Does specific actions to a member.
pub fn edit_member(guild_id: u64, user_id: u64, map: &JsonMap) -> Result<()> {
    let body = serde_json::to_string(map)?;

    verify(
        204,
        request!(
            Route::GuildsIdMembersId(guild_id),
            patch(body),
            "/guilds/{}/members/{}",
            guild_id,
            user_id
        ),
    )
}

/// Edits a message by Id.
///
/// **Note**: Only the author of a message can modify it.
pub fn edit_message(channel_id: u64, message_id: u64, map: &Value) -> Result<Message> {
    let body = map.to_string();
    let response = request!(
        Route::ChannelsIdMessagesId(LightMethod::Any, channel_id),
        patch(body),
        "/channels/{}/messages/{}",
        channel_id,
        message_id
    );

    serde_json::from_reader::<HyperResponse, Message>(response)
        .map_err(From::from)
}

/// Edits the current user's nickname for the provided [`Guild`] via its Id.
///
/// Pass `None` to reset the nickname.
///
/// [`Guild`]: ../model/guild/struct.Guild.html
pub fn edit_nickname(guild_id: u64, new_nickname: Option<&str>) -> Result<()> {
    let map = json!({ "nick": new_nickname });
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdMembersMeNick(guild_id),
        patch(body),
        "/guilds/{}/members/@me/nick",
        guild_id
    );

    verify(200, response)
}

/// Edits the current user's profile settings.
///
/// For bot users, the password is optional.
///
/// # User Accounts
///
/// If a new token is received due to a password change, then the stored token
/// internally will be updated.
///
/// **Note**: this token change may cause requests made between the actual token
/// change and when the token is internally changed to be invalid requests, as
/// the token may be outdated.
pub fn edit_profile(map: &JsonMap) -> Result<CurrentUser> {
    let body = serde_json::to_string(map)?;
    let response = request!(Route::UsersMe, patch(body), "/users/@me");

    let mut value = serde_json::from_reader::<HyperResponse, Value>(response)?;

    if let Some(map) = value.as_object_mut() {
        if !TOKEN.lock().starts_with("Bot ") {
            if let Some(Value::String(token)) = map.remove("token") {
                set_token(&token);
            }
        }
    }

    serde_json::from_value::<CurrentUser>(value)
        .map_err(From::from)
}

/// Changes a role in a guild.
pub fn edit_role(guild_id: u64, role_id: u64, map: &JsonMap) -> Result<Role> {
    let body = serde_json::to_string(map)?;
    let response = request!(
        Route::GuildsIdRolesId(guild_id),
        patch(body),
        "/guilds/{}/roles/{}",
        guild_id,
        role_id
    );

    serde_json::from_reader::<HyperResponse, Role>(response)
        .map_err(From::from)
}

/// Changes the position of a role in a guild.
pub fn edit_role_position(guild_id: u64, role_id: u64, position: u64) -> Result<Vec<Role>> {
    let body = serde_json::to_string(&json!({
        "id": role_id,
        "position": position,
    }))?;
    let response = request!(
        Route::GuildsIdRolesId(guild_id),
        patch(body),
        "/guilds/{}/roles/{}",
        guild_id,
        role_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Role>>(response)
        .map_err(From::from)
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
// external crates being incredibly messy and misleading in the end user's view.
pub fn edit_webhook(webhook_id: u64, map: &Value) -> Result<Webhook> {
    let body = map.to_string();
    let response = request!(
        Route::WebhooksId(webhook_id),
        patch(body),
        "/webhooks/{}",
        webhook_id,
    );

    serde_json::from_reader::<HyperResponse, Webhook>(response)
        .map_err(From::from)
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
pub fn edit_webhook_with_token(webhook_id: u64, token: &str, map: &JsonMap) -> Result<Webhook> {
    let body = serde_json::to_string(map)?;
    let client = request_client!();

    let response = retry(|| {
        client
            .patch(&format!(api!("/webhooks/{}/{}"), webhook_id, token))
            .body(&body)
    }).map_err(Error::Hyper)?;

    serde_json::from_reader::<HyperResponse, Webhook>(response)
        .map_err(From::from)
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
pub fn execute_webhook(webhook_id: u64,
                       token: &str,
                       wait: bool,
                       map: &JsonMap)
                       -> Result<Option<Message>> {
    let body = serde_json::to_string(map)?;

    let client = request_client!();

    let response = retry(|| {
        client
            .post(&format!(
                api!("/webhooks/{}/{}?wait={}"),
                webhook_id,
                token,
                wait
            ))
            .body(&body)
            .header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![]),
            ))
    }).map_err(Error::Hyper)?;

    if response.status == StatusCode::NoContent {
        return Ok(None);
    }

    serde_json::from_reader::<HyperResponse, Message>(response)
        .map(Some)
        .map_err(From::from)
}

/// Gets the active maintenances from Discord's Status API.
///
/// Does not require authentication.
pub fn get_active_maintenances() -> Result<Vec<Maintenance>> {
    let client = request_client!();

    let response = retry(|| {
        client.get(status!("/scheduled-maintenances/active.json"))
    })?;

    let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    match map.remove("scheduled_maintenances") {
        Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
            .map_err(From::from),
        None => Ok(vec![]),
    }
}

/// Gets all the users that are banned in specific guild.
pub fn get_bans(guild_id: u64) -> Result<Vec<Ban>> {
    let response = request!(
        Route::GuildsIdBans(guild_id),
        get,
        "/guilds/{}/bans",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Ban>>(response)
        .map_err(From::from)
}

/// Gets all audit logs in a specific guild.
pub fn get_audit_logs(guild_id: u64,
                      action_type: Option<u8>,
                      user_id: Option<u64>,
                      before: Option<u64>,
                      limit: Option<u8>) -> Result<AuditLogs> {
    let mut params = Vec::with_capacity(4);

    if let Some(action_type) = action_type {
        params.push(format!("action_type={}", action_type));
    }
    if let Some(user_id) = user_id {
        params.push(format!("user_id={}", user_id));
    }
    if let Some(before) = before {
        params.push(format!("before={}", before));
    }
    if let Some(limit) = limit {
        params.push(format!("limit={}", limit));
    }

    let mut query_string = params.join("&");
    if !query_string.is_empty() {
        query_string.insert(0, '?');
    }

    let response = request!(
        Route::GuildsIdAuditLogs(guild_id),
        get,
        "/guilds/{}/audit-logs{}",
        guild_id,
        query_string
    );

    serde_json::from_reader::<HyperResponse, AuditLogs>(response)
        .map_err(From::from)
}

/// Gets current bot gateway.
pub fn get_bot_gateway() -> Result<BotGateway> {
    let response = request!(Route::GatewayBot, get, "/gateway/bot");

    serde_json::from_reader::<HyperResponse, BotGateway>(response)
        .map_err(From::from)
}

/// Gets all invites for a channel.
pub fn get_channel_invites(channel_id: u64) -> Result<Vec<RichInvite>> {
    let response = request!(
        Route::ChannelsIdInvites(channel_id),
        get,
        "/channels/{}/invites",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Vec<RichInvite>>(response)
        .map_err(From::from)
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
pub fn get_channel_webhooks(channel_id: u64) -> Result<Vec<Webhook>> {
    let response = request!(
        Route::ChannelsIdWebhooks(channel_id),
        get,
        "/channels/{}/webhooks",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Webhook>>(response)
        .map_err(From::from)
}

/// Gets channel information.
pub fn get_channel(channel_id: u64) -> Result<Channel> {
    let response = request!(
        Route::ChannelsId(channel_id),
        get,
        "/channels/{}",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Channel>(response)
        .map_err(From::from)
}

/// Gets all channels in a guild.
pub fn get_channels(guild_id: u64) -> Result<Vec<GuildChannel>> {
    let response = request!(
        Route::ChannelsId(guild_id),
        get,
        "/guilds/{}/channels",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<GuildChannel>>(response)
        .map_err(From::from)
}

/// Gets information about the current application.
///
/// **Note**: Only applications may use this endpoint.
pub fn get_current_application_info() -> Result<CurrentApplicationInfo> {
    let response = request!(Route::None, get, "/oauth2/applications/@me");

    serde_json::from_reader::<HyperResponse, CurrentApplicationInfo>(response)
        .map_err(From::from)
}

/// Gets information about the user we're connected with.
pub fn get_current_user() -> Result<CurrentUser> {
    let response = request!(Route::UsersMe, get, "/users/@me");

    serde_json::from_reader::<HyperResponse, CurrentUser>(response)
        .map_err(From::from)
}

/// Gets current gateway.
pub fn get_gateway() -> Result<Gateway> {
    let response = request!(Route::Gateway, get, "/gateway");

    serde_json::from_reader::<HyperResponse, Gateway>(response)
        .map_err(From::from)
}

/// Gets guild information.
pub fn get_guild(guild_id: u64) -> Result<PartialGuild> {
    let response = request!(Route::GuildsId(guild_id), get, "/guilds/{}", guild_id);

    serde_json::from_reader::<HyperResponse, PartialGuild>(response)
        .map_err(From::from)
}

/// Gets a guild embed information.
pub fn get_guild_embed(guild_id: u64) -> Result<GuildEmbed> {
    let response = request!(
        Route::GuildsIdEmbed(guild_id),
        get,
        "/guilds/{}/embeds",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildEmbed>(response)
        .map_err(From::from)
}

/// Gets integrations that a guild has.
pub fn get_guild_integrations(guild_id: u64) -> Result<Vec<Integration>> {
    let response = request!(
        Route::GuildsIdIntegrations(guild_id),
        get,
        "/guilds/{}/integrations",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Integration>>(response)
        .map_err(From::from)
}

/// Gets all invites to a guild.
pub fn get_guild_invites(guild_id: u64) -> Result<Vec<RichInvite>> {
    let response = request!(
        Route::GuildsIdInvites(guild_id),
        get,
        "/guilds/{}/invites",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<RichInvite>>(response)
        .map_err(From::from)
}

/// Gets a guild's vanity URL if it has one.
pub fn get_guild_vanity_url(guild_id: u64) -> Result<String> {
    #[derive(Deserialize)]
    struct GuildVanityUrl {
        code: String,
    }

    let response = request!(
        Route::GuildsIdVanityUrl(guild_id),
        get,
        "/guilds/{}/vanity-url",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildVanityUrl>(response)
        .map(|x| x.code)
        .map_err(From::from)
}

/// Gets the members of a guild. Optionally pass a `limit` and the Id of the
/// user to offset the result by.
pub fn get_guild_members(guild_id: u64,
                         limit: Option<u64>,
                         after: Option<u64>)
                         -> Result<Vec<Member>> {
    let response = request!(
        Route::GuildsIdMembers(guild_id),
        get,
        "/guilds/{}/members?limit={}&after={}",
        guild_id,
        limit.unwrap_or(500),
        after.unwrap_or(0)
    );

    let mut v = serde_json::from_reader::<HyperResponse, Value>(response)?;

    if let Some(values) = v.as_array_mut() {
        let num = Value::Number(Number::from(guild_id));

        for value in values {
            if let Some(element) = value.as_object_mut() {
                element.insert("guild_id".to_string(), num.clone());
            }
        }
    }

    serde_json::from_value::<Vec<Member>>(v).map_err(From::from)
}

/// Gets the amount of users that can be pruned.
pub fn get_guild_prune_count(guild_id: u64, map: &Value) -> Result<GuildPrune> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdPrune(guild_id),
        get(body),
        "/guilds/{}/prune",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildPrune>(response)
        .map_err(From::from)
}

/// Gets regions that a guild can use. If a guild has the `VIP_REGIONS` feature
/// enabled, then additional VIP-only regions are returned.
pub fn get_guild_regions(guild_id: u64) -> Result<Vec<VoiceRegion>> {
    let response = request!(
        Route::GuildsIdRegions(guild_id),
        get,
        "/guilds/{}/regions",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<VoiceRegion>>(response)
        .map_err(From::from)
}

/// Retrieves a list of roles in a [`Guild`].
///
/// [`Guild`]: ../model/guild/struct.Guild.html
pub fn get_guild_roles(guild_id: u64) -> Result<Vec<Role>> {
    let response = request!(
        Route::GuildsIdRoles(guild_id),
        get,
        "/guilds/{}/roles",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Role>>(response)
        .map_err(From::from)
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
pub fn get_guild_webhooks(guild_id: u64) -> Result<Vec<Webhook>> {
    let response = request!(
        Route::GuildsIdWebhooks(guild_id),
        get,
        "/guilds/{}/webhooks",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Webhook>>(response)
        .map_err(From::from)
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
/// use serenity::model::id::GuildId;
///
/// let guild_id = GuildId(81384788765712384);
///
/// let guilds = get_guilds(&GuildPagination::After(guild_id), 10).unwrap();
/// ```
///
/// [docs]: https://discordapp.com/developers/docs/resources/user#get-current-user-guilds
pub fn get_guilds(target: &GuildPagination, limit: u64) -> Result<Vec<GuildInfo>> {
    let mut uri = format!("/users/@me/guilds?limit={}", limit);

    match *target {
        GuildPagination::After(id) => {
            write!(uri, "&after={}", id)?;
        },
        GuildPagination::Before(id) => {
            write!(uri, "&before={}", id)?;
        },
    }

    let response = request!(Route::UsersMeGuilds, get, "{}", uri);

    serde_json::from_reader::<HyperResponse, Vec<GuildInfo>>(response)
        .map_err(From::from)
}

/// Gets information about a specific invite.
#[allow(unused_mut)]
pub fn get_invite(code: &str, stats: bool) -> Result<Invite> {
    let mut invite = code;

    #[cfg(feature = "utils")]
    {
        invite = ::utils::parse_invite(invite);
    }

    let mut uri = format!("/invites/{}", invite);

    if stats {
        uri.push_str("?with_counts=true");
    }

    let response = request!(Route::InvitesCode, get, "{}", uri);

    serde_json::from_reader::<HyperResponse, Invite>(response)
        .map_err(From::from)
}

/// Gets member of a guild.
pub fn get_member(guild_id: u64, user_id: u64) -> Result<Member> {
    let response = request!(
        Route::GuildsIdMembersId(guild_id),
        get,
        "/guilds/{}/members/{}",
        guild_id,
        user_id
    );

    let mut v = serde_json::from_reader::<HyperResponse, Value>(response)?;

    if let Some(map) = v.as_object_mut() {
        map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
    }

    serde_json::from_value::<Member>(v).map_err(From::from)
}

/// Gets a message by an Id, bots only.
pub fn get_message(channel_id: u64, message_id: u64) -> Result<Message> {
    let response = request!(
        Route::ChannelsIdMessagesId(LightMethod::Any, channel_id),
        get,
        "/channels/{}/messages/{}",
        channel_id,
        message_id
    );

    serde_json::from_reader::<HyperResponse, Message>(response)
        .map_err(From::from)
}

/// Gets X messages from a channel.
pub fn get_messages(channel_id: u64, query: &str) -> Result<Vec<Message>> {
    let url = format!(api!("/channels/{}/messages{}"), channel_id, query);
    let client = request_client!();

    let response = request(Route::ChannelsIdMessages(channel_id), || client.get(&url))?;

    serde_json::from_reader::<HyperResponse, Vec<Message>>(response)
        .map_err(From::from)
}

/// Gets all pins of a channel.
pub fn get_pins(channel_id: u64) -> Result<Vec<Message>> {
    let response = request!(
        Route::ChannelsIdPins(channel_id),
        get,
        "/channels/{}/pins",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Vec<Message>>(response)
        .map_err(From::from)
}

/// Gets user Ids based on their reaction to a message. This endpoint is dumb.
pub fn get_reaction_users(channel_id: u64,
                          message_id: u64,
                          reaction_type: &ReactionType,
                          limit: u8,
                          after: Option<u64>)
                          -> Result<Vec<User>> {
    let mut uri = format!(
        "/channels/{}/messages/{}/reactions/{}?limit={}",
        channel_id,
        message_id,
        reaction_type.as_data(),
        limit
    );

    if let Some(user_id) = after {
        write!(uri, "&after={}", user_id)?;
    }

    let response = request!(
        Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
        get,
        "{}",
        uri
    );

    serde_json::from_reader::<HyperResponse, Vec<User>>(response)
        .map_err(From::from)
}

/// Gets the current unresolved incidents from Discord's Status API.
///
/// Does not require authentication.
pub fn get_unresolved_incidents() -> Result<Vec<Incident>> {
    let client = request_client!();

    let response = retry(|| client.get(status!("/incidents/unresolved.json")))?;

    let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    match map.remove("incidents") {
        Some(v) => serde_json::from_value::<Vec<Incident>>(v)
            .map_err(From::from),
        None => Ok(vec![]),
    }
}

/// Gets the upcoming (planned) maintenances from Discord's Status API.
///
/// Does not require authentication.
pub fn get_upcoming_maintenances() -> Result<Vec<Maintenance>> {
    let client = request_client!();

    let response = retry(|| {
        client.get(status!("/scheduled-maintenances/upcoming.json"))
    })?;

    let mut map: BTreeMap<String, Value> = serde_json::from_reader(response)?;

    match map.remove("scheduled_maintenances") {
        Some(v) => serde_json::from_value::<Vec<Maintenance>>(v)
            .map_err(From::from),
        None => Ok(vec![]),
    }
}

/// Gets a user by Id.
pub fn get_user(user_id: u64) -> Result<User> {
    let response = request!(Route::UsersId, get, "/users/{}", user_id);

    serde_json::from_reader::<HyperResponse, User>(response)
        .map_err(From::from)
}

/// Gets our DM channels.
pub fn get_user_dm_channels() -> Result<Vec<PrivateChannel>> {
    let response = request!(Route::UsersMeChannels, get, "/users/@me/channels");

    serde_json::from_reader::<HyperResponse, Vec<PrivateChannel>>(response)
        .map_err(From::from)
}

/// Gets all voice regions.
pub fn get_voice_regions() -> Result<Vec<VoiceRegion>> {
    let response = request!(Route::VoiceRegions, get, "/voice/regions");

    serde_json::from_reader::<HyperResponse, Vec<VoiceRegion>>(response)
        .map_err(From::from)
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
pub fn get_webhook(webhook_id: u64) -> Result<Webhook> {
    let response = request!(
        Route::WebhooksId(webhook_id),
        get,
        "/webhooks/{}",
        webhook_id,
    );

    serde_json::from_reader::<HyperResponse, Webhook>(response)
        .map_err(From::from)
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
pub fn get_webhook_with_token(webhook_id: u64, token: &str) -> Result<Webhook> {
    let client = request_client!();

    let response = retry(|| {
        client
            .get(&format!(api!("/webhooks/{}/{}"), webhook_id, token))
    }).map_err(Error::Hyper)?;

    serde_json::from_reader::<HyperResponse, Webhook>(response)
        .map_err(From::from)
}

/// Kicks a member from a guild.
pub fn kick_member(guild_id: u64, user_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdMembersId(guild_id),
            delete,
            "/guilds/{}/members/{}",
            guild_id,
            user_id
        ),
    )
}

/// Leaves a group DM.
pub fn leave_group(guild_id: u64) -> Result<Group> {
    let response = request!(Route::None, delete, "/channels/{}", guild_id);

    serde_json::from_reader::<HyperResponse, Group>(response)
        .map_err(From::from)
}

/// Leaves a guild.
pub fn leave_guild(guild_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::UsersMeGuildsId,
            delete,
            "/users/@me/guilds/{}",
            guild_id
        ),
    )
}

/// Deletes a user from group DM.
pub fn remove_group_recipient(group_id: u64, user_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::None,
            delete,
            "/channels/{}/recipients/{}",
            group_id,
            user_id
        ),
    )
}

/// Sends file(s) to a channel.
///
/// # Errors
///
/// Returns an
/// [`HttpError::InvalidRequest(PayloadTooLarge)`][`HttpError::InvalidRequest`]
/// if the file is too large to send.
///
/// [`HttpError::InvalidRequest`]: enum.HttpError.html#variant.InvalidRequest
pub fn send_files<'a, T, It: IntoIterator<Item=T>>(channel_id: u64, files: It, map: JsonMap) -> Result<Message>
    where T: Into<AttachmentType<'a>> {
    let uri = format!(api!("/channels/{}/messages"), channel_id);
    let url = match Url::parse(&uri) {
        Ok(url) => url,
        Err(_) => return Err(Error::Url(uri)),
    };

    let tc = NativeTlsClient::new()?;
    let connector = HttpsConnector::new(tc);
    let mut request = Request::with_connector(Method::Post, url, &connector)?;
    request
        .headers_mut()
        .set(header::Authorization(TOKEN.lock().clone()));
    request
        .headers_mut()
        .set(header::UserAgent(constants::USER_AGENT.to_string()));

    let mut request = Multipart::from_request(request)?;
    let mut file_num = "0".to_string();

    for file in files {
        match file.into() {
            AttachmentType::Bytes((mut bytes, filename)) => {
                request
                    .write_stream(&file_num, &mut bytes, Some(filename), None)?;
            },
            AttachmentType::File((mut f, filename)) => {
                request
                    .write_stream(&file_num, &mut f, Some(filename), None)?;
            },
            AttachmentType::Path(p) => {
                request.write_file(&file_num, &p)?;
            },
        }

        unsafe {
            let vec = file_num.as_mut_vec();
            vec[0] += 1;
        }
    }

    for (k, v) in map {
        match v {
            Value::Bool(false) => request.write_text(&k, "false")?,
            Value::Bool(true) => request.write_text(&k, "true")?,
            Value::Number(inner) => request.write_text(&k, inner.to_string())?,
            Value::String(inner) => request.write_text(&k, inner)?,
            Value::Object(inner) => request.write_text(&k, serde_json::to_string(&inner)?)?,
            _ => continue,
        };
    }

    let response = request.send()?;

    if response.status.class() != StatusClass::Success {
        return Err(Error::Http(HttpError::UnsuccessfulRequest(response)));
    }

    serde_json::from_reader::<HyperResponse, Message>(response)
        .map_err(From::from)
}

/// Sends a message to a channel.
pub fn send_message(channel_id: u64, map: &Value) -> Result<Message> {
    let body = map.to_string();
    let response = request!(
        Route::ChannelsIdMessages(channel_id),
        post(body),
        "/channels/{}/messages",
        channel_id
    );

    serde_json::from_reader::<HyperResponse, Message>(response)
        .map_err(From::from)
}

/// Pins a message in a channel.
pub fn pin_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdPinsMessageId(channel_id),
            put,
            "/channels/{}/pins/{}",
            channel_id,
            message_id
        ),
    )
}

/// Unbans a user from a guild.
pub fn remove_ban(guild_id: u64, user_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdBansUserId(guild_id),
            delete,
            "/guilds/{}/bans/{}",
            guild_id,
            user_id
        ),
    )
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
pub fn remove_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdMembersIdRolesId(guild_id),
            delete,
            "/guilds/{}/members/{}/roles/{}",
            guild_id,
            user_id,
            role_id
        ),
    )
}

/// Starts removing some members from a guild based on the last time they've been online.
pub fn start_guild_prune(guild_id: u64, map: &Value) -> Result<GuildPrune> {
    let body = map.to_string();
    let response = request!(
        Route::GuildsIdPrune(guild_id),
        post(body),
        "/guilds/{}/prune",
        guild_id
    );

    serde_json::from_reader::<HyperResponse, GuildPrune>(response)
        .map_err(From::from)
}

/// Starts syncing an integration with a guild.
pub fn start_integration_sync(guild_id: u64, integration_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::GuildsIdIntegrationsIdSync(guild_id),
            post,
            "/guilds/{}/integrations/{}/sync",
            guild_id,
            integration_id
        ),
    )
}

/// Unpins a message from a channel.
pub fn unpin_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(
        204,
        request!(
            Route::ChannelsIdPinsMessageId(channel_id),
            delete,
            "/channels/{}/pins/{}",
            channel_id,
            message_id
        ),
    )
}

fn request<'a, F>(route: Route, f: F) -> Result<HyperResponse>
    where F: Fn() -> RequestBuilder<'a> {
    let response = ratelimiting::perform(route, || {
        f().header(header::Authorization(TOKEN.lock().clone()))
            .header(header::ContentType::json())
    })?;

    if response.status.class() == StatusClass::Success {
        Ok(response)
    } else {
        Err(Error::Http(HttpError::UnsuccessfulRequest(response)))
    }
}

pub(crate) fn retry<'a, F>(f: F) -> HyperResult<HyperResponse>
    where F: Fn() -> RequestBuilder<'a> {
    let req = || {
        f().header(header::UserAgent(constants::USER_AGENT.to_string()))
            .send()
    };

    match req() {
        Err(HyperError::Io(ref io)) if io.kind() == IoErrorKind::ConnectionAborted => req(),
        other => other,
    }
}

fn verify(expected: u16, response: HyperResponse) -> Result<()> {
    if response.status.to_u16() == expected {
        return Ok(());
    }

    debug!("Expected {}, got {}", expected, response.status);
    trace!("Unsuccessful response: {:?}", response);

    Err(Error::Http(HttpError::UnsuccessfulRequest(response)))
}

/// Enum that allows a user to pass a `Path` or a `File` type to `send_files`
pub enum AttachmentType<'a> {
    /// Indicates that the `AttachmentType` is a byte slice with a filename.
    Bytes((&'a [u8], &'a str)),
    /// Indicates that the `AttachmentType` is a `File`
    File((&'a File, &'a str)),
    /// Indicates that the `AttachmentType` is a `Path`
    Path(&'a Path),
}

impl<'a> From<(&'a [u8], &'a str)> for AttachmentType<'a> {
    fn from(params: (&'a [u8], &'a str)) -> AttachmentType { AttachmentType::Bytes(params) }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    fn from(s: &'a str) -> AttachmentType { AttachmentType::Path(Path::new(s)) }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType { AttachmentType::Path(pathbuf.as_path()) }
}

impl<'a> From<(&'a File, &'a str)> for AttachmentType<'a> {
    fn from(f: (&'a File, &'a str)) -> AttachmentType<'a> { AttachmentType::File((f.0, f.1)) }
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

#[cfg(test)]
mod test {
    use super::AttachmentType;
    use std::path::Path;

    #[test]
    fn test_attachment_type() {
        assert!(match AttachmentType::from(Path::new("./dogs/corgis/kona.png")) {
            AttachmentType::Path(_) => true,
            _ => false,
        });
        assert!(match AttachmentType::from("./cats/copycat.png") {
            AttachmentType::Path(_) => true,
            _ => false,
        });
    }
}
