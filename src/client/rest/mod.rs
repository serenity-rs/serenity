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
//! Note that you may want to perform requests through a [`Context`] or through
//! [model]s' instance methods where possible, as they each offer different
//! levels of a high-level interface to the HTTP module.
//!
//! [`Client`]: ../struct.Client.html
//! [`Context`]: ../struct.Context.html
//! [model]: ../../model/index.html

mod ratelimiting;

use hyper::client::{
    Client as HyperClient,
    RequestBuilder,
    Response as HyperResponse,
    Request,
};
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::{Error as HyperError, Result as HyperResult, Url, header};
use multipart::client::Multipart;
use self::ratelimiting::Route;
use serde_json::builder::ObjectBuilder;
use serde_json;
use std::default::Default;
use std::io::{ErrorKind as IoErrorKind, Read};
use std::sync::{Arc, Mutex};
use ::constants;
use ::internal::prelude::*;
use ::model::*;
use ::utils::decode_array;

lazy_static! {
    static ref TOKEN: Arc<Mutex<String>> = Arc::new(Mutex::new(String::default()));
}

/// Sets the token to be used across all requests which require authentication.
///
/// This is really only for internal use, and if you are reading this as a user,
/// you should _not_ use this yourself.
#[doc(hidden)]
pub fn set_token(token: &str) {
    TOKEN.lock().unwrap().clone_from(&token.to_owned());
}

/// Accepts the [`Invite`] given its code, placing the current user in the
/// [`Guild`] that the invite was for.
///
/// Use [`utils::parse_invite`] to retrieve codes from URLs.
///
/// Refer to the documentation for [`Context::accept_invite`] for restrictions on
/// accepting an invite.
///
/// This will fire the [`Client::on_guild_create`] handler once the associated
/// event is received.
///
/// **Note**: This will fail if you are already in the guild, or are banned. A
/// ban is equivilant to an IP ban.
///
/// **Note**: Requires that the current user be a user account. Bots can not
/// accept invites. Instead, they must be accepted via OAuth2 authorization
/// links. These are in the format of:
///
/// `https://discordapp.com/oauth2/authorize?client_id=CLIENT_ID&scope=bot`
///
/// # Examples
///
/// Accept an invite given a code from a URL:
///
/// ```rust,no_run
/// use serenity::client::rest;
/// use serenity::utils;
///
/// let url = "https://discord.gg/0cDvIgU2voY8RSYL";
/// let code = utils::parse_invite(url);
///
/// let _result = rest::accept_invite(code);
/// ```
///
/// [`Context::accept_invite`]: ../struct.Context.html#method.accept_invite
/// [`Invite`]: ../../model/struct.Invite.html
/// [`utils::parse_invite`]: ../../utils/fn.parse_invite.html
pub fn accept_invite(code: &str) -> Result<Invite> {
    let response = request!(Route::InvitesCode, post, "/invites/{}", code);

    Invite::decode(try!(serde_json::from_reader(response)))
}

/// Marks a [`Channel`] as being "read" up to a certain [`Message`]. Any
/// message past the given one will not be marked as read.
///
/// Usually you should use this to mark the latest message as being read.
///
/// **Note**: Bot users should not use this, as it has no bearing on them
/// whatsoever.
///
/// [`Channel`]: ../../model/enum.Channel.html
/// [`Message`]: ../../model/struct.Message.html
pub fn ack_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(204, request!(Route::ChannelsIdMessagesIdAck(channel_id),
                         post,
                         "/channels/{}/messages/{}/ack",
                         channel_id,
                         message_id))
}

/// Adds a [`User`] as a recipient to a [`Group`].
///
/// **Note**: Groups have a limit of 10 recipients, including the current user.
///
/// [`Group`]: ../../model/struct.Group.html
/// [`Group::add_recipient`]: ../../model/struct.Group.html#method.add_recipient
/// [`User`]: ../../model/struct.User.html
pub fn add_group_recipient(group_id: u64, user_id: u64)
    -> Result<()> {
    verify(204, request!(Route::None,
                         put,
                         "/channels/{}/recipients/{}",
                         group_id,
                         user_id))
}

/// Adds a single [`Role`] to a [`Member`] in a [`Guild`].
///
/// **Note**: Requires the [Manage Roles] permission and respect of role
/// hierarchy.
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`Member`]: ../../model/struct.Member.html
/// [`Role`]: ../../model/struct.Role.html
/// [Manage Roles]: ../../model/permissions/constant.MANAGE_ROLES.html
pub fn add_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdMembersIdRolesId(guild_id),
                         put,
                         "/guilds/{}/members/{}/roles/{}",
                         guild_id,
                         user_id,
                         role_id))
}

