//! Error enum definition wrapping potential model implementation errors.

use std::error::Error as StdError;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Maximum {
    EmbedLength,
    EmbedCount,
    MessageLength,
    StickerCount,
    WebhookName,
    AuditLogReason,
    DeleteMessageDays,
    BulkDeleteAmount,
}

#[cfg(feature = "http")]
impl Maximum {
    pub(crate) fn check_overflow(self, value: usize) -> Result<(), Error> {
        let max = self.value();
        if value > max {
            Err(Error::TooLarge {
                maximum: self,
                value,
            })
        } else {
            Ok(())
        }
    }

    pub(crate) fn value(self) -> usize {
        match self {
            Self::EmbedCount => crate::constants::EMBED_MAX_COUNT,
            Self::EmbedLength => crate::constants::EMBED_MAX_LENGTH,
            Self::MessageLength => crate::constants::MESSAGE_CODE_LIMIT,
            Self::StickerCount => crate::constants::STICKER_MAX_COUNT,
            Self::WebhookName | Self::BulkDeleteAmount => 100,
            Self::AuditLogReason => 512,
            Self::DeleteMessageDays => 7,
        }
    }
}

impl fmt::Display for Maximum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmbedCount => f.write_str("Embed count"),
            Self::EmbedLength => f.write_str("Embed length"),
            Self::MessageLength => f.write_str("Message length"),
            Self::StickerCount => f.write_str("Sticker count"),
            Self::WebhookName => f.write_str("Webhook name"),
            Self::AuditLogReason => f.write_str("Audit log reason"),
            Self::DeleteMessageDays => f.write_str("Delete message days"),
            Self::BulkDeleteAmount => f.write_str("Message bulk delete count"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Minimum {
    WebhookName,
    BulkDeleteAmount,
}

#[cfg(feature = "http")]
impl Minimum {
    pub(crate) fn check_underflow(self, value: usize) -> Result<(), Error> {
        let min = self.value();
        if value < min {
            Err(Error::TooSmall {
                minimum: self,
                value,
            })
        } else {
            Ok(())
        }
    }

    pub(crate) fn value(self) -> usize {
        match self {
            Self::WebhookName => 2,
            Self::BulkDeleteAmount => 1,
        }
    }
}

impl fmt::Display for Minimum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebhookName => f.write_str("Webhook name"),
            Self::BulkDeleteAmount => f.write_str("Bulk delete amount"),
        }
    }
}

