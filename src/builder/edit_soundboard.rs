#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create or edit a [`Soundboard`] for use via a number of model methods.
///
/// These are:
///
/// - [`Guild::edit_soundboard`]
/// - [`PartialGuild::edit_soundboard`]
/// - [`GuildId::edit_soundboard`]
///
/// [Discord docs](https://discord.com/developers/docs/resources/soundboard#soundboard-resource)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditSoundboard<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<EmojiId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_name: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> EditSoundboard<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The name of the soundboard sound to set.
    ///
    /// **Note**: Must be between 2 and 32 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the volume of the soundboard sound.
    ///
    /// **Note**: Must be between 0 to 1.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume.into();
        self
    }

    /// Set the ID of the custom emoji.
    pub fn emoji_id(mut self, id: EmojiId) -> Self {
        self.emoji_id = Some(id);
        self
    }

    /// Set the Unicode character of the custom emoji.
    pub fn emoji_name(mut self, name: String) -> Self {
        self.emoji_name = Some(name);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: &'a str) -> Self {
        self.audit_log_reason = Some(reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for EditSoundboard<'_> {
    type Context<'ctx> = (GuildId, SoundId);
    type Built = Soundboard;

    /// Edits the soundboard sound.
    ///
    /// **Note**: If the soundboard sound was created by the current user, requires either the
    /// [Create Guild Expressions] or the [Manage Guild Expressions] permission. Otherwise, the
    /// [Manage Guild Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().edit_guild_soundboard(ctx.0, ctx.1, &self, self.audit_log_reason).await
    }
}
