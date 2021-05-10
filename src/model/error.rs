//! Error enum definition wrapping potential model implementation errors.

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

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
/// # #[cfg(all(feature = "client", feature = "model"))]
/// # async fn run() -> Result<(), Box<std::error::Error>> {
/// use serenity::prelude::*;
/// use serenity::model::prelude::*;
/// use serenity::Error;
/// use serenity::model::ModelError;
///
/// struct Handler;
///
/// #[serenity::async_trait]
/// impl EventHandler for Handler {
///     async fn guild_ban_removal(&self, context: Context, guild_id: GuildId, user: User) {
///         // If the user has an even discriminator, don't re-ban them.
///         if user.discriminator % 2 == 0 {
///             return;
///         }
///
///         match guild_id.ban(&context, user, 8).await {
///             Ok(()) => {
///                 // Ban successful.
///             },
///             Err(Error::Model(ModelError::DeleteMessageDaysAmount(amount))) => {
///                 println!("Failed deleting {} days' worth of messages", amount);
///             },
///             Err(why) => {
///                 println!("Unexpected error: {:?}", why);
///             },
///         }
///     }
/// }
/// let token = std::env::var("DISCORD_BOT_TOKEN")?;
/// let mut client = Client::builder(&token).event_handler(Handler).await?;
///
/// client.start().await?;
/// #     Ok(())
/// # }
/// ```
///
/// [`Error`]: crate::Error
/// [`Error::Model`]: crate::Error::Model
/// [`GuildId::ban`]: super::id::GuildId::ban
/// [`model`]: crate::model
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// When attempting to delete below or above the minimum and maximum allowed
    /// number of messages.
    BulkDeleteAmount,
    /// When attempting to delete a number of days' worth of messages that is
    /// not allowed.
    DeleteMessageDaysAmount(u8),
    /// Indicates that the textual content of an embed exceeds the maximum
    /// length.
    EmbedTooLarge(usize),
    /// An indication that a [guild][`Guild`] could not be found by
    /// [Id][`GuildId`] in the [`Cache`].
    ///
    /// [`Guild`]: super::guild::Guild
    /// [`GuildId`]: super::id::GuildId
    /// [`Cache`]: crate::cache::Cache
    GuildNotFound,
    /// An indication that a [role][`Role`] could not be found by
    /// [Id][`RoleId`] in the [`Cache`].
    ///
    /// [`Role`]: super::guild::Role
    /// [`RoleId`]: super::id::RoleId
    /// [`Cache`]: crate::cache::Cache
    RoleNotFound,
    /// An indication that a [member][`Member`] could not be found by
    /// [Id][`UserId`] in the [`Cache`].
    ///
    /// [`Member`]: super::guild::Member
    /// [`UserId`]: super::id::UserId
    /// [`Cache`]: crate::cache::Cache
    MemberNotFound,
    /// An indication that a [channel][`Channel`] could not be found by
    /// [Id][`ChannelId`] in the [`Cache`].
    ///
    /// [`Channel`]: super::channel::Channel
    /// [`ChannelId`]: super::id::ChannelId
    /// [`Cache`]: crate::cache::Cache
    ChannelNotFound,
    /// An indication that a [`Message`] has already been crossposted,
    /// and cannot be crossposted twice.
    ///
    /// [`Message`]: super::channel::Message
    MessageAlreadyCrossposted,
    /// An indication that you cannot crosspost a [`Message`].
    ///
    /// For instance, you cannot crosspost a system message or a
    /// message coming from the crosspost feature.
    ///
    /// [`Message`]: super::channel::Message
    CannotCrosspostMessage,
    /// Indicates that there are hierarchy problems restricting an action.
    ///
    /// For example, when banning a user, if the other user has a role with an
    /// equal to or higher position, then they can not be banned.
    ///
    /// When editing a role, if the role is higher in position than the current
    /// user's highest role, then the role can not be edited.
    Hierarchy,
    /// Indicates that you do not have the required permissions to perform an
    /// operation.
    ///
    /// The provided [`Permissions`] is the set of required permissions
    /// required.
    InvalidPermissions(Permissions),
    /// An indicator that the [current user] cannot perform an action.
    ///
    /// [current user]: super::user::CurrentUser
    InvalidUser,
    /// An indicator that an item is missing from the [`Cache`], and the action
    /// can not be continued.
    ///
    /// [`Cache`]: crate::cache::Cache
    ItemMissing,
    /// Indicates that a member, role or channel from the wrong [`Guild`] was provided.
    ///
    /// [`Guild`]: super::guild::Guild
    WrongGuild,
    /// Indicates that a [`Message`]s content was too long and will not
    /// successfully send, as the length is over 2000 codepoints.
    ///
    /// The number of code points larger than the limit is provided.
    ///
    /// [`Message`]: super::channel::Message
    MessageTooLong(usize),
    /// Indicates that the current user is attempting to Direct Message another
    /// bot user, which is disallowed by the API.
    MessagingBot,
    /// An indicator that the [`ChannelType`] cannot perform an action.
    ///
    /// [`ChannelType`]: super::channel::ChannelType
    InvalidChannelType,
    /// Indicates that the webhook name is under the 2 characters limit.
    NameTooShort,
    /// Indicates that the webhook name is over the 100 characters limit.
    NameTooLong,
    /// Indicates that the bot is not author of the message.
    /// This error is returned in private/direct channels.
    NotAuthor,
    /// Indicates that the webhook token is missing.
    NoTokenSet,
}

impl Error {
    /// Return `true` if the model error is related to an item missing in the
    /// cache.
    pub fn is_cache_err(&self) -> bool {
        matches!(
            self,
            Self::ItemMissing
                | Self::ChannelNotFound
                | Self::RoleNotFound
                | Self::GuildNotFound
                | Self::MemberNotFound
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::BulkDeleteAmount => f.write_str("Too few/many messages to bulk delete."),
            Error::DeleteMessageDaysAmount(_) => f.write_str("Invalid delete message days."),
            Error::EmbedTooLarge(_) => f.write_str("Embed too large."),
            Error::GuildNotFound => f.write_str("Guild not found in the cache."),
            Error::RoleNotFound => f.write_str("Role not found in the cache."),
            Error::MemberNotFound => f.write_str("Member not found in the cache."),
            Error::ChannelNotFound => f.write_str("Channel not found in the cache."),
            Error::Hierarchy => f.write_str("Role hierarchy prevents this action."),
            Error::InvalidChannelType => f.write_str("The channel cannot perform the action."),
            Error::InvalidPermissions(_) => f.write_str("Invalid permissions."),
            Error::InvalidUser => f.write_str("The current user cannot perform the action."),
            Error::ItemMissing => f.write_str("The required item is missing from the cache."),
            Error::WrongGuild => f.write_str("Provided member or channel is from the wrong guild."),
            Error::MessageTooLong(_) => f.write_str("Message too large."),
            Error::MessageAlreadyCrossposted => f.write_str("Message already crossposted."),
            Error::CannotCrosspostMessage => f.write_str("Cannot crosspost this message type."),
            Error::MessagingBot => f.write_str("Attempted to message another bot user."),
            Error::NameTooShort => f.write_str("Name is under the character limit."),
            Error::NameTooLong => f.write_str("Name is over the character limit."),
            Error::NotAuthor => f.write_str("The bot is not author of this message."),
            Error::NoTokenSet => f.write_str("Token is not set."),
        }
    }
}

impl StdError for Error {}
