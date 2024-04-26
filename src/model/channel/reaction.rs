use std::borrow::Cow;
use std::cmp::Ordering;
#[cfg(doc)]
use std::fmt::Display as _;
use std::fmt::{self, Write as _};

use nonmax::NonMaxU8;
#[cfg(feature = "http")]
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::de::Error as DeError;
use serde::ser::{Serialize, SerializeMap, Serializer};
#[cfg(feature = "model")]
use tracing::warn;

#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// An emoji reaction to a message.
///
/// [Discord docs](https://discord.com/developers/docs/topics/gateway#message-reaction-add-message-reaction-add-event-fields).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[non_exhaustive]
pub struct Reaction {
    /// The Id of the [`User`] that sent the reaction.
    ///
    /// Always present when received from gateway.
    /// Set to [`None`] by [`Message::react`] when cache is not available.
    pub user_id: Option<UserId>,
    /// The [`Channel`] of the associated [`Message`].
    pub channel_id: ChannelId,
    /// The Id of the [`Message`] that was reacted to.
    pub message_id: MessageId,
    /// The optional Id of the [`Guild`] where the reaction was sent.
    pub guild_id: Option<GuildId>,
    /// The optional object of the member which added the reaction.
    pub member: Option<Member>,
    /// The reactive emoji used.
    pub emoji: FixedReactionType,
}

// Manual impl needed to insert guild_id into PartialMember
impl<'de> Deserialize<'de> for Reaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut reaction = Self::deserialize(deserializer)?; // calls #[serde(remote)]-generated inherent method
        if let (Some(guild_id), Some(member)) = (reaction.guild_id, reaction.member.as_mut()) {
            member.guild_id = guild_id;
        }
        Ok(reaction)
    }
}

impl Serialize for Reaction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        Self::serialize(self, serializer) // calls #[serde(remote)]-generated inherent method
    }
}

#[cfg(feature = "model")]
impl Reaction {
    /// Retrieves the associated the reaction was made in.
    ///
    /// If the cache is enabled, this will search for the already-cached channel. If not - or the
    /// channel was not found - this will perform a request over the REST API for the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if the channel no longer
    /// exists.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        self.channel_id.to_channel(cache_http).await
    }

    /// Deletes the reaction, but only if the current user is the user who made the reaction or has
    /// permission to.
    ///
    /// Requires the [Manage Messages] permission, _if_ the current user did not perform the
    /// reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidPermissions`] if the current
    /// user does not have the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: crate::model::permissions
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut user_id = self.user_id;

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.user_id == Some(cache.current_user().id) {
                    user_id = None;
                }

                if let (Some(_), Some(guild_id)) = (user_id, self.guild_id) {
                    crate::utils::user_has_perms_cache(
                        cache,
                        guild_id,
                        self.channel_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }

        self.channel_id
            .delete_reaction(cache_http.http(), self.message_id, user_id, self.emoji.clone())
            .await
    }

    /// Deletes all reactions from the message with this emoji.
    ///
    /// Requires the [Manage Messages] permission
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a [`ModelError::InvalidPermissions`] if the current
    /// user does not have the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: crate::model::permissions
    pub async fn delete_all(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let (Some(cache), Some(guild_id)) = (cache_http.cache(), self.guild_id) {
                crate::utils::user_has_perms_cache(
                    cache,
                    guild_id,
                    self.channel_id,
                    Permissions::MANAGE_MESSAGES,
                )?;
            }
        }
        cache_http
            .http()
            .delete_message_reaction_emoji(self.channel_id, self.message_id, &self.emoji)
            .await
    }

    /// Retrieves the [`Message`] associated with this reaction.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to read message history, or if
    /// the message was deleted.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    pub async fn message(&self, cache_http: impl CacheHttp) -> Result<Message> {
        self.channel_id.message(cache_http, self.message_id).await
    }

    /// Retrieves the user that made the reaction.
    ///
    /// If the cache is enabled, this will search for the already-cached user. If not - or the user
    /// was not found - this will perform a request over the REST API for the user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the user that made the reaction is unable to be retrieved from
    /// the API.
    pub async fn user(&self, cache_http: impl CacheHttp) -> Result<User> {
        if let Some(id) = self.user_id {
            id.to_user(cache_http).await
        } else {
            // This can happen if only Http was passed to Message::react, even though
            // "cache" was enabled.
            #[cfg(feature = "cache")]
            {
                if let Some(cache) = cache_http.cache() {
                    return Ok(cache.current_user().clone().into());
                }
            }

            Ok(cache_http.http().get_current_user().await?.into())
        }
    }

    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different maximum number of
    /// users. The maximum that may be retrieve at a time is `100`, if a greater number is provided
    /// then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain user. This is
    /// useful for pagination.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// **Note**: This will send a request to the REST API.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does not have the required
    /// [permissions].
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    /// [permissions]: crate::model::permissions
    pub async fn users(
        &self,
        http: &Http,
        reaction_type: impl Into<ReactionType<'_>>,
        limit: Option<NonMaxU8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        self._users(http, &reaction_type.into(), limit, after).await
    }

    async fn _users(
        &self,
        http: &Http,
        reaction_type: &ReactionType<'_>,
        limit: Option<NonMaxU8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let mut limit = limit.map_or(50, |limit| limit.get());

        if limit > 100 {
            limit = 100;
            warn!("Reaction users limit clamped to 100! (API Restriction)");
        }

        http.get_reaction_users(self.channel_id, self.message_id, reaction_type, limit, after).await
    }
}

