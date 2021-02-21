use std::collections::HashMap;

use crate::json::{from_number, json};
use crate::json::{Value, NULL};
use crate::model::channel::{PermissionOverwrite, PermissionOverwriteType};
use crate::model::id::ChannelId;

/// A builder to edit a [`GuildChannel`] for use via [`GuildChannel::edit`]
///
/// Defaults are not directly provided by the builder itself.
///
/// # Examples
///
/// Edit a channel, providing a new name and topic:
///
/// ```rust,no_run
/// # use serenity::{http::Http, model::id::ChannelId};
/// #
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// #     let http = Http::default();
/// #     let mut channel = ChannelId(0);
/// // assuming a channel has already been bound
/// if let Err(why) = channel.edit(&http, |c| c.name("new name").topic("a test topic")).await {
///     // properly handle the error
/// }
/// #     Ok(())
/// # }
/// ```
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`GuildChannel::edit`]: crate::model::channel::GuildChannel::edit
#[derive(Clone, Debug, Default)]
pub struct EditChannel(pub HashMap<&'static str, Value>);

impl EditChannel {
    /// The bitrate of the channel in bits.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn bitrate(&mut self, bitrate: u64) -> &mut Self {
        self.0.insert("bitrate", from_number(bitrate));
        self
    }

    /// The name of the channel.
    ///
    /// Must be between 2 and 100 characters long.
    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));
        self
    }

    /// The position of the channel in the channel list.
    pub fn position(&mut self, position: u64) -> &mut Self {
        self.0.insert("position", from_number(position));
        self
    }

    /// The topic of the channel. Can be empty.
    ///
    /// Must be between 0 and 1024 characters long.
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    pub fn topic<S: ToString>(&mut self, topic: S) -> &mut Self {
        self.0.insert("topic", Value::from(topic.to_string()));
        self
    }

    /// Is the channel inappropriate for work?
    ///
    /// This is for [text] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.0.insert("nsfw", Value::from(nsfw));

        self
    }

    /// The number of users that may be in the channel simultaneously.
    ///
    /// This is for [voice] channels only.
    ///
    /// [voice]: crate::model::channel::ChannelType::Voice
    pub fn user_limit(&mut self, user_limit: u64) -> &mut Self {
        self.0.insert("user_limit", from_number(user_limit));
        self
    }

    /// The parent category of the channel.
    ///
    /// This is for [text] and [voice] channels only.
    ///
    /// [text]: crate::model::channel::ChannelType::Text
    /// [voice]: crate::model::channel::ChannelType::Voice
    #[inline]
    pub fn category<C: Into<Option<ChannelId>>>(&mut self, category: C) -> &mut Self {
        self._category(category.into());
        self
    }

    fn _category(&mut self, category: Option<ChannelId>) {
        self.0.insert("parent_id", match category {
            Some(c) => Value::from(c.0),
            None => NULL,
        });
    }

    /// The seconds a user has to wait before sending another message.
    ///
    /// **Info**: Only values from 0 to 120 are valid.
    #[inline]
    pub fn slow_mode_rate(&mut self, seconds: u64) -> &mut Self {
        self.0.insert("rate_limit_per_user", from_number(seconds));

        self
    }

    /// A set of overwrites defining what a user or a user carrying a certain role can
    /// and cannot do.
    ///
    /// # Example
    ///
    /// Inheriting permissions from an exisiting channel:
    ///
    /// ```rust,no_run
    /// # use serenity::{http::Http, model::id::ChannelId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Arc::new(Http::default());
    /// #     let mut channel = ChannelId(0);
    /// use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::id::UserId;
    /// use serenity::model::permissions::Permissions;
    ///
    /// // Assuming a channel has already been bound.
    /// let permissions = vec![PermissionOverwrite {
    ///     allow: Permissions::READ_MESSAGES,
    ///     deny: Permissions::SEND_TTS_MESSAGES,
    ///     kind: PermissionOverwriteType::Member(UserId(1234)),
    /// }];
    ///
    /// channel.edit(http, |c| {
    ///     c.name("my_edited_cool_channel")
    ///     .permissions(permissions)
    /// })
    /// .await?;
    /// #    Ok(())
    /// # }
    /// ```
    pub fn permissions<I>(&mut self, perms: I) -> &mut Self
    where
        I: IntoIterator<Item = PermissionOverwrite>,
    {
        let overwrites = perms
            .into_iter()
            .map(|perm| {
                let (id, kind) = match perm.kind {
                    PermissionOverwriteType::Member(id) => (id.0, "member"),
                    PermissionOverwriteType::Role(id) => (id.0, "role"),
                };

                json!({
                    "allow": perm.allow.bits(),
                    "deny": perm.deny.bits(),
                    "id": id,
                    "type": kind,
                })
            })
            .collect();

        self.0.insert("permission_overwrites", Value::Array(overwrites));

        self
    }
}