/// Bans a [`User`] from a [`Guild`], removing their messages sent in the last
/// X number of days.
///
/// Passing a `delete_message_days` of `0` is equivilant to not removing any
/// messages. Up to `7` days' worth of messages may be deleted.
///
/// **Note**: Requires that you have the [Ban Members] permission.
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`User`]: ../../model/struct.User.html
/// [Ban Members]: ../../model/permissions/constant.BAN_MEMBERS.html
pub fn ban_user(guild_id: u64, user_id: u64, delete_message_days: u8)
    -> Result<()> {
    verify(204, request!(Route::GuildsIdBansUserId(guild_id),
                         put,
                         "/guilds/{}/bans/{}?delete_message_days={}",
                         guild_id,
                         user_id,
                         delete_message_days))
}

/// Broadcasts that the current user is typing in the given [`Channel`].
///
/// This lasts for about 10 seconds, and will then need to be renewed to
/// indicate that the current user is still typing.
///
/// This should rarely be used for bots, although it is a good indicator that a
/// long-running command is still being processed.
///
/// [`Channel`]: ../../model/enum.Channel.html
pub fn broadcast_typing(channel_id: u64) -> Result<()> {
    verify(204, request!(Route::ChannelsIdTyping(channel_id),
                         post,
                         "/channels/{}/typing",
                         channel_id))
}

/// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
///
/// Refer to the Discord's [docs] for information on what fields this requires.
///
/// **Note**: Requires the [Manage Channels] permission.
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`GuildChannel`]: ../../model/struct.GuildChannel.html
/// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-channel
/// [Manage Channels]: ../../model/permissions/constant.MANAGE_CHANNELS.html
pub fn create_channel(guild_id: u64, map: Value) -> Result<Channel> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdChannels(guild_id),
                            post(body),
                            "/guilds/{}/channels",
                            guild_id);

    Channel::decode(try!(serde_json::from_reader(response)))
}

/// Creates an emoji in the given [`Guild`] with the given data.
///
/// View the source code for [`Context::create_emoji`] to see what fields this
/// requires.
///
/// **Note**: Requires the [Manage Emojis] permission.
///
/// [`Context::create_emoji`]: ../struct.Context.html#method.create_emoji
/// [`Guild`]: ../../model/struct.Guild.html
/// [Manage Emojis]: ../../model/permissions/constant.MANAGE_EMOJIS.html
pub fn create_emoji(guild_id: u64, map: Value)
    -> Result<Emoji> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdEmojis(guild_id),
                            post(body),
                            "/guilds/{}/emojis",
                            guild_id);

    Emoji::decode(try!(serde_json::from_reader(response)))
}

/// Creates a guild with the data provided.
///
/// Only a [`PartialGuild`] will be immediately returned, and a full [`Guild`]
/// will be received over a [`Shard`], if at least one is running.
///
/// **Note**: This endpoint is usually only available for user accounts. Refer
/// to Discord's documentation for the endpoint [here][whitelist] for more
/// information. If your bot requires this, re-think what you are doing and
/// whether it _really_ needs to be doing this.
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
/// use serenity::client::rest;
///
/// let map = ObjectBuilder::new()
///     .insert("name", "test")
///     .insert("region", "us-west")
///     .build();
///
/// let _result = rest::create_guild(map);
/// ```
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`PartialGuild`]: ../../model/struct.PartialGuild.html
/// [`Shard`]: ../gateway/struct.Shard.html
/// [US West Region]: ../../model/enum.Region.html#variant.UsWest
/// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
pub fn create_guild(map: Value) -> Result<PartialGuild> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::Guilds, post(body), "/guilds");

    PartialGuild::decode(try!(serde_json::from_reader(response)))
}

/// Creates an [`Integration`] for a [`Guild`].
///
/// Refer to Discord's [docs] for field information.
///
/// **Note**: Requires the [Manage Guild] permission.
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`Integration`]: ../../model/struct.Integration.html
/// [Manage Guild]: ../../model/permissions/constant.MANAGE_GUILD.html
/// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-integration
pub fn create_guild_integration(guild_id: u64, integration_id: u64, map: Value)
    -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::GuildsIdIntegrations(guild_id),
                         post(body),
                         "/guilds/{}/integrations/{}",
                         guild_id,
                         integration_id))
}

/// Creates a [`RichInvite`] for the given [channel][`GuildChannel`].
///
/// Refer to Discord's [docs] for field information.
///
/// All fields are optional.
///
/// **Note**: Requires the [Create Invite] permission.
///
/// [`GuildChannel`]: ../../model/struct.GuildChannel.html
/// [`RichInvite`]: ../../model/struct.RichInvite.html
/// [Create Invite]: ../../model/permissions/constant.CREATE_INVITE.html
/// [docs]: https://discordapp.com/developers/docs/resources/channel#create-channel-invite
pub fn create_invite(channel_id: u64, map: Value)
    -> Result<RichInvite> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::ChannelsIdInvites(channel_id),
                            post(body),
                            "/channels/{}/invites",
                            channel_id);

    RichInvite::decode(try!(serde_json::from_reader(response)))
}

/// Creates a permission override for a member or a role in a channel.
pub fn create_permission(channel_id: u64, target_id: u64, map: Value)
    -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::ChannelsIdPermissionsOverwriteId(channel_id),
                         put(body),
                         "/channels/{}/permissions/{}",
                         channel_id,
                         target_id))
}

