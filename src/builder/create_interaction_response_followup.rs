#[cfg(not(feature = "http"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::application::interaction::MessageFlags;
use crate::model::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInteractionResponseFollowup<'a> {
    #[serde(skip)]
    #[cfg(feature = "http")]
    id: Option<MessageId>,
    #[serde(skip)]
    #[cfg(feature = "http")]
    token: &'a str,
    #[cfg(not(feature = "http"))]
    token: PhantomData<&'a ()>,

    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<CreateComponents>,

    #[serde(skip)]
    files: Vec<AttachmentType<'a>>,
}

impl<'a> CreateInteractionResponseFollowup<'a> {
    pub fn new(
        #[cfg(feature = "http")] id: Option<MessageId>,
        #[cfg(feature = "http")] token: &'a str,
    ) -> Self {
        Self {
            #[cfg(feature = "http")]
            id,
            #[cfg(feature = "http")]
            token,
            #[cfg(not(feature = "http"))]
            token: PhantomData::default(),

            embeds: Vec::new(),
            content: None,
            username: None,
            avatar_url: None,
            tts: None,
            allowed_mentions: None,
            flags: None,
            components: None,

            files: Vec::new(),
        }
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Override the default username of the webhook
    #[inline]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Override the default avatar of the webhook
    #[inline]
    pub fn avatar(mut self, avatar_url: impl Into<String>) -> Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = Some(tts);
        self
    }

    /// Appends a file to the message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(self, file: T) -> Self {
        self.add_files(vec![file])
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        mut self,
        files: It,
    ) -> Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Create an embed for the message.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.add_embed(embed)
    }

    /// Adds an embed to the message.
    pub fn add_embed(mut self, embed: CreateEmbed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Adds multiple embeds to the message.
    pub fn add_embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embed(self, embed: CreateEmbed) -> Self {
        self.set_embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embeds(mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> Self {
        self.embeds = embeds.into_iter().collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the flags for the response.
    pub fn flags(mut self, flags: MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    /// Adds or removes the ephemeral flag
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        let flags = self.flags.unwrap_or_else(MessageFlags::empty);

        let flags = if ephemeral {
            flags | MessageFlags::EPHEMERAL
        } else {
            flags & !MessageFlags::EPHEMERAL
        };

        self.flags = Some(flags);
        self
    }

    /// Creates components for this message.
    pub fn components<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(mut self, components: CreateComponents) -> Self {
        self.components = Some(components);
        self
    }

    /// Creates/Edits a followup response to the response sent.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 unicode code points.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Model`] if the message content is too long. May also return an
    /// [`Error::Http`] if invalid data is provided, or an [`Error::Json`] if there is an error
    /// when deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: impl AsRef<Http>) -> Result<Message> {
        self.check_lengths()?;
        self._execute(http.as_ref()).await
    }

    #[cfg(feature = "http")]
    async fn _execute(mut self, http: &Http) -> Result<Message> {
        let files = std::mem::take(&mut self.files);

        match self.id {
            Some(id) => {
                if files.is_empty() {
                    http.edit_followup_message(&self.token, id.into(), &self).await
                } else {
                    http.edit_followup_message_and_attachments(&self.token, id.into(), &self, files)
                        .await
                }
            },
            None => {
                if files.is_empty() {
                    http.create_followup_message(&self.token, &self).await
                } else {
                    http.create_followup_message_with_files(&self.token, &self, files).await
                }
            },
        }
    }

    #[cfg(feature = "http")]
    fn check_lengths(&self) -> Result<()> {
        if let Some(ref content) = self.content {
            let length = content.chars().count();
            let max_length = constants::MESSAGE_CODE_LIMIT;
            if length > max_length {
                let overflow = length - max_length;
                return Err(Error::Model(ModelError::MessageTooLong(overflow)));
            }
        }

        if self.embeds.len() > constants::EMBED_MAX_COUNT {
            return Err(Error::Model(ModelError::EmbedAmount));
        }
        for embed in &self.embeds {
            embed.check_length()?;
        }
        Ok(())
    }
}
