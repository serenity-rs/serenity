use hyper::status::StatusCode;
use ::model::{ChannelType, Permissions};

/// An error returned from the [`Client`] or the [`Context`], or model instance.
///
/// This is always wrapped within the library's generic [`Error::Client`]
/// variant.
///
/// # Examples
///
/// Matching an [`Error`] with this variant may look something like the
/// following for the [`Client::ban`] method, which in this example is used to
/// re-ban all members with an odd discriminator:
///
/// ```rust,no_run
/// use serenity::client::{Client, ClientError};
/// use serenity::Error;
/// use std::env;
///
/// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
/// let mut client = Client::login_bot(&token);
///
/// client.on_member_unban(|context, guild_id, user| {
///     let discriminator = match user.discriminator.parse::<u16>() {
///         Ok(discriminator) => discriminator,
///         Err(_why) => return,
///     };
///
///     // If the user has an even discriminator, don't re-ban them.
///     if discriminator % 2 == 0 {
///         return;
///     }
///
///     match context.ban(guild_id, user, 8) {
///         Ok(()) => {
///             // Ban successful.
///         },
///         Err(Error::Client(ClientError::DeleteMessageDaysAmount(amount))) => {
///             println!("Failed deleting {} days' worth of messages", amount);
///         },
///         Err(why) => {
///             println!("Unexpected error: {:?}", why);
///         },
///     }
/// });
/// ```
///
/// [`Client`]: struct.Client.html
/// [`Context`]: struct.Context.html
/// [`Context::ban`]: struct.Context.html#method.ban
/// [`Error::Client`]: ../enum.Error.html#variant.Client
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// When attempting to delete below or above the minimum and maximum allowed
    /// number of messages.
    BulkDeleteAmount,
    /// When attempting to delete a number of days' worth of messages that is
    /// not allowed.
    DeleteMessageDaysAmount(u8),
    /// When there was an error retrieving the gateway URI from the REST API.
    Gateway,
    /// An indication that a [guild][`LiveGuild`] could not be found by
    /// [Id][`GuildId`] in the [`Cache`].
    ///
    /// [`GuildId`]: ../model/struct.GuildId.html
    /// [`LiveGuild`]: ../model/struct.LiveGuild.html
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    GuildNotFound,
    InvalidOpCode,
    /// When attempting to perform an action which is only available to user
    /// accounts.
    InvalidOperationAsBot,
    /// When attempting to perform an action which is only available to bot
    /// accounts.
    InvalidOperationAsUser,
    /// Indicates that you do not have the required permissions to perform an
    /// operation.
    ///
    /// The provided [`Permission`]s is the set of required permissions
    /// required.
    ///
    /// [`Permission`]: ../model/permissions/struct.Permissions.html
    InvalidPermissions(Permissions),
    /// An indicator that the shard data received from the gateway is invalid.
    InvalidShards,
    /// When the token provided is invalid. This is returned when validating a
    /// token through the [`validate_token`] function.
    ///
    /// [`validate_token`]: fn.validate_token.html
    InvalidToken,
    /// An indicator that the [current user] can not perform an action.
    ///
    /// [current user]: ../model/struct.CurrentUser.html
    InvalidUser,
    /// An indicator that an item is missing from the [`Cache`], and the action
    /// can not be continued.
    ///
    /// [`Cache`]: ../ext/cache/struct.Cache.html
    ItemMissing,
    /// Indicates that a [`Message`]s content was too long and will not
    /// successfully send, as the length is over 2000 codepoints, or 4000 bytes.
    ///
    /// The number of bytes larger than the limit is provided.
    ///
    /// [`Message`]: ../model/struct.Message.html
    MessageTooLong(u64),
    /// When attempting to use a [`Context`] helper method which requires a
    /// contextual [`ChannelId`], but the current context is not appropriate for
    /// the action.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`Context`]: struct.Context.html
    NoChannelId,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// into an `i64`.
    RateLimitI64,
    /// When the decoding of a ratelimit header could not be properly decoded
    /// from UTF-8.
    RateLimitUtf8,
    /// When attempting to find a required record from the Cache could not be
    /// found. This is required in methods such as [`Context::edit_role`].
    ///
    /// [`Context::edit_role`]: struct.Context.html#method.edit_role
    RecordNotFound,
    /// When the shard being retrieved from within the Client could not be
    /// found after being inserted into the Client's internal vector of
    /// [`Shard`]s.
    ///
    /// This can be returned from one of the options for starting one or
    /// multiple shards.
    ///
    /// **This should never be received.**
    ///
    /// [`Shard`]: gateway/struct.Shard.html
    ShardUnknown,
    /// When a function such as [`Context::edit_channel`] did not expect the
    /// received [`ChannelType`].
    ///
    /// [`ChannelType`]: ../model/enum.ChannelType.html
    /// [`Context::edit_channel`]: struct.Context.html#method.edit_channel
    UnexpectedChannelType(ChannelType),
    /// When a status code was unexpectedly received for a request's status.
    UnexpectedStatusCode(StatusCode),
    /// When a status is received, but the verification to ensure the response
    /// is valid does not recognize the status.
    UnknownStatus(u16),
}