/// Creates a private channel with a user.
pub fn create_private_channel(map: Value)
    -> Result<PrivateChannel> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::UsersMeChannels,
                            post(body),
                            "/users/@me/channels");

    PrivateChannel::decode(try!(serde_json::from_reader(response)))
}

/// Reacts to a message.
pub fn create_reaction(channel_id: u64,
                       message_id: u64,
                       reaction_type: ReactionType)
                       -> Result<()> {
    verify(204, request!(Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                         put,
                         "/channels/{}/messages/{}/reactions/{}/@me",
                         channel_id,
                         message_id,
                         reaction_type.as_data()))
}

/// Creates a role.
pub fn create_role(guild_id: u64) -> Result<Role> {
    let body = String::from("{}");
    let response = request!(Route::GuildsIdRoles(guild_id),
                            post(body),
                            "/guilds/{}/roles",
                            guild_id);

    Role::decode(try!(serde_json::from_reader(response)))
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
/// use serenity::client::rest;
///
/// let channel_id = 81384788765712384;
/// let map = ObjectBuilder::new().insert("name", "test").build();
///
/// let webhook = rest::create_webhook(channel_id, map).expect("err creating");
/// ```
///
/// [`GuildChannel`]: ../../model/struct.GuildChannel.html
pub fn create_webhook(channel_id: u64, map: Value) -> Result<Webhook> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::ChannelsIdWebhooks(channel_id),
                            post(body),
                            "/channels/{}/webhooks",
                            channel_id);

    Webhook::decode(try!(serde_json::from_reader(response)))
}

/// Deketes a private channel or a channel in a guild.
pub fn delete_channel(channel_id: u64) -> Result<Channel> {
    let response = request!(Route::ChannelsId(channel_id),
                            delete,
                            "/channels/{}",
                            channel_id);

    Channel::decode(try!(serde_json::from_reader(response)))
}

/// Removes an emoji from a server.
pub fn delete_emoji(guild_id: u64, emoji_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdEmojisId(guild_id),
                         delete,
                         "/guilds/{}/emojis/{}",
                         guild_id,
                         emoji_id))
}

/// Removes a guild, only if connected account owns it.
pub fn delete_guild(guild_id: u64) -> Result<PartialGuild> {
    let response = request!(Route::GuildsId(guild_id),
                            delete,
                            "/guilds/{}",
                            guild_id);

    PartialGuild::decode(try!(serde_json::from_reader(response)))
}

/// Remvoes an integration from a guild.
pub fn delete_guild_integration(guild_id: u64, integration_id: u64)
    -> Result<()> {
    verify(204, request!(Route::GuildsIdIntegrationsId(guild_id),
                         delete,
                         "/guilds/{}/integrations/{}",
                         guild_id,
                         integration_id))
}

/// Removes an invite by code.
pub fn delete_invite(code: &str) -> Result<Invite> {
    let response = request!(Route::InvitesCode, delete, "/invite/{}", code);

    Invite::decode(try!(serde_json::from_reader(response)))
}

/// Removes a message if created by us or we have
/// specific permissions.
pub fn delete_message(channel_id: u64, message_id: u64)
    -> Result<()> {
    verify(204, request!(Route::ChannelsIdMessagesId(channel_id),
                         delete,
                         "/channels/{}/messages/{}",
                         channel_id,
                         message_id))
}

/// Removes a bunch of messages, only works for bots.
pub fn delete_messages(channel_id: u64, map: Value) -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::ChannelsIdMessagesBulkDelete(channel_id),
                         post(body),
                         "/channels/{}/messages/bulk_delete",
                         channel_id))
}

/// Removes all of the [`Reaction`]s associated with a [`Message`].
///
/// # Examples
///
/// ```rust,no_run
/// use serenity::client::rest;
/// use serenity::model::{ChannelId, MessageId};
///
/// let channel_id = ChannelId(7);
/// let message_id = MessageId(8);
///
/// match rest::delete_message_reactions(channel_id.0, message_id.0) {
///     Ok(()) => println!("Reactions deleted"),
///     Err(why) => println!("Error deleting reactions: {:?}", why),
/// }
/// ```
///
/// [`Message`]: ../../model/struct.Message.html
/// [`Reaction`]: ../../model/struct.Reaction.html
pub fn delete_message_reactions(channel_id: u64, message_id: u64)
    -> Result<()> {
    verify(204, request!(Route::ChannelsIdMessagesIdReactions(channel_id),
                         delete,
                         "/channels/{}/messages/{}/reactions",
                         channel_id,
                         message_id))
}

/// Removes a permission override from a role or a member in a channel.
pub fn delete_permission(channel_id: u64, target_id: u64)
    -> Result<()> {
    verify(204, request!(Route::ChannelsIdPermissionsOverwriteId(channel_id),
                         delete,
                         "/channels/{}/permissions/{}",
                         channel_id,
                         target_id))
}

