use constants;
use hyper::{
    client::{
        Request as HyperRequest,
        Response as HyperResponse
    },
    header::{ContentType, Headers},
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
use super::{
    TOKEN,
    ratelimiting,
    request::Request,
    routing::RouteInfo,
    AttachmentType,
    GuildPagination,
    HttpError,
    StatusClass,
    StatusCode,
};
use serde::de::DeserializeOwned;
use serde_json;
use std::{
    collections::BTreeMap,
    io::ErrorKind as IoErrorKind,
};

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
/// [`Group`]: ../../model/channel/struct.Group.html
/// [`Group::add_recipient`]: ../../model/channel/struct.Group.html#method.add_recipient
/// [`User`]: ../../model/user/struct.User.html
pub fn add_group_recipient(group_id: u64, user_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::AddGroupRecipient { group_id, user_id },
    })
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
pub fn add_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::AddMemberRole { guild_id, role_id, user_id },
    })
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
pub fn ban_user(guild_id: u64, user_id: u64, delete_message_days: u8, reason: &str) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::GuildBanUser {
            delete_message_days: Some(delete_message_days),
            reason: Some(reason),
            guild_id,
            user_id,
        },
    })
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
/// [Ban Members]: ../model/permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
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
/// [Ban Members]: ../model/permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
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
/// [Ban Members]: ../model/permissions/struct.Permissions.html#associatedconstant.BAN_MEMBERS
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
/// [`Channel`]: ../../model/channel/enum.Channel.html
pub fn broadcast_typing(channel_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::BroadcastTyping { channel_id },
    })
}

/// Creates a [`GuildChannel`] in the [`Guild`] given its Id.
///
/// Refer to the Discord's [docs] for information on what fields this requires.
///
/// **Note**: Requires the [Manage Channels] permission.
///
/// [`Guild`]: ../../model/guild/struct.Guild.html
/// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
/// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-channel
/// [Manage Channels]: ../../model/permissions/struct.Permissions.html#associatedconstant.MANAGE_CHANNELS
pub fn create_channel(guild_id: u64, map: &Value) -> Result<GuildChannel> {
    fire(Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::CreateChannel { guild_id },
    })
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
pub fn create_emoji(guild_id: u64, map: &Value) -> Result<Emoji> {
    fire(Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::CreateEmoji { guild_id },
    })
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
/// [`Guild`]: ../../model/guild/struct.Guild.html
/// [`PartialGuild`]: ../../model/guild/struct.PartialGuild.html
/// [`Shard`]: ../../gateway/struct.Shard.html
/// [GameBridge]: https://discordapp.com/developers/docs/topics/gamebridge
/// [US West Region]: ../../model/guild/enum.Region.html#variant.UsWest
/// [documentation on this endpoint]:
/// https://discordapp.com/developers/docs/resources/guild#create-guild
/// [whitelist]: https://discordapp.com/developers/docs/resources/guild#create-guild
pub fn create_guild(map: &Value) -> Result<PartialGuild> {
    fire(Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::CreateGuild,
    })
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
/// [docs]: https://discordapp.com/developers/docs/resources/guild#create-guild-integration
pub fn create_guild_integration(guild_id: u64, integration_id: u64, map: &Value) -> Result<()> {
    wind(204, Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::CreateGuildIntegration { guild_id, integration_id },
    })
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
/// [docs]: https://discordapp.com/developers/docs/resources/channel#create-channel-invite
pub fn create_invite(channel_id: u64, map: &JsonMap) -> Result<RichInvite> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreateInvite { channel_id },
    })
}

/// Creates a permission override for a member or a role in a channel.
pub fn create_permission(channel_id: u64, target_id: u64, map: &Value) -> Result<()> {
    let body = serde_json::to_vec(map)?;

    wind(204, Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreatePermission { channel_id, target_id },
    })
}

/// Creates a private channel with a user.
pub fn create_private_channel(map: &Value) -> Result<PrivateChannel> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreatePrivateChannel,
    })
}

/// Reacts to a message.
pub fn create_reaction(channel_id: u64,
                       message_id: u64,
                       reaction_type: &ReactionType)
                       -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::CreateReaction {
            reaction: &reaction_type.as_data(),
            channel_id,
            message_id,
        },
    })
}

/// Creates a role.
pub fn create_role(guild_id: u64, map: &JsonMap) -> Result<Role> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreateRole {guild_id },
    })
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
/// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
pub fn create_webhook(channel_id: u64, map: &Value) -> Result<Webhook> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreateWebhook { channel_id },
    })
}

