#[cfg(feature = "http")]
use super::Builder;
use super::CreateAttachment;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to create a soundboard sound.
///
/// [Discord docs](https://discord.com/developers/docs/resources/soundboard#soundboard-resource)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateSoundboard<'a> {
    name: String,
    sound: String,
    volume: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<EmojiId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_name: Option<String>,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,
}

impl<'a> CreateSoundboard<'a> {
    /// Creates a new builder with the given data.
    pub fn new(name: impl Into<String>, sound: &CreateAttachment) -> Self {
        Self {
            name: name.into(),
            sound: sound.to_base64(),
            volume: 1.0,
            emoji_id: None,
            emoji_name: None,
            audit_log_reason: None,
        }
    }

    /// Set the name of the soundboard sound, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 2 and 32 characters long.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the sound file. Replaces the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be audio that is encoded in MP3 or OGG, max 512 KB, max
    /// duration 5.2 seconds.
    pub fn sound(mut self, sound: &CreateAttachment) -> Self {
        self.sound = sound.to_base64();
        self
    }

    /// Set the volume of the soundboard sound.
    ///
    /// **Note**: Must be between 0 to 1.
    pub fn volume(mut self, volume: f64) -> Self {
        self.volume = volume;
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
impl Builder for CreateSoundboard<'_> {
    type Context<'ctx> = GuildId;
    type Built = Soundboard;

    /// Creates a new soundboard in the guild with the data set.
    ///
    /// **Note**: Requires the [Create Guild Expressions] permission.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(
            &cache_http,
            ctx,
            Permissions::CREATE_GUILD_EXPRESSIONS,
        )?;

        cache_http.http().create_guild_soundboard(ctx, &self, self.audit_log_reason).await
    }
}