/// Removes a reaction from a message if owned by us or
/// we have specific permissions.
pub fn delete_reaction(channel_id: u64,
                       message_id: u64,
                       user_id: Option<u64>,
                       reaction_type: ReactionType)
                       -> Result<()> {
    let user = user_id.map(|uid| uid.to_string()).unwrap_or("@me".to_string());

    verify(204, request!(Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                         delete,
                         "/channels/{}/messages/{}/reactions/{}/{}",
                         channel_id,
                         message_id,
                         reaction_type.as_data(),
                         user))
}

/// Removes a role from a server. Can't remove the default everyone role.
pub fn delete_role(guild_id: u64, role_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdRolesId(guild_id),
                         delete,
                         "/guilds/{}/roles/{}",
                         guild_id,
                         role_id))
}

/// Deletes a [`Webhook`] given its Id.
///
/// This method requires authentication, whereas [`delete_webhook_with_token`]
/// does not.
///
/// # Examples
///
/// Remove a webhook given its Id:
///
/// ```rust,no_run
/// use serenity::client::{Client, rest};
/// use std::env;
///
/// // Due to the `delete_webhook` function requiring you to authenticate, you
/// // must have initialized a client first.
/// let client = Client::login_user(&env::var("DISCORD_TOKEN").unwrap());
///
/// rest::delete_webhook(245037420704169985).expect("err deleting webhook");
/// ```
///
/// [`Webhook`]: ../../model/struct.Webhook.html
/// [`delete_webhook_with_token`]: fn.delete_webhook_with_token.html
pub fn delete_webhook(webhook_id: u64) -> Result<()> {
    verify(204, request!(Route::WebhooksId, delete, "/webhooks/{}", webhook_id))
}

/// Deletes a [`Webhook`] given its Id and unique token.
///
/// This method does _not_ require authentication.
///
/// # Examples
///
/// Remove a webhook given its Id and unique token:
///
/// ```rust,no_run
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
///
/// rest::delete_webhook_with_token(id, token).expect("err deleting webhook");
///
/// [`Webhook`]: ../../model/struct.Webhook.html
pub fn delete_webhook_with_token(webhook_id: u64, token: &str) -> Result<()> {
    let client = HyperClient::new();
    verify(204, try!(retry(|| client
        .delete(&format!(api!("/webhooks/{}/{}"), webhook_id, token)))
        .map_err(Error::Hyper)))
}

/// Changes channel information.
pub fn edit_channel(channel_id: u64, map: Value)
    -> Result<GuildChannel> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::ChannelsId(channel_id),
                            patch(body),
                            "/channels/{}",
                            channel_id);

    GuildChannel::decode(try!(serde_json::from_reader(response)))
}

/// Changes emoji information.
pub fn edit_emoji(guild_id: u64, emoji_id: u64, map: Value)
    -> Result<Emoji> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdEmojisId(guild_id),
                            patch(body),
                            "/guilds/{}/emojis/{}",
                            guild_id,
                            emoji_id);

    Emoji::decode(try!(serde_json::from_reader(response)))
}

/// Changes guild information.
pub fn edit_guild(guild_id: u64, map: Value) -> Result<PartialGuild> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsId(guild_id),
                            patch(body),
                            "/guilds/{}",
                            guild_id);

    PartialGuild::decode(try!(serde_json::from_reader(response)))
}

/// Does specific actions to a member.
pub fn edit_member(guild_id: u64, user_id: u64, map: Value)
    -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::GuildsIdMembersId(guild_id),
                         patch(body),
                         "/guilds/{}/members/{}",
                         guild_id,
                         user_id))
}

/// Changes message content, only if owned by us.
pub fn edit_message(channel_id: u64,
                    message_id: u64,
                    map: Value)
                    -> Result<Message> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::ChannelsIdMessagesId(channel_id),
                            patch(body),
                            "/channels/{}/messages/{}",
                            channel_id,
                            message_id);

    Message::decode(try!(serde_json::from_reader(response)))
}

/// Edits the current user's nickname for the provided [`Guild`] via its Id.
///
/// Pass `None` to reset the nickname.
///
/// [`Guild`]: ../../model/struct.Guild.html
pub fn edit_nickname(guild_id: u64, new_nickname: Option<&str>)
    -> Result<()> {
    let map = ObjectBuilder::new().insert("nick", new_nickname).build();
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdMembersMeNick(guild_id),
                            patch(body),
                            "/guilds/{}/members/@me/nick",
                            guild_id);

    verify(200, response)
}

/// Changes a profile note.
pub fn edit_note(user_id: u64, map: Value) -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::None,
                         put(body),
                         "/users/@me/notes/{}",
                         user_id))
}

/// Edits profile we're connected to.
pub fn edit_profile(map: Value) -> Result<CurrentUser> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::UsersMe, patch(body), "/users/@me");

    CurrentUser::decode(try!(serde_json::from_reader(response)))
}

