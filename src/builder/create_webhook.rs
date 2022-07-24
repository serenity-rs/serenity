#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Debug, Default, Clone, Serialize)]
#[must_use]
pub struct CreateWebhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
}

impl CreateWebhook {
    /// Creates the webhook.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidChannelType`] if the
    /// corresponding channel is not of type [`Text`] or [`News`].
    ///
    /// If the provided name is less than 2 characters, returns [`ModelError::NameTooShort`]. If it
    /// is more than 100 characters, returns [`ModelError::NameTooLong`].
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission, or if invalid data is
    /// given.
    ///
    /// [`Text`]: ChannelType::Text
    /// [`News`]: ChannelType::News
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        cache_http: impl CacheHttp,
        channel_id: ChannelId,
    ) -> Result<Webhook> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(channel_id) {
                    if !channel.is_text_based() {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self._execute(cache_http.http(), channel_id).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http, channel_id: ChannelId) -> Result<Webhook> {
        if let Some(name) = &self.name {
            if name.len() < 2 {
                return Err(Error::Model(ModelError::NameTooShort));
            } else if name.len() > 100 {
                return Err(Error::Model(ModelError::NameTooLong));
            }
        }

        http.create_webhook(channel_id.into(), &self, None).await
    }

    /// Set the webhook's name.
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the webhook's default avatar.
    ///
    /// # Errors
    ///
    /// May error if the input is a URL and the HTTP request fails, or if it is a path to a file
    /// that does not exist.
    #[cfg(feature = "http")]
    pub async fn avatar<'a>(
        mut self,
        http: impl AsRef<Http>,
        avatar: impl Into<AttachmentType<'a>>,
    ) -> Result<Self> {
        let avatar_data = avatar.into().data(&http.as_ref().client).await?;
        self.avatar = Some(crate::utils::encode_image(&avatar_data));
        Ok(self)
    }

    #[cfg(not(feature = "http"))]
    /// Set the webhook's default avatar. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    pub fn avatar(mut self, avatar: String) -> Self {
        self.avatar = Some(avatar);
        self
    }
}