/// An error returned from the [`model`] module.
///
/// This is always wrapped within the library's [`Error::Model`] variant.
///
/// # Examples
///
/// Matching an [`Error`] with this variant would look something like the following for the
/// [`GuildId::ban`] method, which in this example is used to re-ban all members.
///
/// ```rust,no_run
/// use serenity::model::prelude::*;
/// use serenity::model::ModelError;
/// use serenity::prelude::*;
/// use serenity::Error;
///
/// # #[cfg(feature = "http")]
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// # let http: serenity::http::Http = unimplemented!();
/// # let guild_id: GuildId = unimplemented!();
/// # let user: User = unimplemented!();
///
/// match guild_id.ban(&http, user.id, 8, Some("No unbanning people!")).await {
///     Ok(()) => {
///         // Ban successful.
///     },
///     Err(Error::Model(ModelError::TooLarge {
///         value, ..
///     })) => {
///         println!("Failed deleting {value} days' worth of messages");
///     },
///     Err(why) => {
///         println!("Unexpected error: {why:?}");
///     },
/// }
///
/// # Ok(())
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
    /// Indicates that the `minimum` has been missed by the `value`.
    TooSmall { minimum: Minimum, value: usize },
    /// Indicates that the `maximum` has been exceeded by the `value`.
    TooLarge { maximum: Maximum, value: usize },
    /// An indication that a [`Guild`] could not be found by [Id][`GuildId`] in the [`Cache`].
    ///
    /// [`Guild`]: super::guild::Guild
    /// [`GuildId`]: super::id::GuildId
    /// [`Cache`]: crate::cache::Cache
    GuildNotFound,
    /// An indication that a [`Role`] could not be found by [Id][`RoleId`] in the [`Cache`].
    ///
    /// [`Role`]: super::guild::Role
    /// [`RoleId`]: super::id::RoleId
    /// [`Cache`]: crate::cache::Cache
    RoleNotFound,
    /// An indication that a [`Member`] could not be found by [Id][`UserId`] in the [`Cache`].
    ///
    /// [`Member`]: super::guild::Member
    /// [`UserId`]: super::id::UserId
    /// [`Cache`]: crate::cache::Cache
    MemberNotFound,
    /// An indication that a [`Channel`] could not be found by [Id][`ChannelId`] in the [`Cache`].
    ///
    /// [`Channel`]: super::channel::Channel
    /// [`ChannelId`]: super::id::ChannelId
    /// [`Cache`]: crate::cache::Cache
    ChannelNotFound,
    /// An indication that a [`Message`] has already been crossposted, and cannot be crossposted
    /// twice.
    ///
    /// [`Message`]: super::channel::Message
    MessageAlreadyCrossposted,
    /// An indication that you cannot crosspost a [`Message`].
    ///
    /// For instance, you cannot crosspost a system message or a message coming from the crosspost
    /// feature.
    ///
    /// [`Message`]: super::channel::Message
    CannotCrosspostMessage,
    /// Indicates that there are hierarchy problems restricting an action.
    ///
    /// For example, when banning a user, if the other user has a role with an equal to or higher
    /// position, then they can not be banned.
    ///
    /// When editing a role, if the role is higher in position than the current user's highest
    /// role, then the role can not be edited.
    Hierarchy,
    /// An indicator that the [current user] cannot perform an action.
    ///
    /// [current user]: super::user::CurrentUser
    InvalidUser,
    /// An indicator that an item is missing from the [`Cache`], and the action can not be
    /// continued.
    ///
    /// [`Cache`]: crate::cache::Cache
    ItemMissing,
    /// Indicates that the current user is attempting to Direct Message another bot user, which is
    /// disallowed by the API.
    MessagingBot,
    /// An indicator that the [`ChannelType`] cannot perform an action.
    ///
    /// [`ChannelType`]: super::channel::ChannelType
    InvalidChannelType,
    /// Indicates that the webhook token is missing.
    NoTokenSet,
    /// When attempting to delete a built in nitro sticker instead of a guild sticker.
    DeleteNitroSticker,
    /// When attempting to edit a voice message.
    CannotEditVoiceMessage,
}

impl Error {
    /// Return `true` if the model error is related to an item missing in the cache.
    #[must_use]
    pub const fn is_cache_err(&self) -> bool {
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall {
                minimum,
                value,
            } => write!(f, "The {minimum} minimum has been missed by {value}"),
            Self::TooLarge {
                maximum,
                value,
            } => write!(f, "The {maximum} maximum has been overflowed by {value}"),
            Self::GuildNotFound => f.write_str("Guild not found in the cache."),
            Self::RoleNotFound => f.write_str("Role not found in the cache."),
            Self::MemberNotFound => f.write_str("Member not found in the cache."),
            Self::ChannelNotFound => f.write_str("Channel not found in the cache."),
            Self::Hierarchy => f.write_str("Role hierarchy prevents this action."),
            Self::InvalidChannelType => f.write_str("The channel cannot perform the action."),
            Self::InvalidUser => f.write_str("The current user cannot perform the action."),
            Self::ItemMissing => f.write_str("The required item is missing from the cache."),
            Self::MessageAlreadyCrossposted => f.write_str("Message already crossposted."),
            Self::CannotCrosspostMessage => f.write_str("Cannot crosspost this message type."),
            Self::MessagingBot => f.write_str("Attempted to message another bot user."),
            Self::NoTokenSet => f.write_str("Token is not set."),
            Self::DeleteNitroSticker => f.write_str("Cannot delete an official sticker."),
            Self::CannotEditVoiceMessage => f.write_str("Cannot edit voice message."),
        }
    }
}

impl StdError for Error {}