/// Changes a role in a guild.
pub fn edit_role(guild_id: u64, role_id: u64, map: Value)
    -> Result<Role> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdRolesId(guild_id),
                            patch(body),
                            "/guilds/{}/roles/{}",
                            guild_id,
                            role_id);

    Role::decode(try!(serde_json::from_reader(response)))
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
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let image = serenity::utils::read_image("./webhook_img.png")
///     .expect("err reading image");
/// let map = ObjectBuilder::new().insert("avatar", image).build();
///
/// let edited = rest::edit_webhook_with_token(id, token, map)
///     .expect("err editing webhook");
/// ```
///
/// [`create_webhook`]: fn.create_webhook.html
/// [`edit_webhook_with_token`]: fn.edit_webhook_with_token.html
// The tests are ignored, rather than no_run'd, due to rustdoc tests with
// external crates being incredibly messy and misleading in the end user's view.
pub fn edit_webhook(webhook_id: u64, map: Value) -> Result<Webhook> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::WebhooksId,
                            patch(body),
                            "/webhooks/{}",
                            webhook_id);

    Webhook::decode(try!(serde_json::from_reader(response)))
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
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let map = ObjectBuilder::new().insert("name", "new name").build();
///
/// let edited = rest::edit_webhook_with_token(id, token, map)
///     .expect("err editing webhook");
/// ```
///
/// [`edit_webhook`]: fn.edit_webhook.html
pub fn edit_webhook_with_token(webhook_id: u64, token: &str, map: Value)
    -> Result<Webhook> {
    let body = try!(serde_json::to_string(&map));
    let client = HyperClient::new();
    let response = try!(retry(|| client
        .patch(&format!(api!("/webhooks/{}/{}"), webhook_id, token))
        .body(&body))
        .map_err(Error::Hyper));

    Webhook::decode(try!(serde_json::from_reader(response)))
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
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
/// let map = ObjectBuilder::new().insert("content", "test").build();
///
/// let message = match rest::execute_webhook(id, token, map) {
///     Ok(message) => message,
///     Err(why) => {
///         println!("Error executing webhook: {:?}", why);
///
///         return;
///     },
/// };
pub fn execute_webhook(webhook_id: u64, token: &str, map: Value)
    -> Result<Message> {
    let body = try!(serde_json::to_string(&map));
    let client = HyperClient::new();
    let response = try!(retry(|| client
        .patch(&format!(api!("/webhooks/{}/{}"), webhook_id, token))
        .body(&body))
        .map_err(Error::Hyper));

    Message::decode(try!(serde_json::from_reader(response)))
}

/// Get information about an oauth2 application we own, only for user accouts.
pub fn get_application_info() -> Result<CurrentApplicationInfo> {
    let response = request!(Route::None, get, "/oauth2/applications/@me");

    CurrentApplicationInfo::decode(try!(serde_json::from_reader(response)))
}

/// Get all oauth2 applications we've made, only for user accounts.
pub fn get_applications() -> Result<Vec<ApplicationInfo>> {
    let response = request!(Route::None, get, "/oauth2/applications");
    let decoded = try!(serde_json::from_reader(response));

    decode_array(decoded, ApplicationInfo::decode)
}

/// Get all the users that are banned in specific guild.
pub fn get_bans(guild_id: u64) -> Result<Vec<Ban>> {
    let response = request!(Route::GuildsIdBans(guild_id),
                            get,
                            "/guilds/{}/bans",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)), Ban::decode)
}

/// Gets current bot gateway.
pub fn get_bot_gateway() -> Result<BotGateway> {
    let response = request!(Route::GatewayBot, get, "/gateway/bot");

    BotGateway::decode(try!(serde_json::from_reader(response)))
}

/// Get all invites for a channel.
pub fn get_channel_invites(channel_id: u64)
    -> Result<Vec<RichInvite>> {
    let response = request!(Route::ChannelsIdInvites(channel_id),
                            get,
                            "/channels/{}/invites",
                            channel_id);

    decode_array(try!(serde_json::from_reader(response)),
                 RichInvite::decode)
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
/// use serenity::client::rest;
///
/// let channel_id = 81384788765712384;
///
/// let webhooks = rest::get_channel_webhooks(channel_id)
///     .expect("err getting channel webhooks");
/// ```
///
/// [`GuildChannel`]: ../../model/struct.GuildChannel.html
pub fn get_channel_webhooks(channel_id: u64) -> Result<Vec<Webhook>> {
    let response = request!(Route::ChannelsIdWebhooks(channel_id),
                            get,
                            "/channels/{}/webhooks",
                            channel_id);

    decode_array(try!(serde_json::from_reader(response)), Webhook::decode)
}

/// Gets channel information.
pub fn get_channel(channel_id: u64) -> Result<Channel> {
    let response = request!(Route::ChannelsId(channel_id),
                            get,
                            "/channels/{}",
                            channel_id);

    Channel::decode(try!(serde_json::from_reader(response)))
}

/// Gets all channels in a guild.
pub fn get_channels(guild_id: u64) -> Result<Vec<GuildChannel>> {
    let response = request!(Route::ChannelsId(guild_id),
                            get,
                            "/guilds/{}/channels",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)),
                 GuildChannel::decode)
}

