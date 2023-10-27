#[cfg(feature = "http")]
use super::{check_overflow, Builder};
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::constants;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#create-followup-message)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInteractionResponseFollowup {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    // [Omitting username: not supported in interaction followups]
    // [Omitting avatar_url: not supported in interaction followups]
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    embeds: Vec<CreateEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Vec<CreateActionRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    attachments: EditAttachments,
}

impl CreateInteractionResponseFollowup {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<()> {
        if let Some(content) = &self.content {
            check_overflow(content.chars().count(), constants::MESSAGE_CODE_LIMIT)
                .map_err(|overflow| Error::Model(ModelError::MessageTooLong(overflow)))?;
        }

        check_overflow(self.embeds.len(), constants::EMBED_MAX_COUNT)
            .map_err(|_| Error::Model(ModelError::EmbedAmount))?;
        for embed in &self.embeds {
            embed.check_length()?;
        }

        Ok(())
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
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
    pub fn add_file(mut self, file: CreateAttachment) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
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

    /// Sets a single embed to include in the message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed) -> Self {
        self.embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list. To append embeds, call
    /// [`Self::add_embeds`] instead.
    pub fn embeds(mut self, embeds: Vec<CreateEmbed>) -> Self {
        self.embeds = embeds;
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions) -> Self {
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
        let mut flags = self.flags.unwrap_or_else(MessageFlags::empty);

        if ephemeral {
            flags |= MessageFlags::EPHEMERAL;
        } else {
            flags &= !MessageFlags::EPHEMERAL;
        };

        self.flags = Some(flags);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: Vec<CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for CreateInteractionResponseFollowup {
    type Context<'ctx> = (Option<MessageId>, &'ctx str);
    type Built = Message;

    /// Creates or edits a followup response to the response sent. If a [`MessageId`] is provided,
    /// then the corresponding message will be edited. Otherwise, a new message will be created.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points, and embeds must be under
    /// 6000 code points.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Model`] if the content is too long. May also return [`Error::Http`] if the
    /// API returns an error, or [`Error::Json`] if there is an error in deserializing the
    /// response.
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        self.check_length()?;

        let files = self.attachments.take_files();

        let http = cache_http.http();
        match ctx.0 {
            Some(id) => http.as_ref().edit_followup_message(ctx.1, id, &self, files).await,
            None => http.as_ref().create_followup_message(ctx.1, &self, files).await,
        }
    }
}