/// The type of a [`Reaction`] sent.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum FixedReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the guild.
    Custom {
        /// Whether the emoji is animated.
        animated: bool,
        /// The Id of the custom [`Emoji`].
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration and distinguishing
        /// the emoji client-side.
        name: Option<FixedString>,
    },
    /// A reaction with a twemoji.
    Unicode(FixedString),
}

// Manual impl needed to decide enum variant by presence of `id`
impl<'de> Deserialize<'de> for FixedReactionType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        struct PartialEmoji {
            #[serde(default)]
            animated: bool,
            id: Option<EmojiId>,
            name: Option<FixedString>,
        }
        let emoji = PartialEmoji::deserialize(deserializer)?;
        Ok(match (emoji.id, emoji.name) {
            (Some(id), name) => Self::Custom {
                animated: emoji.animated,
                id,
                name,
            },
            (None, Some(name)) => Self::Unicode(name),
            (None, None) => return Err(DeError::custom("invalid reaction type data")),
        })
    }
}

impl serde::Serialize for FixedReactionType {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let reaction_type = match self {
            Self::Custom {
                id,
                name,
                animated,
            } => ReactionType::Custom {
                id: *id,
                animated: *animated,
                name: Cow::Borrowed(name.as_str()),
            },
            Self::Unicode(name) => ReactionType::Unicode(Cow::Borrowed(name)),
        };

        reaction_type.serialize(serializer)
    }
}

/// The type of a [`Reaction`] to send.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReactionType<'a> {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the guild.
    Custom {
        /// Whether the emoji is animated.
        animated: bool,
        /// The Id of the custom [`Emoji`].
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration and distinguishing
        /// the emoji client-side.
        name: Option<Cow<'a, str>>,
    },
    /// A reaction with a twemoji.
    Unicode(Cow<'a, str>),
    /// A character reaction with a twemoji.
    UnicodeChar(char),
}

impl Serialize for ReactionType<'_> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Custom {
                animated,
                id,
                name,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;

                map.serialize_entry("animated", animated)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("name", name)?;

                map.end()
            },
            Self::Unicode(name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry("name", name)?;

                map.end()
            },
            Self::UnicodeChar(name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry("name", name)?;

                map.end()
            },
        }
    }
}

impl ReactionType<'_> {
    /// Creates a data-esque display of the type. This is not very useful for displaying, as the
    /// primary client can not render it, but can be useful for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most likely little use for
    /// it.
    #[must_use]
    #[cfg(feature = "http")]
    pub fn as_data(&self) -> String {
        match self {
            ReactionType::Custom {
                id,
                name,
                ..
            } => {
                format!("{}:{id}", name.as_deref().unwrap_or_default())
            },
            ReactionType::Unicode(unicode) => {
                utf8_percent_encode(unicode, NON_ALPHANUMERIC).to_string()
            },
        }
    }
}

impl FixedReactionType {
    /// Helper function to allow testing equality of unicode emojis without having to perform any
    /// allocation. Will always return false if the reaction was not a unicode reaction.
    #[must_use]
    pub fn unicode_eq(&self, other: &str) -> bool {
        if let ReactionType::Unicode(unicode) = &self {
            &**unicode == other
        } else {
            // Always return false if not a unicode reaction
            false
        }
    }

    /// Helper function to allow comparing unicode emojis without having to perform any allocation.
    /// Will return None if the reaction was not a unicode reaction.
    #[must_use]
    pub fn unicode_partial_cmp(&self, other: &str) -> Option<Ordering> {
        if let ReactionType::Unicode(unicode) = &self {
            Some(unicode.as_str().cmp(other))
        } else {
            // Always return None if not a unicode reaction
            None
        }
    }
}

impl From<char> for ReactionType<'static> {
    /// Creates a [`ReactionType`] from a `char`.
    ///
    /// # Examples
    ///
    /// Reacting to a message with an apple:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "http")]
    /// # use serenity::http::CacheHttp;
    /// # use serenity::model::channel::Message;
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(feature = "http")]
    /// # async fn example(ctx: impl CacheHttp, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    /// message.react(ctx, '🍎').await?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    fn from(ch: char) -> Self {
        Self::UnicodeChar(ch)
    }
}

impl From<Emoji> for FixedReactionType {
    fn from(emoji: Emoji) -> Self {
        Self::Custom {
            animated: emoji.animated(),
            id: emoji.id,
            name: Some(emoji.name),
        }
    }
}

impl From<EmojiId> for ReactionType<'static> {
    fn from(emoji_id: EmojiId) -> Self {
        Self::Custom {
            animated: false,
            id: emoji_id,
            name: None,
        }
    }
}

impl From<EmojiIdentifier> for FixedReactionType {
    fn from(emoji_id: EmojiIdentifier) -> Self {
        Self::Custom {
            animated: emoji_id.animated,
            id: emoji_id.id,
            name: Some(emoji_id.name),
        }
    }
}

impl fmt::Display for FixedReactionType {
    /// Formats the reaction type, displaying the associated emoji in a way that clients can
    /// understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to the documentation
    /// for [emoji's formatter][`Emoji::fmt`] on how this is displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom {
                animated,
                id,
                name,
            } => {
                if *animated {
                    f.write_str("<a:")?;
                } else {
                    f.write_str("<:")?;
                }

                if let Some(name) = name {
                    f.write_str(name)?;
                }

                f.write_char(':')?;
                fmt::Display::fmt(id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(unicode) => f.write_str(unicode),
        }
    }
}