/// Gets information about the user we're connected with.
pub fn get_current_user() -> Result<CurrentUser> {
    let response = request!(Route::UsersMe, get, "/users/@me");

    CurrentUser::decode(try!(serde_json::from_reader(response)))
}

/// Gets current gateway.
pub fn get_gateway() -> Result<Gateway> {
    let response = request!(Route::Gateway, get, "/gateway");

    Gateway::decode(try!(serde_json::from_reader(response)))
}

/// Gets information about an emoji.
pub fn get_emoji(guild_id: u64, emoji_id: u64) -> Result<Emoji> {
    let response = request!(Route::GuildsIdEmojisId(guild_id),
                            get,
                            "/guilds/{}/emojis/{}",
                            guild_id,
                            emoji_id);

    Emoji::decode(try!(serde_json::from_reader(response)))
}

/// Gets all emojis in a guild.
pub fn get_emojis(guild_id: u64) -> Result<Vec<Emoji>> {
    let response = request!(Route::GuildsIdEmojis(guild_id),
                            get,
                            "/guilds/{}/emojis",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)), Emoji::decode)
}

/// Gets guild information.
pub fn get_guild(guild_id: u64) -> Result<PartialGuild> {
    let response = request!(Route::GuildsId(guild_id),
                            get,
                            "/guilds/{}",
                            guild_id);

    PartialGuild::decode(try!(serde_json::from_reader(response)))
}

/// Gets a guild embed information.
pub fn get_guild_embed(guild_id: u64) -> Result<GuildEmbed> {
    let response = request!(Route::GuildsIdEmbed(guild_id),
                            get,
                            "/guilds/{}/embeds",
                            guild_id);

    GuildEmbed::decode(try!(serde_json::from_reader(response)))
}

/// Gets integrations that a guild has.
pub fn get_guild_integrations(guild_id: u64) -> Result<Vec<Integration>> {
    let response = request!(Route::GuildsIdIntegrations(guild_id),
                            get,
                            "/guilds/{}/integrations",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)), Integration::decode)
}

/// Gets all invites to a guild.
pub fn get_guild_invites(guild_id: u64) -> Result<Vec<RichInvite>> {
    let response = request!(Route::GuildsIdInvites(guild_id),
                            get,
                            "/guilds/{}/invites",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)),
                 RichInvite::decode)
}

/// Get the amount of users that can be pruned.
pub fn get_guild_prune_count(guild_id: u64, map: Value)
    -> Result<GuildPrune> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdPrune(guild_id),
                            get(body),
                            "/guilds/{}/prune",
                            guild_id);

    GuildPrune::decode(try!(serde_json::from_reader(response)))
}

/// Gets regions that a guild can use, if a guild is VIP shows more regions :o
pub fn get_guild_regions(guild_id: u64) -> Result<Vec<VoiceRegion>> {
    let response = request!(Route::GuildsIdRegions(guild_id),
                            get,
                            "/guilds/{}/regions",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)), VoiceRegion::decode)
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
/// use serenity::client::rest;
///
/// let guild_id = 81384788765712384;
///
/// let webhooks = rest::get_guild_webhooks(guild_id)
///     .expect("err getting guild webhooks");
/// ```
///
/// [`Guild`]: ../../model/struct.Guild.html
pub fn get_guild_webhooks(guild_id: u64) -> Result<Vec<Webhook>> {
    let response = request!(Route::GuildsIdWebhooks(guild_id),
                            get,
                            "/guilds/{}/webhooks",
                            guild_id);

    decode_array(try!(serde_json::from_reader(response)), Webhook::decode)
}

/// Gets all guilds we're connected to.
pub fn get_guilds() -> Result<Vec<GuildInfo>> {
    let response = request!(Route::UsersMeGuilds,
                            get,
                            "/users/@me/guilds");

    decode_array(try!(serde_json::from_reader(response)), GuildInfo::decode)
}

/// Gets information about a specific invite.
pub fn get_invite(code: &str) -> Result<Invite> {
    let invite = ::utils::parse_invite(code);
    let response = request!(Route::InvitesCode, get, "/invite/{}", invite);

    Invite::decode(try!(serde_json::from_reader(response)))
}

/// Gets member of a guild.
pub fn get_member(guild_id: u64, user_id: u64) -> Result<Member> {
    let response = request!(Route::GuildsIdMembersId(guild_id),
                            get,
                            "/guilds/{}/members/{}",
                            guild_id,
                            user_id);

    Member::decode(try!(serde_json::from_reader(response)))
}

/// Gets a message by an Id, bots only.
pub fn get_message(channel_id: u64, message_id: u64)
    -> Result<Message> {
    let response = request!(Route::ChannelsIdMessagesId(channel_id),
                            get,
                            "/channels/{}/messages/{}",
                            channel_id,
                            message_id);

    Message::decode(try!(serde_json::from_reader(response)))
}