/// Deletes a private channel or a channel in a guild.
pub fn delete_channel(channel_id: u64) -> Result<Channel> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteChannel { channel_id },
    })
}

/// Deletes an emoji from a server.
pub fn delete_emoji(guild_id: u64, emoji_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteEmoji { guild_id, emoji_id },
    })
}

/// Deletes a guild, only if connected account owns it.
pub fn delete_guild(guild_id: u64) -> Result<PartialGuild> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteGuild { guild_id },
    })
}

/// Removes an integration from a guild.
pub fn delete_guild_integration(guild_id: u64, integration_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteGuildIntegration { guild_id, integration_id },
    })
}

/// Deletes an invite by code.
pub fn delete_invite(code: &str) -> Result<Invite> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteInvite { code },
    })
}

/// Deletes a message if created by us or we have
/// specific permissions.
pub fn delete_message(channel_id: u64, message_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteMessage { channel_id, message_id },
    })
}

/// Deletes a bunch of messages, only works for bots.
pub fn delete_messages(channel_id: u64, map: &Value) -> Result<()> {
    wind(204, Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::DeleteMessages { channel_id },
    })
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
/// [`Message`]: ../../model/channel/struct.Message.html
/// [`Reaction`]: ../../model/channel/struct.Reaction.html
pub fn delete_message_reactions(channel_id: u64, message_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteMessageReactions { channel_id, message_id },
    })
}

/// Deletes a permission override from a role or a member in a channel.
pub fn delete_permission(channel_id: u64, target_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeletePermission { channel_id, target_id },
    })
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

    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteReaction {
            reaction: &reaction_type.as_data(),
            user: &user,
            channel_id,
            message_id,
        },
    })
}

/// Deletes a role from a server. Can't remove the default everyone role.
pub fn delete_role(guild_id: u64, role_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteRole { guild_id, role_id },
    })
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
/// use serenity::http;
/// use std::env;
///
/// // Due to the `delete_webhook` function requiring you to authenticate, you
/// // must have set the token first.
/// http::set_token(&env::var("DISCORD_TOKEN").unwrap());
///
/// http::delete_webhook(245037420704169985).expect("Error deleting webhook");
/// ```
///
/// [`Webhook`]: ../../model/webhook/struct.Webhook.html
/// [`delete_webhook_with_token`]: fn.delete_webhook_with_token.html
pub fn delete_webhook(webhook_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteWebhook { webhook_id },
    })
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
/// [`Webhook`]: ../../model/webhook/struct.Webhook.html
pub fn delete_webhook_with_token(webhook_id: u64, token: &str) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::DeleteWebhookWithToken { token, webhook_id },
    })
}

/// Changes channel information.
pub fn edit_channel(channel_id: u64, map: &JsonMap) -> Result<GuildChannel> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditChannel {channel_id },
    })
}

/// Changes emoji information.
pub fn edit_emoji(guild_id: u64, emoji_id: u64, map: &Value) -> Result<Emoji> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditEmoji { guild_id, emoji_id },
    })
}

/// Changes guild information.
pub fn edit_guild(guild_id: u64, map: &JsonMap) -> Result<PartialGuild> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditGuild { guild_id },
    })
}

/// Edits the positions of a guild's channels.
pub fn edit_guild_channel_positions(guild_id: u64, value: &Value)
                                    -> Result<()> {
    let body = serde_json::to_vec(value)?;

    wind(204, Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditGuildChannels { guild_id },
    })
}

/// Edits a [`Guild`]'s embed setting.
///
/// [`Guild`]: ../../model/guild/struct.Guild.html
pub fn edit_guild_embed(guild_id: u64, map: &Value) -> Result<GuildEmbed> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditGuildEmbed { guild_id },
    })
}

/// Does specific actions to a member.
pub fn edit_member(guild_id: u64, user_id: u64, map: &JsonMap) -> Result<()> {
    let body = serde_json::to_vec(map)?;

    wind(204, Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditMember { guild_id, user_id },
    })
}

/// Edits a message by Id.
///
/// **Note**: Only the author of a message can modify it.
pub fn edit_message(channel_id: u64, message_id: u64, map: &Value) -> Result<Message> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditMessage { channel_id, message_id },
    })
}

