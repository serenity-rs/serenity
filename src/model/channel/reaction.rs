use std::convert::TryFrom;
use std::str::FromStr;
use std::{
    cmp::Ordering,
    error::Error as StdError,
    fmt::{self, Display, Formatter, Result as FmtResult, Write as FmtWrite},
};

use serde::de::{Deserialize, Error as DeError, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
#[cfg(feature = "model")]
use tracing::warn;

#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// An emoji reaction to a message.
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct Reaction {
    /// The [`Channel`] of the associated [`Message`].
    pub channel_id: ChannelId,
    /// The reactive emoji used.
    pub emoji: ReactionType,
    /// The Id of the [`Message`] that was reacted to.
    pub message_id: MessageId,
    /// The Id of the [`User`] that sent the reaction.
    ///
    /// Set to [`None`] by [`Message::react`] when cache is not available.
    pub user_id: Option<UserId>,
    /// The optional Id of the [`Guild`] where the reaction was sent.
    pub guild_id: Option<GuildId>,
    /// The optional object of the member which added the reaction.
    pub member: Option<PartialMember>,
}

impl<'de> Deserialize<'de> for Reaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let channel_id = map
            .remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

        let message_id = map
            .remove("message_id")
            .ok_or_else(|| DeError::custom("expected message_id"))
            .and_then(MessageId::deserialize)
            .map_err(DeError::custom)?;

        let emoji = map
            .remove("emoji")
            .ok_or_else(|| DeError::custom("expected emoji"))
            .and_then(ReactionType::deserialize)
            .map_err(DeError::custom)?;

        let user_id = match map.contains_key("user_id") {
            true => Some(
                map.remove("user_id")
                    .ok_or_else(|| DeError::custom("expected user_id"))
                    .and_then(UserId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        let guild_id = match map.contains_key("guild_id") {
            true => Some(
                map.remove("guild_id")
                    .ok_or_else(|| DeError::custom("expected guild_id"))
                    .and_then(GuildId::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        if let Some(id) = guild_id {
            if let Some(member) = map.get_mut("member") {
                if let Some(object) = member.as_object_mut() {
                    object.insert("guild_id".to_owned(), Value::String(id.to_string()));
                }
            }
        }

        let member = match map.contains_key("member") {
            true => Some(
                map.remove("member")
                    .ok_or_else(|| DeError::custom("expected member"))
                    .and_then(PartialMember::deserialize)
                    .map_err(DeError::custom)?,
            ),
            false => None,
        };

        Ok(Self {
            channel_id,
            emoji,
            message_id,
            user_id,
            guild_id,
            member,
        })
    }
}

#[cfg(feature = "model")]
impl Reaction {
    /// Retrieves the associated the reaction was made in.
    ///
    /// If the cache is enabled, this will search for the already-cached
    /// channel. If not - or the channel was not found - this will perform a
    /// request over the REST API for the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission,
    /// or if the channel no longer exists.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn channel(&self, cache_http: impl CacheHttp) -> Result<Channel> {
        self.channel_id.to_channel(cache_http).await
    }

    /// Deletes the reaction, but only if the current user is the user who made
    /// the reaction or has permission to.
    ///
    /// Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: super::permissions
    pub async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        // Silences a warning when compiling without the `cache` feature.
        #[allow(unused_mut)]
        let mut user_id = self.user_id.map(|id| id.0);

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.user_id.is_some() && self.user_id == Some(cache.current_user().await.id) {
                    user_id = None;
                }

                if user_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        self.guild_id,
                        Permissions::MANAGE_MESSAGES,
                    )
                    .await?;
                }
            }
        }

        cache_http
            .http()
            .delete_reaction(self.channel_id.0, self.message_id.0, user_id, &self.emoji)
            .await
    }

    /// Deletes all reactions from the message with this emoji.
    ///
    /// Requires the [Manage Messages] permission
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// Otherwise returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Messages]: Permissions::MANAGE_MESSAGES
    /// [permissions]: super::permissions
    pub async fn delete_all(&self, cache_http: impl CacheHttp) -> Result<()> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                utils::user_has_perms_cache(
                    cache,
                    self.channel_id,
                    self.guild_id,
                    Permissions::MANAGE_MESSAGES,
                )
                .await?;
            }
        }
        cache_http
            .http()
            .as_ref()
            .delete_message_reaction_emoji(self.channel_id.0, self.message_id.0, &self.emoji)
            .await
    }

    /// Retrieves the [`Message`] associated with this reaction.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// **Note**: This will send a request to the REST API. Prefer maintaining
    /// your own message cache or otherwise having the message available if
    /// possible.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// read message history, or if the message was deleted.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn message(&self, http: impl AsRef<Http>) -> Result<Message> {
        self.channel_id.message(&http, self.message_id).await
    }

    /// Retrieves the user that made the reaction.
    ///
    /// If the cache is enabled, this will search for the already-cached user.
    /// If not - or the user was not found - this will perform a request over
    /// the REST API for the user.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the user that made the reaction is unable to be
    /// retrieved from the API.
    pub async fn user(&self, cache_http: impl CacheHttp) -> Result<User> {
        match self.user_id {
            Some(id) => id.to_user(cache_http).await,
            None => {
                // This can happen if only Http was passed to Message::react, even though
                // "cache" was enabled.
                #[cfg(feature = "cache")]
                {
                    if let Some(cache) = cache_http.cache() {
                        return Ok(User::from(&cache.current_user().await));
                    }
                }

                Ok(cache_http.http().get_current_user().await?.into())
            },
        }
    }

    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// **Note**: This will send a request to the REST API.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    /// [permissions]: super::permissions
    #[inline]
    pub async fn users<R, U>(
        &self,
        http: impl AsRef<Http>,
        reaction_type: R,
        limit: Option<u8>,
        after: Option<U>,
    ) -> Result<Vec<User>>
    where
        R: Into<ReactionType>,
        U: Into<UserId>,
    {
        self._users(&http, &reaction_type.into(), limit, after.map(Into::into)).await
    }

    async fn _users(
        &self,
        http: impl AsRef<Http>,
        reaction_type: &ReactionType,
        limit: Option<u8>,
        after: Option<UserId>,
    ) -> Result<Vec<User>> {
        let mut limit = limit.unwrap_or(50);

        if limit > 100 {
            limit = 100;
            warn!("Rection users limit clamped to 100! (API Restriction)");
        }

        http.as_ref()
            .get_reaction_users(
                self.channel_id.0,
                self.message_id.0,
                reaction_type,
                limit,
                after.map(|u| u.0),
            )
            .await
    }
}

