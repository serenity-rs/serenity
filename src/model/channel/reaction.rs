use model::prelude::*;
use serde::de::{Deserialize, Error as DeError, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serialize, Serializer};
use std::{
    error::Error as StdError,
    fmt::{
        Display, 
        Formatter, 
        Result as FmtResult, 
        Write as FmtWrite
    },
    str::FromStr
};
use internal::prelude::*;

/// An emoji reaction to a message.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reaction {
    /// The [`Channel`] of the associated [`Message`].
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Message`]: struct.Message.html
    pub channel_id: ChannelId,
    /// The reactive emoji used.
    pub emoji: ReactionType,
    /// The Id of the [`Message`] that was reacted to.
    ///
    /// [`Message`]: struct.Message.html
    pub message_id: MessageId,
    /// The Id of the [`User`] that sent the reaction.
    ///
    /// [`User`]: struct.User.html
    pub user_id: UserId,
}

/// The type of a [`Reaction`] sent.
///
/// [`Reaction`]: struct.Reaction.html
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the
    /// guild.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Guild`]: struct.Guild.html
    Custom {
        /// Whether the emoji is animated.
        animated: bool,
        /// The Id of the custom [`Emoji`].
        ///
        /// [`Emoji`]: struct.Emoji.html
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

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
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
        where S: Serializer {
        match *self {
            ReactionType::Custom { animated, id, ref name } => {
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

#[cfg(any(feature = "model", feature = "http-client"))]
impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom {
                id,
                ref name,
                ..
            } => format!("{}:{}", name.as_ref().map_or("", |s| s.as_str()), id),
            ReactionType::Unicode(ref unicode) => unicode.clone(),
        }
    }
}

impl From<char> for ReactionType {
    /// Creates a `ReactionType` from a `char`.
    ///
    /// # Examples
    ///
    /// Reacting to a message with an apple:
    ///
    /// ```rust,no_run
    /// # use serenity::model::id::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let message = ChannelId(0).message(0)?;
    /// #
    /// message.react('🍎')?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    fn from(ch: char) -> ReactionType { ReactionType::Unicode(ch.to_string()) }
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
            name: None
        }
    }
}

impl From<EmojiIdentifier> for ReactionType {
    fn from(emoji_id: EmojiIdentifier) -> ReactionType {
        ReactionType::Custom {
            animated: false,
            id: emoji_id.id,
            name: Some(emoji_id.name)
        }
    }
}

impl From<String> for ReactionType {
    fn from(unicode: String) -> ReactionType { ReactionType::Unicode(unicode) }
}

impl<'a> From<&'a str> for ReactionType {
    /// Creates a `ReactionType` from a string slice.
    ///
    /// # Examples
    ///
    /// Creating a `ReactionType` from a `🍎`, modeling a similar API as the
    /// rest of the library:
    ///
    /// ```rust
    /// use serenity::model::channel::ReactionType;
    ///
    /// fn foo<R: Into<ReactionType>>(bar: R) {
    ///     println!("{:?}", bar.into());
    /// }
    ///
    /// foo("🍎");
    /// ```
    fn from(unicode: &str) -> ReactionType { ReactionType::Unicode(unicode.to_string()) }
}

// TODO: Change this to `!` once it becomes stable.

#[derive(Debug)]
pub enum NeverFails {}

impl Display for NeverFails {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "never fails")
    }
}

impl StdError for NeverFails {
    fn description(&self) -> &str {
        "never fails"
    }
}

impl FromStr for ReactionType {
    type Err = NeverFails;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(ReactionType::from(s))
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
    ///
    /// [`Emoji::fmt`]: struct.Emoji.html#method.fmt
    /// [`ReactionType::Custom`]: enum.ReactionType.html#variant.Custom
    /// [`ReactionType::Unicode`]: enum.ReactionType.html#variant.Unicode
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ReactionType::Custom {
                id,
                ref name,
                ..
            } => {
                f.write_char('<')?;
                f.write_char(':')?;
                f.write_str(name.as_ref().map_or("", |s| s.as_str()))?;
                f.write_char(':')?;
                Display::fmt(&id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
