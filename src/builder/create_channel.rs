use std::collections::HashMap;

use crate::json::from_number;
use crate::json::json;
use crate::json::Value;
use crate::model::prelude::*;

/// A builder for creating a new [`GuildChannel`] in a [`Guild`].
///
/// Except [`name`], all fields are optional.
///
/// [`GuildChannel`]: crate::model::channel::GuildChannel
/// [`Guild`]: crate::model::guild::Guild
/// [`name`]: Self::name
#[derive(Debug, Clone)]
pub struct CreateChannel(pub HashMap<&'static str, Value>);

impl CreateChannel {
    /// Specify how to call this new channel.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::from(name.to_string()));

        self
    }
    /// Specify what type the channel is, whether it's a text, voice, category or news channel.
    pub fn kind(&mut self, kind: ChannelType) -> &mut Self {
        self.0.insert("type", from_number(kind as u8));

        self
    }

    /// Specifiy the category, the "parent" of this channel.
    pub fn category<I: Into<ChannelId>>(&mut self, id: I) -> &mut Self {
        self.0.insert("parent_id", from_number(id.into().0));

        self
    }

    /// Set an interesting topic.
    ///
    /// **Note**: Must be between 0 and 1000 characters long.
    pub fn topic<D: ToString>(&mut self, topic: D) -> &mut Self {
        self.0.insert("topic", Value::from(topic.to_string()));

        self
    }

    /// Specify if this channel will be inappropriate to browse while at work.
    pub fn nsfw(&mut self, b: bool) -> &mut Self {
        self.0.insert("nsfw", Value::from(b));

        self
    }

    /// [Voice-only] Specify the bitrate at which sound plays in the voice channel.
    pub fn bitrate(&mut self, rate: u32) -> &mut Self {
        self.0.insert("bitrate", from_number(rate));

        self
    }

    /// [Voice-only] Set how many users may occupy this voice channel.
    pub fn user_limit(&mut self, limit: u32) -> &mut Self {
        self.0.insert("user_limit", from_number(limit));

        self
    }

    /// How many seconds must a user wait before sending another message.
    ///
    /// Bots, or users with the `MANAGE_MESSAGES` and/or`MANAGE_CHANNEL` permissions are exempt
    /// from this restriction.
    ///
    /// **Note**: Must be between 0 and 21600 seconds (360 minutes or 6 hours).
    pub fn rate_limit(&mut self, limit: u64) -> &mut Self {
        self.0.insert("rate_limit_per_user", from_number(limit));

        self
    }

    /// Specify where the channel should be located.
    pub fn position(&mut self, pos: u32) -> &mut Self {
        self.0.insert("position", from_number(pos));

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
    /// # use serenity::{http::Http, model::id::GuildId};
    /// # use std::sync::Arc;
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// #     let http = Arc::new(Http::default());
    /// #     let mut guild = GuildId(0).to_partial_guild(&http).await?;
    /// use serenity::model::channel::{PermissionOverwrite, PermissionOverwriteType};
    /// use serenity::model::id::UserId;
    /// use serenity::model::permissions::Permissions;
    ///
    /// // Assuming a guild has already been bound.
    /// let permissions = vec![PermissionOverwrite {
    ///     allow: Permissions::READ_MESSAGES,
    ///     deny: Permissions::SEND_TTS_MESSAGES,
    ///     kind: PermissionOverwriteType::Member(UserId(1234)),
    /// }];
    ///
    /// guild.create_channel(http, |c| {
    ///     c.name("my_new_cool_channel")
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

impl Default for CreateChannel {
    /// Creates a builder with default values, setting `kind` to `ChannelType::Text`.
    ///
    /// # Examples
    ///
    /// Create a default `CreateChannel` builder:
    ///
    /// ```rust
    /// use serenity::builder::CreateChannel;
    ///
    /// let channel_builder = CreateChannel::default();
    /// ```
    fn default() -> Self {
        let mut builder = CreateChannel(HashMap::new());
        builder.kind(ChannelType::Text);

        builder
    }
}