/// The type of a [`Reaction`] sent.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the
    /// guild.
    Custom {
        /// Whether the emoji is animated.
        animated: bool,
        /// The Id of the custom [`Emoji`].
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration
        /// and distinguishing the emoji client-side.
        name: Option<String>,
    },
    /// A reaction with a twemoji.
    Unicode(String),
}

impl<'de> Deserialize<'de> for ReactionType {
    #[allow(clippy::unwrap_used)] // allow unwrap here because name being none is unreachable
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Animated,
            Id,
            Name,
        }

        struct ReactionTypeVisitor;

        impl<'de> Visitor<'de> for ReactionTypeVisitor {
            type Value = ReactionType;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                formatter.write_str("enum ReactionType")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> StdResult<Self::Value, V::Error> {
                let mut animated = None;
                let mut id = None;
                let mut name = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Animated => {
                            if animated.is_some() {
                                return Err(DeError::duplicate_field("animated"));
                            }

                            animated = Some(map.next_value()?);
                        },
                        Field::Id => {
                            if id.is_some() {
                                return Err(DeError::duplicate_field("id"));
                            }

                            if let Ok(emoji_id) = map.next_value::<EmojiId>() {
                                id = Some(emoji_id)
                            }
                        },
                        Field::Name => {
                            if name.is_some() {
                                return Err(DeError::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        },
                    }
                }

                let animated = animated.unwrap_or(false);
                let name = name.ok_or_else(|| DeError::missing_field("name"))?;

                Ok(if let Some(id) = id {
                    ReactionType::Custom {
                        animated,
                        id,
                        name,
                    }
                } else {
                    ReactionType::Unicode(name.unwrap())
                })
            }
        }

        deserializer.deserialize_map(ReactionTypeVisitor)
    }
}

impl Serialize for ReactionType {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ReactionType::Custom {
                animated,
                id,
                ref name,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;

                map.serialize_entry("animated", &animated)?;
                map.serialize_entry("id", &id.0)?;
                map.serialize_entry("name", &name)?;

                map.end()
            },
            ReactionType::Unicode(ref name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry("name", &name)?;

                map.end()
            },
        }
    }
}

#[cfg(feature = "model")]
impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    #[inline]
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom {
                id,
                ref name,
                ..
            } => {
                format!("{}:{}", name.as_ref().map_or("", |s| s.as_str()), id)
            },
            ReactionType::Unicode(ref unicode) => unicode.clone(),
        }
    }

    /// Helper function to allow testing equality of unicode emojis without
    /// having to perform any allocation.
    /// Will always return false if the reaction was not a unicode reaction.
    pub fn unicode_eq(&self, other: &str) -> bool {
        if let ReactionType::Unicode(unicode) = &self {
            unicode == other
        } else {
            // Always return false if not a unicode reaction
            false
        }
    }

    /// Helper function to allow comparing unicode emojis without having
    /// to perform any allocation.
    /// Will return None if the reaction was not a unicode reaction.
    pub fn unicode_partial_cmp(&self, other: &str) -> Option<Ordering> {
        if let ReactionType::Unicode(unicode) = &self {
            Some(unicode.as_str().cmp(other))
        } else {
            // Always return None if not a unicode reaction
            None
        }
    }
}

