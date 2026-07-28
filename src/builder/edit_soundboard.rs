use std::borrow::Cow;

use crate::model::prelude::*;

/// A builder to create or edit a [`Soundboard`] for use with [`GuildId::edit_soundboard`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/soundboard#modify-guild-soundboard-sound)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditSoundboard<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<EmojiId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_name: Option<Cow<'a, str>>,

    #[serde(skip)]
    audit_log_reason: Option<Cow<'a, str>>,
}

impl<'a> EditSoundboard<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// The name of the soundboard sound to set.
    ///
    /// **Note**: Must be between 2 and 32 characters long.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
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
    pub fn emoji_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.emoji_name = Some(name.into());
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.audit_log_reason = Some(reason.into());
        self
    }

    pub fn into_owned(self) -> EditSoundboard<'static> {
        let Self {
            name,
            volume,
            emoji_id,
            emoji_name,
            audit_log_reason,
        } = self;
        EditSoundboard {
            name: name.map(|n| n.into_owned().into()),
            volume,
            emoji_id,
            emoji_name: emoji_name.map(|e| e.into_owned().into()),
            audit_log_reason: audit_log_reason.map(|r| r.into_owned().into()),
        }
    }

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
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        guild_id: GuildId,
        sound_id: SoundId,
    ) -> Result<Soundboard> {
        cache_http
            .http()
            .edit_guild_soundboard(guild_id, sound_id, &self, self.audit_log_reason.as_deref())
            .await
    }
}