/// Gets X messages from a channel.
pub fn get_messages(channel_id: u64, query: &str)
    -> Result<Vec<Message>> {
    let url = format!(api_concat!("/channels/{}/messages{}"),
                      channel_id,
                      query);
    let client = HyperClient::new();
    let response = try!(request(Route::ChannelsIdMessages(channel_id),
                                || client.get(&url)));

    decode_array(try!(serde_json::from_reader(response)), Message::decode)
}

/// Gets all pins of a channel.
pub fn get_pins(channel_id: u64) -> Result<Vec<Message>> {
    let response = request!(Route::ChannelsIdPins(channel_id),
                            get,
                            "/channels/{}/pins",
                            channel_id);

    decode_array(try!(serde_json::from_reader(response)), Message::decode)
}

/// Get user Ids based on their reaction to a message. This endpoint is dumb.
pub fn get_reaction_users(channel_id: u64,
                          message_id: u64,
                          reaction_type: ReactionType,
                          limit: u8,
                          after: Option<u64>)
                          -> Result<Vec<User>> {
    let mut uri = format!("/channels/{}/messages/{}/reactions/{}?limit={}",
                      channel_id,
                      message_id,
                      reaction_type.as_data(),
                      limit);

    if let Some(user_id) = after {
        uri.push_str("&after=");
        uri.push_str(&user_id.to_string());
    }

    let response = request!(Route::ChannelsIdMessagesIdReactionsUserIdType(channel_id),
                            get,
                            "{}",
                            uri);

    decode_array(try!(serde_json::from_reader(response)), User::decode)
}

/// Gets a user by Id.
pub fn get_user(user_id: u64) -> Result<CurrentUser> {
    let response = request!(Route::UsersId, get, "/users/{}", user_id);

    CurrentUser::decode(try!(serde_json::from_reader(response)))
}

/// Gets our connections.
pub fn get_user_connections() -> Result<Vec<UserConnection>> {
    let response = request!(Route::UsersMeConnections,
                            get,
                            "/users/@me/connections");

    decode_array(try!(serde_json::from_reader(response)),
                      UserConnection::decode)
}

/// Gets our DM channels.
pub fn get_user_dm_channels() -> Result<Vec<PrivateChannel>> {
    let response = request!(Route::UsersMeChannels, get, "/users/@me/channels");

    decode_array(try!(serde_json::from_reader(response)),
                      PrivateChannel::decode)
}

/// Gets all voice regions.
pub fn get_voice_regions() -> Result<Vec<VoiceRegion>> {
    let response = request!(Route::VoiceRegions, get, "/voice/regions");

    decode_array(try!(serde_json::from_reader(response)), VoiceRegion::decode)
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
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let webhook = rest::get_webhook(id).expect("err getting webhook");
/// ```
///
/// [`get_webhook_with_token`]: fn.get_webhook_with_token.html
pub fn get_webhook(webhook_id: u64) -> Result<Webhook> {
    let response = request!(Route::WebhooksId, get, "/webhooks/{}", webhook_id);

    Webhook::decode(try!(serde_json::from_reader(response)))
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
/// use serenity::client::rest;
///
/// let id = 245037420704169985;
/// let token = "ig5AO-wdVWpCBtUUMxmgsWryqgsW3DChbKYOINftJ4DCrUbnkedoYZD0VOH1QLr-S3sV";
///
/// let webhook = rest::get_webhook_with_token(id, token)
///     .expect("err getting webhook");
/// ```
pub fn get_webhook_with_token(webhook_id: u64, token: &str) -> Result<Webhook> {
    let client = HyperClient::new();
    let response = try!(retry(|| client
        .get(&format!(api!("/webhooks/{}/{}"), webhook_id, token)))
        .map_err(Error::Hyper));

    Webhook::decode(try!(serde_json::from_reader(response)))
}

/// Kicks a member from a guild.
pub fn kick_member(guild_id: u64, user_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdMembersId(guild_id),
                         delete,
                         "/guilds/{}/members/{}",
                         guild_id,
                         user_id))
}

/// Leaves a group DM.
pub fn leave_group(guild_id: u64) -> Result<Group> {
    let response = request!(Route::None,
                            delete,
                            "/channels/{}",
                            guild_id);

    Group::decode(try!(serde_json::from_reader(response)))
}

/// Leaves a guild.
pub fn leave_guild(guild_id: u64) -> Result<PartialGuild> {
    let response = request!(Route::UsersMeGuildsId,
                            delete,
                            "/users/@me/guilds/{}",
                            guild_id);

    PartialGuild::decode(try!(serde_json::from_reader(response)))
}

/// Logs out. That's supposed to disable the token but doesn't.
pub fn logout(map: Value) -> Result<()> {
    let body = try!(serde_json::to_string(&map));

    verify(204, request!(Route::None, post(body), "/auth/logout"))
}

/// Removes a user from group DM.
pub fn remove_group_recipient(group_id: u64, user_id: u64)
    -> Result<()> {
    verify(204, request!(Route::None,
                         delete,
                         "/channels/{}/recipients/{}",
                         group_id,
                         user_id))
}

