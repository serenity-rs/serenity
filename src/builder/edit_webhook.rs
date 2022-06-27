#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[derive(Debug, Serialize)]
#[must_use]
pub struct EditWebhook<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    webhook: &'a mut Webhook,
    #[cfg(not(feature = "http"))]
    webhook: PhantomData<&'a ()>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_id: Option<ChannelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<Option<String>>,
}

impl<'a> EditWebhook<'a> {
    pub fn new(#[cfg(feature = "http")] webhook: &'a mut Webhook) -> Self {
        Self {
            #[cfg(feature = "http")]
            webhook,
            #[cfg(not(feature = "http"))]
            webhook: PhantomData::default(),

            name: None,
            channel_id: None,
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

    /// Set the channel to move the webhook to.
    pub fn channel_id(mut self, channel_id: impl Into<ChannelId>) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Set the webhook's default avatar.
    ///
    /// # Errors
    ///
    /// May error if a URL is given and the HTTP request fails, or if a path is given to a file
    /// that does not exist.
    #[cfg(featuer = "http")]
    pub async fn avatar(
        mut self,
        http: impl AsRef<Http>,
        avatar: impl Into<AttachmentType<'a>>,
    ) -> Result<Self> {
        let avatar_data = avatar.into().data(&http.as_ref().client).await?;
        self.avatar = Some(Some(encode_image(&avatar_data)));
        Ok(self)
    }

    /// Set the webhook's default avatar. Requires the input be a base64-encoded image that is in
    /// either JPG, GIF, or PNG format.
    #[cfg(not(feature = "http"))]
    pub fn avatar(mut self, avatar: String) -> Self {
        self.avatar = Some(Some(avatar));
        self
    }

    /// Deletes the webhook's avatar, resetting it to the default logo.
    pub fn delete_avatar(mut self) -> Self {
        self.avatar = Some(None);
        self
    }

    /// Sends off the request and edits the webhook. Does not require authentication, as a token is
    /// required.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the token field of the current webhook is `None`.
    ///
    /// May also return an [`Error::Http`] if the content is malformed, or if the token is invalid.
    ///
    /// Or may return an [`Error::Json`] if there is an error in deserialising Discord's response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<()> {
        let token = self.webhook.token.as_ref().ok_or(ModelError::NoTokenSet)?;
        *self.webhook =
            http.as_ref().edit_webhook_with_token(self.webhook.id.into(), token, &self).await?;
        Ok(())
    }
}