impl From<char> for ReactionType {
    /// Creates a [`ReactionType`] from a `char`.
    ///
    /// # Examples
    ///
    /// Reacting to a message with an apple:
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "client")]
    /// # use serenity::client::Context;
    /// # #[cfg(feature = "framework")]
    /// # use serenity::framework::standard::{CommandResult, macros::command};
    /// # use serenity::model::id::ChannelId;
    /// #
    /// # #[cfg(all(feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(ctx: &Context) -> CommandResult {
    /// #   let message = ChannelId(0).message(&ctx.http, 0).await?;
    /// #
    /// message.react(ctx, '🍎').await?;
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {}
    /// ```
    fn from(ch: char) -> ReactionType {
        ReactionType::Unicode(ch.to_string())
    }
}

impl From<Emoji> for ReactionType {
    fn from(emoji: Emoji) -> ReactionType {
        ReactionType::Custom {
            animated: emoji.animated,
            id: emoji.id,
            name: Some(emoji.name),
        }
    }
}

impl From<EmojiId> for ReactionType {
    fn from(emoji_id: EmojiId) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: emoji_id,
            name: None,
        }
    }
}

impl From<EmojiIdentifier> for ReactionType {
    fn from(emoji_id: EmojiIdentifier) -> ReactionType {
        ReactionType::Custom {
            animated: emoji_id.animated,
            id: emoji_id.id,
            name: Some(emoji_id.name),
        }
    }
}

#[derive(Debug)]
pub struct ReactionConversionError;

impl Display for ReactionConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to convert from a string to ReactionType")
    }
}

impl std::error::Error for ReactionConversionError {}

impl TryFrom<String> for ReactionType {
    type Error = ReactionConversionError;

    fn try_from(emoji_string: String) -> std::result::Result<Self, Self::Error> {
        if emoji_string.is_empty() {
            return Err(ReactionConversionError);
        }

        if !emoji_string.starts_with('<') {
            return Ok(ReactionType::Unicode(emoji_string));
        }
        ReactionType::try_from(&emoji_string[..])
    }
}

impl<'a> TryFrom<&'a str> for ReactionType {
    /// Creates a [`ReactionType`] from a string slice.
    ///
    /// # Examples
    ///
    /// Creating a [`ReactionType`] from a `🍎`, modeling a similar API as the
    /// rest of the library:
    ///
    /// ```rust
    /// use std::convert::TryInto;
    /// use std::fmt::Debug;
    ///
    /// use serenity::model::channel::ReactionType;
    ///
    /// fn foo<R: TryInto<ReactionType>>(bar: R)
    /// where
    ///     R::Error: Debug,
    /// {
    ///     println!("{:?}", bar.try_into().unwrap());
    /// }
    ///
    /// foo("🍎");
    /// ```
    ///
    /// Creating a [`ReactionType`] from a custom emoji argument in the following format:
    ///
    /// ```rust
    /// use std::convert::TryFrom;
    ///
    /// use serenity::model::channel::ReactionType;
    /// use serenity::model::id::EmojiId;
    ///
    /// let emoji_string = "<:customemoji:600404340292059257>";
    /// let reaction = ReactionType::try_from(emoji_string).unwrap();
    /// let reaction2 = ReactionType::Custom {
    ///     animated: false,
    ///     id: EmojiId(600404340292059257),
    ///     name: Some("customemoji".to_string()),
    /// };
    ///
    /// assert_eq!(reaction, reaction2);
    /// ```

    type Error = ReactionConversionError;

    fn try_from(emoji_str: &str) -> std::result::Result<Self, Self::Error> {
        if emoji_str.is_empty() {
            return Err(ReactionConversionError);
        }

        if !emoji_str.starts_with('<') {
            return Ok(ReactionType::Unicode(emoji_str.to_string()));
        }

        if !emoji_str.ends_with('>') {
            return Err(ReactionConversionError);
        }

        let emoji_str = emoji_str.trim_matches(&['<', '>'] as &[char]);

        let mut split_iter = emoji_str.split(':');

        let animated = split_iter.next().ok_or(ReactionConversionError)? == "a";

        let name = split_iter.next().ok_or(ReactionConversionError)?.to_string().into();

        let id = split_iter
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(ReactionConversionError)?
            .into();

        Ok(ReactionType::Custom {
            animated,
            id,
            name,
        })
    }
}

// TODO: Change this to `!` once it becomes stable.
#[derive(Debug)]
pub enum NeverFails {}

impl Display for NeverFails {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "never fails")
    }
}

impl StdError for NeverFails {
    fn description(&self) -> &str {
        "never fails"
    }
}

impl FromStr for ReactionType {
    type Err = ReactionConversionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ReactionType::try_from(s)
    }
}

impl Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a
    /// way that clients can understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to
    /// the documentation for [emoji's formatter][`Emoji::fmt`] on how this is
    /// displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            ReactionType::Custom {
                animated,
                id,
                ref name,
            } => {
                if animated {
                    f.write_str("<a:")?;
                } else {
                    f.write_str("<:")?;
                }
                f.write_str(name.as_ref().map_or("", |s| s.as_str()))?;
                f.write_char(':')?;
                Display::fmt(&id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