/// Sends a file to a channel.
pub fn send_file<R: Read>(channel_id: u64,
                          content: &str,
                          mut file: R,
                          filename: &str)
                          -> Result<Message> {
    let uri = format!(api_concat!("/channels/{}/messages"), channel_id);
    let url = match Url::parse(&uri) {
        Ok(url) => url,
        Err(_why) => return Err(Error::Url(uri)),
    };

    let mut request = try!(Request::new(Method::Post, url));
    request.headers_mut().set(header::Authorization(TOKEN.lock().unwrap().clone()));
    request.headers_mut()
        .set(header::UserAgent(constants::USER_AGENT.to_owned()));

    let mut request = try!(Multipart::from_request(request));
    try!(request.write_text("content", content));
    try!(request.write_stream("file", &mut file, Some(&filename), None));

    Message::decode(try!(serde_json::from_reader(try!(request.send()))))
}

/// Sends a message to a channel.
pub fn send_message(channel_id: u64, map: Value) -> Result<Message> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::ChannelsIdMessages(channel_id),
                            post(body),
                            "/channels/{}/messages",
                            channel_id);

    Message::decode(try!(serde_json::from_reader(response)))
}

/// Pins a message in a channel.
pub fn pin_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(204, request!(Route::ChannelsIdPinsMessageId(channel_id),
                         put,
                         "/channels/{}/pins/{}",
                         channel_id,
                         message_id))
}

/// Unbans a user from a guild.
pub fn remove_ban(guild_id: u64, user_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdBansUserId(guild_id),
                         delete,
                         "/guilds/{}/bans/{}",
                         guild_id,
                         user_id))
}

/// Removes a single [`Role`] from a [`Member`] in a [`Guild`].
///
/// **Note**: Requires the [Manage Roles] permission and respect of role
/// hierarchy.
///
/// [`Guild`]: ../../model/struct.Guild.html
/// [`Member`]: ../../model/struct.Member.html
/// [`Role`]: ../../model/struct.Role.html
/// [Manage Roles]: ../../model/permissions/constant.MANAGE_ROLES.html
pub fn remove_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    verify(204, request!(Route::GuildsIdMembersIdRolesId(guild_id),
                         delete,
                         "/guilds/{}/members/{}/roles/{}",
                         guild_id,
                         user_id,
                         role_id))
}

/// Starts removing some members from a guild based on the last time they've been online.
pub fn start_guild_prune(guild_id: u64, map: Value)
    -> Result<GuildPrune> {
    let body = try!(serde_json::to_string(&map));
    let response = request!(Route::GuildsIdPrune(guild_id),
                            post(body),
                            "/guilds/{}/prune",
                            guild_id);

    GuildPrune::decode(try!(serde_json::from_reader(response)))
}

/// Adds an integration to a guild.
pub fn start_integration_sync(guild_id: u64, integration_id: u64)
    -> Result<()> {
    verify(204, request!(Route::GuildsIdIntegrationsIdSync(guild_id),
                         post,
                         "/guilds/{}/integrations/{}",
                         guild_id,
                         integration_id))
}

/// Unpins a message from a channel.
pub fn unpin_message(channel_id: u64, message_id: u64) -> Result<()> {
    verify(204, request!(Route::ChannelsIdPinsMessageId(channel_id),
                         delete,
                         "/channels/{}/pins/{}",
                         channel_id,
                         message_id))
}

/// Does a rest request.
fn request<'a, F>(route: Route, f: F) -> Result<HyperResponse>
    where F: Fn() -> RequestBuilder<'a> {
    ratelimiting::perform(route, || f()
        .header(header::Authorization(TOKEN.lock().unwrap().clone()))
        .header(header::ContentType::json()))
}

#[doc(hidden)]
pub fn retry<'a, F>(f: F) -> HyperResult<HyperResponse>
    where F: Fn() -> RequestBuilder<'a> {
    let req = || f()
        .header(header::UserAgent(constants::USER_AGENT.to_owned()))
        .send();

    match req() {
        Err(HyperError::Io(ref io))
            if io.kind() == IoErrorKind::ConnectionAborted => req(),
        other => other,
    }
}

/// Verifies a request's response.
fn verify(expected_status_code: u16,
          mut response: HyperResponse)
          -> Result<()> {
    let expected_status = match expected_status_code {
        200 => StatusCode::Ok,
        204 => StatusCode::NoContent,
        401 => StatusCode::Unauthorized,
        _ => {
            let client_error = ClientError::UnknownStatus(expected_status_code);

            return Err(Error::Client(client_error));
        },
    };

    if response.status == expected_status {
        return Ok(());
    }

    debug!("Expected {}, got {}", expected_status_code, response.status);

    let mut s = String::default();
    try!(response.read_to_string(&mut s));

    debug!("Content: {}", s);

    Err(Error::Client(ClientError::UnexpectedStatusCode(response.status)))
}
