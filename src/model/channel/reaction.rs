use serde::de::{Deserialize, Error as DeError, MapAccess, Visitor};
use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature="cache")]
use ::CACHE;
#[cfg(feature="model")]
use ::http;

/// An emoji reaction to a message.
#[derive(Clone, Debug, Deserialize)]
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

#[cfg(feature="model")]
impl Reaction {
    /// Deletes the reaction, but only if the current user is the user who made
    /// the reaction or has permission to.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a
    /// [`ModelError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    /// [permissions]: permissions
    pub fn delete(&self) -> Result<()> {
        let user_id = feature_cache! {{
            let user = if self.user_id == CACHE.read().unwrap().user.id {
                None
            } else {
                Some(self.user_id.0)
            };

            // If the reaction is one _not_ made by the current user, then ensure
            // that the current user has permission* to delete the reaction.
            //
            // Normally, users can only delete their own reactions.
            //
            // * The `Manage Messages` permission.
            if user.is_some() {
                let req = permissions::MANAGE_MESSAGES;

                if !utils::user_has_perms(self.channel_id, req).unwrap_or(true) {
                    return Err(Error::Model(ModelError::InvalidPermissions(req)));
                }
            }

            user
        } else {
            Some(self.user_id.0)
        }};

        http::delete_reaction(self.channel_id.0,
                              self.message_id.0,
                              user_id,
                              &self.emoji)
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
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ModelError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ModelError::InvalidPermissions`]: enum.ModelError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    /// [permissions]: permissions
    pub fn users<R, U>(&self,
                       reaction_type: R,
                       limit: Option<u8>,
                       after: Option<U>)
                       -> Result<Vec<User>>
                       where R: Into<ReactionType>,
                             U: Into<UserId> {
        http::get_reaction_users(self.channel_id.0,
                                 self.message_id.0,
                                 &reaction_type.into(),
                                 limit.unwrap_or(50),
                                 after.map(|u| u.into().0))
    }
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
        /// The Id of the custom [`Emoji`].
        ///
        /// [`Emoji`]: struct.Emoji.html
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration
        /// and distinguishing the emoji client-side.
        name: String,
    },
    /// A reaction with a twemoji.
    Unicode(String),
}

impl<'de> Deserialize<'de> for ReactionType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        enum Field {
            Id,
            Name,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                        formatter.write_str("`id` or `name`")
                    }

                    fn visit_str<E: DeError>(self, value: &str) -> StdResult<Field, E> {
                        match value {
                            "id" => Ok(Field::Id),
                            "name" => Ok(Field::Name),
                            _ => Err(DeError::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ReactionTypeVisitor;

        impl<'de> Visitor<'de> for ReactionTypeVisitor {
            type Value = ReactionType;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("enum ReactionType")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> StdResult<Self::Value, V::Error> {
                let mut id = None;
                let mut name = None;

                while let Some(key) = map.next_key()? {
                    match key {
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

                let name = name.ok_or_else(|| DeError::missing_field("name"))?;

                Ok(if let Some(id) = id {
                    ReactionType::Custom {
                        id: id,
                        name: name,
                    }
                } else {
                    ReactionType::Unicode(name)
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "name"];

        deserializer.deserialize_map(ReactionTypeVisitor)
    }
}

#[cfg(any(feature="model", feature="http"))]
impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom { id, ref name } => {
                format!("{}:{}", name, id)
            },
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
    /// # use serenity::model::ChannelId;
    /// # use std::error::Error;
    /// #
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #     let message = ChannelId(0).get_message(0)?;
    /// #
    /// message.react('🍎')?;
    /// #     Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// #     try_main().unwrap();
    /// # }
    /// ```
    fn from(ch: char) -> ReactionType {
        ReactionType::Unicode(ch.to_string())
    }
}

impl From<Emoji> for ReactionType {
    fn from(emoji: Emoji) -> ReactionType {
        ReactionType::Custom {
            id: emoji.id,
            name: emoji.name,
        }
    }
}

impl From<String> for ReactionType {
    fn from(unicode: String) -> ReactionType {
        ReactionType::Unicode(unicode)
    }
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
    /// use serenity::model::ReactionType;
    ///
    /// fn foo<R: Into<ReactionType>>(bar: R) {
    ///     println!("{:?}", bar.into());
    /// }
    ///
    /// foo("🍎");
    /// ```
    fn from(unicode: &str) -> ReactionType {
        ReactionType::Unicode(unicode.to_owned())
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
            ReactionType::Custom { id, ref name } => {
                f.write_char('<')?;
                f.write_char(':')?;
                f.write_str(name)?;
                f.write_char(':')?;
                Display::fmt(&id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