/// Edits the current user's nickname for the provided [`Guild`] via its Id.
///
/// Pass `None` to reset the nickname.
///
/// [`Guild`]: ../../model/guild/struct.Guild.html
pub fn edit_nickname(guild_id: u64, new_nickname: Option<&str>) -> Result<()> {
    let map = json!({ "nick": new_nickname });
    let body = serde_json::to_vec(&map)?;

    wind(200, Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditNickname { guild_id },
    })
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
    let body = serde_json::to_vec(map)?;

    let response = request(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditProfile,
    })?;

    let mut value = serde_json::from_reader::<HyperResponse, Value>(response)?;

    if let Some(map) = value.as_object_mut() {
        if !TOKEN.lock().starts_with("Bot ") {
            if let Some(Value::String(token)) = map.remove("token") {
                set_token(&token);
            }
        }
    }

    serde_json::from_value::<CurrentUser>(value).map_err(From::from)
}

/// Changes a role in a guild.
pub fn edit_role(guild_id: u64, role_id: u64, map: &JsonMap) -> Result<Role> {
    let body = serde_json::to_vec(&map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditRole { guild_id, role_id },
    })
}

/// Changes the position of a role in a guild.
pub fn edit_role_position(guild_id: u64, role_id: u64, position: u64) -> Result<Vec<Role>> {
    let body = serde_json::to_vec(&json!({
        "id": role_id,
        "position": position,
    }))?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditRole { guild_id, role_id },
    })
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
    fire(Request {
        body: Some(map.to_string().as_bytes()),
        headers: None,
        route: RouteInfo::EditWebhook { webhook_id },
    })
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
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::EditWebhookWithToken { token, webhook_id },
    })
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
/// [`Channel`]: ../../model/channel/enum.Channel.html
/// [`Message`]: ../../model/channel/struct.Message.html
/// [Discord docs]: https://discordapp.com/developers/docs/resources/webhook#querystring-params
pub fn execute_webhook(webhook_id: u64,
                       token: &str,
                       wait: bool,
                       map: &JsonMap)
                       -> Result<Option<Message>> {
    let body = serde_json::to_vec(map)?;

    let mut headers = Headers::new();
    headers.set(ContentType(
        Mime(TopLevel::Application, SubLevel::Json, vec![]),
    ));

    let response = request(Request {
        body: Some(&body),
        headers: Some(headers),
        route: RouteInfo::ExecuteWebhook { token, wait, webhook_id },
    })?;

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
    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetActiveMaintenance,
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
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetBans { guild_id },
    })
}

/// Gets all audit logs in a specific guild.
pub fn get_audit_logs(guild_id: u64,
                      action_type: Option<u8>,
                      user_id: Option<u64>,
                      before: Option<u64>,
                      limit: Option<u8>) -> Result<AuditLogs> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetAuditLogs {
            action_type,
            before,
            guild_id,
            limit,
            user_id,
        },
    })
}

/// Gets current bot gateway.
pub fn get_bot_gateway() -> Result<BotGateway> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetBotGateway,
    })
}

/// Gets all invites for a channel.
pub fn get_channel_invites(channel_id: u64) -> Result<Vec<RichInvite>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetChannelInvites { channel_id },
    })
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
/// [`GuildChannel`]: ../../model/channel/struct.GuildChannel.html
pub fn get_channel_webhooks(channel_id: u64) -> Result<Vec<Webhook>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetChannelWebhooks { channel_id },
    })
}

/// Gets channel information.
pub fn get_channel(channel_id: u64) -> Result<Channel> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetChannel { channel_id },
    })
}

/// Gets all channels in a guild.
pub fn get_channels(guild_id: u64) -> Result<Vec<GuildChannel>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetChannels { guild_id },
    })
}

/// Gets information about the current application.
///
/// **Note**: Only applications may use this endpoint.
pub fn get_current_application_info() -> Result<CurrentApplicationInfo> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetCurrentApplicationInfo,
    })
}

/// Gets information about the user we're connected with.
pub fn get_current_user() -> Result<CurrentUser> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetCurrentUser,
    })
}

/// Gets current gateway.
pub fn get_gateway() -> Result<Gateway> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGateway,
    })
}

/// Gets guild information.
pub fn get_guild(guild_id: u64) -> Result<PartialGuild> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuild { guild_id },
    })
}

/// Gets a guild embed information.
pub fn get_guild_embed(guild_id: u64) -> Result<GuildEmbed> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildEmbed { guild_id },
    })
}

/// Gets integrations that a guild has.
pub fn get_guild_integrations(guild_id: u64) -> Result<Vec<Integration>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildIntegrations { guild_id },
    })
}

/// Gets all invites to a guild.
pub fn get_guild_invites(guild_id: u64) -> Result<Vec<RichInvite>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildInvites { guild_id },
    })
}

