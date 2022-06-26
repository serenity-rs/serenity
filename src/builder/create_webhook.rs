#[cfg(feature = "http")]
use crate::http::{CacheHttp, Http};
#[cfg(feature = "http")]
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateWebhook {
    #[cfg(feature = "http")]
    #[serde(skip)]
    id: ChannelId,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
}

impl CreateWebhook {
    pub fn new(#[cfg(feature = "http")] id: ChannelId) -> Self {
        Self {
            #[cfg(feature = "http")]
            id,
            name: None,
            avatar: None,
        }
    }

    /// Set the webhook's name.
    ///
    /// This must be between 1-80 characters.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the webhook's default avatar. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    pub fn avatar(mut self, avatar: String) -> Self {
        self.avatar = Some(avatar);
        self
    }

    /// Creates the webhook.
    ///
    /// # Errors
    ///
    /// Returns a [`Error::Http`] if the current user lacks permission.
    /// Returns a [`ModelError::NameTooShort`] if the name of the webhook is
    /// under the limit of 2 characters.
    /// Returns a [`ModelError::NameTooLong`] if the name of the webhook is
    /// over the limit of 100 characters.
    /// Returns a [`ModelError::InvalidChannelType`] if the channel type is not text.
    #[cfg(feature = "http")]
    pub async fn execute(self, cache_http: impl CacheHttp) -> Result<Webhook> {
        #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                if let Some(channel) = cache.guild_channel(self.id) {
                    if !channel.is_text_based() {
                        return Err(Error::Model(ModelError::InvalidChannelType));
                    }
                }
            }
        }

        self._execute(cache_http.http()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(self, http: &Http) -> Result<Webhook> {
        if let Some(ref name) = self.name {
            if name.len() < 2 {
                return Err(Error::Model(ModelError::NameTooShort));
            } else if name.len() > 100 {
                return Err(Error::Model(ModelError::NameTooLong));
            }
        }

        http.as_ref().create_webhook(self.id.into(), &self, None).await
    }
}
