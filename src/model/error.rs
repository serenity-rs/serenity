use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use super::Permissions;

/// An error returned from the [`model`] module.
///
/// This is always wrapped within the library's [`Error::Model`] variant.
///
/// # Examples
///
/// Matching an [`Error`] with this variant would look something like the
/// following for the [`GuildId::ban`] method, which in this example is used to
/// re-ban all members with an odd discriminator:
///
/// ```rust,no_run
/// use serenity::{Client, Error};
/// use serenity::model::ModelError;
/// use std::env;
///
/// let token = env::var("DISCORD_BOT_TOKEN").unwrap();
/// let mut client = Client::login(&token);
///
/// client.on_member_unban(|context, guild_id, user| {
///     // If the user has an even discriminator, don't re-ban them.
///     if user.discriminator % 2 == 0 {
///         return;
///     }
///
///     match guild_id.ban(user, 8) {
///         Ok(()) => {
///             // Ban successful.
///         },
///         Err(Error::Model(ModelError::DeleteMessageDaysAmount(amount))) => {
///             println!("Failed deleting {} days' worth of messages", amount);
///         },
///         Err(why) => {
///             println!("Unexpected error: {:?}", why);
///         },
///     }
/// });
/// ```
///
/// [`Error`]: ../enum.Error.html
/// [`Error::Model`]: ../enum.Error.html#variant.Model
/// [`GuildId::ban`]: struct.GuildId.html#method.ban
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    /// When attempting to delete below or above the minimum and maximum allowed
    /// number of messages.
    BulkDeleteAmount,
    /// When attempting to delete a number of days' worth of messages that is
    /// not allowed.
    DeleteMessageDaysAmount(u8),
    /// Indicates that the textual content of an embed exceeds the maximum
    /// length.
    EmbedTooLarge(u64),
    /// An indication that a [guild][`Guild`] could not be found by
    /// [Id][`GuildId`] in the [`Cache`].
    ///
    /// [`Guild`]: ../model/struct.Guild.html
    /// [`GuildId`]: ../model/struct.GuildId.html
    /// [`Cache`]: ../cache/struct.Cache.html
    GuildNotFound,
    /// Indicates that you do not have the required permissions to perform an
    /// operation.
    ///
    /// The provided [`Permission`]s is the set of required permissions
    /// required.
    ///
    /// [`Permission`]: ../model/permissions/struct.Permissions.html
    InvalidPermissions(Permissions),
    /// An indicator that the [current user] can not perform an action.
    ///
    /// [current user]: ../model/struct.CurrentUser.html
    InvalidUser,
    /// An indicator that an item is missing from the [`Cache`], and the action
    /// can not be continued.
    ///
    /// [`Cache`]: ../cache/struct.Cache.html
    ItemMissing,
    /// Indicates that a [`Message`]s content was too long and will not
    /// successfully send, as the length is over 2000 codepoints, or 4000 bytes.
    ///
    /// The number of bytes larger than the limit is provided.
    ///
    /// [`Message`]: ../model/struct.Message.html
    MessageTooLong(u64),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::BulkDeleteAmount => "Too few/many messages to bulk delete",
            Error::DeleteMessageDaysAmount(_) => "Invalid delete message days",
            Error::EmbedTooLarge(_) => "Embed too large",
            Error::GuildNotFound => "Guild not found in the cache",
            Error::InvalidPermissions(_) => "Invalid permissions",
            Error::InvalidUser => "The current user can not perform the action",
            Error::ItemMissing => "The required item is missing from the cache",
            Error::MessageTooLong(_) => "Message too large",
        }
    }
}