/// Gets a guild's vanity URL if it has one.
pub fn get_guild_vanity_url(guild_id: u64) -> Result<String> {
    #[derive(Deserialize)]
    struct GuildVanityUrl {
        code: String,
    }

    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildVanityUrl { guild_id },
    })?;

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
    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildMembers { after, guild_id, limit },
    })?;

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
    // Note for 0.6.x: turn this into a function parameter.
    #[derive(Deserialize)]
    struct GetGuildPruneCountRequest {
        days: u64,
    }

    let req = serde_json::from_value::<GetGuildPruneCountRequest>(map.clone())?;

    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildPruneCount {
            days: req.days,
            guild_id,
        },
    })
}

/// Gets regions that a guild can use. If a guild has the `VIP_REGIONS` feature
/// enabled, then additional VIP-only regions are returned.
pub fn get_guild_regions(guild_id: u64) -> Result<Vec<VoiceRegion>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildRegions { guild_id },
    })
}

/// Retrieves a list of roles in a [`Guild`].
///
/// [`Guild`]: ../../model/guild/struct.Guild.html
pub fn get_guild_roles(guild_id: u64) -> Result<Vec<Role>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildRoles { guild_id },
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
/// [`Guild`]: ../../model/guild/struct.Guild.html
pub fn get_guild_webhooks(guild_id: u64) -> Result<Vec<Webhook>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuildWebhooks { guild_id },
    })
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
    let (after, before) = match *target {
        GuildPagination::After(id) => (Some(id.0), None),
        GuildPagination::Before(id) => (None, Some(id.0)),
    };

    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetGuilds { after, before, limit },
    })
}

/// Gets information about a specific invite.
#[allow(unused_mut)]
pub fn get_invite(mut code: &str, stats: bool) -> Result<Invite> {
    #[cfg(feature = "utils")]
        {
            code = ::utils::parse_invite(code);
        }

    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetInvite { code, stats },
    })
}

/// Gets member of a guild.
pub fn get_member(guild_id: u64, user_id: u64) -> Result<Member> {
    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetMember { guild_id, user_id },
    })?;

    let mut v = serde_json::from_reader::<HyperResponse, Value>(response)?;

    if let Some(map) = v.as_object_mut() {
        map.insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
    }

    serde_json::from_value::<Member>(v).map_err(From::from)
}

/// Gets a message by an Id, bots only.
pub fn get_message(channel_id: u64, message_id: u64) -> Result<Message> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetMessage { channel_id, message_id },
    })
}

/// Gets X messages from a channel.
pub fn get_messages(channel_id: u64, query: &str) -> Result<Vec<Message>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetMessages {
            query: query.to_owned(),
            channel_id,
        },
    })
}

/// Gets all pins of a channel.
pub fn get_pins(channel_id: u64) -> Result<Vec<Message>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetPins { channel_id },
    })
}

/// Gets user Ids based on their reaction to a message. This endpoint is dumb.
pub fn get_reaction_users(channel_id: u64,
                          message_id: u64,
                          reaction_type: &ReactionType,
                          limit: u8,
                          after: Option<u64>)
                          -> Result<Vec<User>> {
    let reaction = reaction_type.as_data();

    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetReactionUsers {
            after,
            channel_id,
            limit,
            message_id,
            reaction,
        },
    })
}

/// Gets the current unresolved incidents from Discord's Status API.
///
/// Does not require authentication.
pub fn get_unresolved_incidents() -> Result<Vec<Incident>> {
    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetUnresolvedIncidents,
    })?;

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
    let response = request(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetUpcomingMaintenances,
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
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetUser { user_id },
    })
}

/// Gets our DM channels.
pub fn get_user_dm_channels() -> Result<Vec<PrivateChannel>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetUserDmChannels,
    })
}

/// Gets all voice regions.
pub fn get_voice_regions() -> Result<Vec<VoiceRegion>> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetVoiceRegions,
    })
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
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetWebhook { webhook_id },
    })
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
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::GetWebhookWithToken { token, webhook_id },
    })
}

/// Kicks a member from a guild.
pub fn kick_member(guild_id: u64, user_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::KickMember { guild_id, user_id },
    })
}

/// Leaves a group DM.
pub fn leave_group(group_id: u64) -> Result<Group> {
    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::LeaveGroup { group_id },
    })
}

/// Leaves a guild.
pub fn leave_guild(guild_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::LeaveGuild { guild_id },
    })
}

