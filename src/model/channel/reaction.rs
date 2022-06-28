#[cfg(feature = "model")]
use std::cmp::Ordering;
use std::convert::TryFrom;
#[cfg(doc)]
use std::fmt::Display as _;
use std::fmt::{self, Write as _};
use std::str::FromStr;

use serde::de::{Deserialize, Error as DeError, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
#[cfg(feature = "model")]
use tracing::warn;

#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::internal::prelude::*;
use crate::json::from_number;
use crate::model::prelude::*;
use crate::model::utils::{remove_from_map, remove_from_map_opt};

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

        let guild_id = remove_from_map_opt::<GuildId, _>(&mut map, "guild_id")?;

        if let Some(id) = guild_id {
            if let Some(member) = map.get_mut("member") {
                if let Some(object) = member.as_object_mut() {
                    object.insert("guild_id".to_owned(), from_number(id.get()));
                }
            }
        }

        Ok(Self {
            guild_id,
            channel_id: remove_from_map(&mut map, "channel_id")?,
            message_id: remove_from_map(&mut map, "message_id")?,
            user_id: remove_from_map_opt(&mut map, "user_id")?,
            member: remove_from_map_opt(&mut map, "member")?,
            emoji: remove_from_map(&mut map, "emoji")?,
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
        #[cfg_attr(not(feature = "cache"), allow(unused_mut))]
        let mut user_id = self.user_id;

        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if self.user_id == Some(cache.current_user().id) {
                    user_id = None;
                }

                if user_id.is_some() {
                    utils::user_has_perms_cache(
                        cache,
                        self.channel_id,
                        self.guild_id,
                        Permissions::MANAGE_MESSAGES,
                    )?;
                }
            }
        }

        cache_http
            .http()
            .delete_reaction(
                self.channel_id.get(),
                self.message_id.get(),
                user_id.map(UserId::get),
                &self.emoji,
            )
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
                )?;
            }
        }
        cache_http
            .http()
            .as_ref()
            .delete_message_reaction_emoji(
                self.channel_id.get(),
                self.message_id.get(),
                &self.emoji,
            )
            .await
    }

    /// Retrieves the [`Message`] associated with this reaction.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to
    /// read message history, or if the message was deleted.
    ///
    /// [Read Message History]: Permissions::READ_MESSAGE_HISTORY
    #[inline]
    pub async fn message(&self, cache_http: impl CacheHttp) -> Result<Message> {
        self.channel_id.message(&cache_http, self.message_id).await
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
        if let Some(id) = self.user_id {
            id.to_user(cache_http).await
        } else {
            // This can happen if only Http was passed to Message::react, even though
            // "cache" was enabled.
            #[cfg(feature = "cache")]
            {
                if let Some(cache) = cache_http.cache() {
                    return Ok(User::from(&*cache.current_user()));
                }
            }

            Ok(cache_http.http().get_current_user().await?.into())
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
            warn!("Reaction users limit clamped to 100! (API Restriction)");
        }

        http.as_ref()
            .get_reaction_users(
                self.channel_id.get(),
                self.message_id.get(),
                reaction_type,
                limit,
                after.map(UserId::get),
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

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
                                id = Some(emoji_id);
                            }
                        },
                        Field::Name => {
                            if name.is_some() {
                                return Err(DeError::duplicate_field("name"));
                            }

                            name = Some(map.next_value::<Option<String>>()?);
                        },
                    }
                }

                let rt = match (id, name) {
                    (Some(id), name) => ReactionType::Custom {
                        animated: animated.unwrap_or_default(),
                        id,
                        name: name.flatten(),
                    },
                    (None, Some(Some(name))) => ReactionType::Unicode(name),
                    _ => return Err(DeError::custom("invalid reaction type data")),
                };
                Ok(rt)
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
        match self {
            ReactionType::Custom {
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
            ReactionType::Unicode(name) => {
                let mut map = serializer.serialize_map(Some(1))?;

                map.serialize_entry("name", name)?;

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
    #[must_use]
    pub fn as_data(&self) -> String {
        match self {
            ReactionType::Custom {
                id,
                name,
                ..
            } => {
                format!("{}:{}", name.as_deref().unwrap_or(""), id)
            },
            ReactionType::Unicode(unicode) => unicode.clone(),
        }
    }

    /// Helper function to allow testing equality of unicode emojis without
    /// having to perform any allocation.
    /// Will always return false if the reaction was not a unicode reaction.
    #[must_use]
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
    /// # use serenity::model::id::{ChannelId, MessageId};
    /// #
    /// # #[cfg(all(feature = "client", feature = "framework", feature = "http"))]
    /// # #[command]
    /// # async fn example(ctx: &Context) -> CommandResult {
    /// #   let message = ChannelId::new(1).message(&ctx.http, MessageId::new(0)).await?;
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

impl fmt::Display for ReactionConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to convert from a string to ReactionType")
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
    ///     id: EmojiId::new(600404340292059257),
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
            .and_then(|s| s.parse().ok())
            .map(EmojiId)
            .ok_or(ReactionConversionError)?;

        Ok(ReactionType::Custom {
            animated,
            id,
            name,
        })
    }
}

impl FromStr for ReactionType {
    type Err = ReactionConversionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        ReactionType::try_from(s)
    }
}

impl fmt::Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a
    /// way that clients can understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to
    /// the documentation for [emoji's formatter][`Emoji::fmt`] on how this is
    /// displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReactionType::Custom {
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