/// Deletes a user from group DM.
pub fn remove_group_recipient(group_id: u64, user_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::RemoveGroupRecipient { group_id, user_id },
    })
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
    let uri = api!("/channels/{}/messages", channel_id);
    let url = match Url::parse(&uri) {
        Ok(url) => url,
        Err(_) => return Err(Error::Url(uri)),
    };

    let tc = NativeTlsClient::new()?;
    let connector = HttpsConnector::new(tc);
    let mut request = HyperRequest::with_connector(Method::Post, url, &connector)?;
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

    serde_json::from_reader(response).map_err(From::from)
}

/// Sends a message to a channel.
pub fn send_message(channel_id: u64, map: &Value) -> Result<Message> {
    let body = serde_json::to_vec(map)?;

    fire(Request {
        body: Some(&body),
        headers: None,
        route: RouteInfo::CreateMessage { channel_id },
    })
}

/// Pins a message in a channel.
pub fn pin_message(channel_id: u64, message_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::PinMessage { channel_id, message_id },
    })
}

/// Unbans a user from a guild.
pub fn remove_ban(guild_id: u64, user_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::RemoveBan { guild_id, user_id },
    })
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
pub fn remove_member_role(guild_id: u64, user_id: u64, role_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::RemoveMemberRole { guild_id, user_id, role_id },
    })
}

/// Starts removing some members from a guild based on the last time they've been online.
pub fn start_guild_prune(guild_id: u64, map: &Value) -> Result<GuildPrune> {
    // Note for 0.6.x: turn this into a function parameter.
    #[derive(Deserialize)]
    struct StartGuildPruneRequest {
        days: u64,
    }

    let req = serde_json::from_value::<StartGuildPruneRequest>(map.clone())?;

    fire(Request {
        body: None,
        headers: None,
        route: RouteInfo::StartGuildPrune {
            days: req.days,
            guild_id,
        },
    })
}

/// Starts syncing an integration with a guild.
pub fn start_integration_sync(guild_id: u64, integration_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::StartIntegrationSync { guild_id, integration_id },
    })
}

/// Unpins a message from a channel.
pub fn unpin_message(channel_id: u64, message_id: u64) -> Result<()> {
    wind(204, Request {
        body: None,
        headers: None,
        route: RouteInfo::UnpinMessage { channel_id, message_id },
    })
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
/// # extern crate serenity;
/// #
/// # use std::error::Error;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// #
/// use serenity::{
///     http::{
///         self,
///         request::RequestBuilder,
///         routing::RouteInfo,
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
/// let message = http::fire::<Message>(request.build())?;
///
/// println!("Message content: {}", message.content);
/// #
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
///
/// [`request`]: fn.request.html
pub fn fire<T: DeserializeOwned>(req: Request) -> Result<T> {
    let response = request(req)?;

    serde_json::from_reader(response).map_err(From::from)
}

/// Performs a request, ratelimiting it if necessary.
///
/// Returns the raw hyper Response. Use [`fire`] to deserialize the response
/// into some type.
///
/// # Examples
///
/// Send a body of bytes over the [`RouteInfo::CreateMessage`] endpoint:
///
/// ```rust,no_run
/// # extern crate serenity;
/// #
/// # use std::error::Error;
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// #
/// use serenity::http::{
///     self,
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
/// let response = http::request(request.build())?;
///
/// println!("Response successful?: {}", response.status.is_success());
/// #
/// #     Ok(())
/// # }
/// #
/// # fn main() {
/// #     try_main().unwrap();
/// # }
/// ```
///
/// [`fire`]: fn.fire.html
pub fn request(req: Request) -> Result<HyperResponse> {
    let response = ratelimiting::perform(req)?;

    if response.status.class() == StatusClass::Success {
        Ok(response)
    } else {
        Err(Error::Http(HttpError::UnsuccessfulRequest(response)))
    }
}

pub(super) fn retry(request: &Request) -> HyperResult<HyperResponse> {
    // Retry the request twice in a loop until it succeeds.
    //
    // If it doesn't and the loop breaks, try one last time.
    for _ in 0..3 {
        match request.build().send() {
            Err(HyperError::Io(ref io))
            if io.kind() == IoErrorKind::ConnectionAborted => continue,
            other => return other,
        }
    }

    request.build().send()
}

/// Performs a request and then verifies that the response status code is equal
/// to the expected value.
///
/// This is a function that performs a light amount of work and returns an
/// empty tuple, so it's called "wind" to denote that it's lightweight.
pub(super) fn wind(expected: u16, req: Request) -> Result<()> {
    let resp = request(req)?;

    if resp.status.to_u16() == expected {
        return Ok(());
    }

    debug!("Expected {}, got {}", expected, resp.status);
    trace!("Unsuccessful response: {:?}", resp);

    Err(Error::Http(HttpError::UnsuccessfulRequest(resp)))
}
