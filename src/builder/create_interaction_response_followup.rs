use std::borrow::Cow;

use super::create_poll::Ready;
use super::{
    CreateActionRow,
    CreateAllowedMentions,
    CreateAttachment,
    CreateEmbed,
    CreatePoll,
    EditAttachments,
};
#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/interactions/receiving-and-responding#create-followup-message)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateInteractionResponseFollowup<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Cow<'a, str>>,
    // [Omitting username: not supported in interaction followups]
    // [Omitting avatar_url: not supported in interaction followups]
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
    embeds: Option<Cow<'a, [CreateEmbed<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<CreateAllowedMentions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Cow<'a, [CreateActionRow<'a>]>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<MessageFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    poll: Option<CreatePoll<'a, Ready>>,
    attachments: EditAttachments<'a>,
}

impl<'a> CreateInteractionResponseFollowup<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http")]
    fn check_length(&self) -> Result<(), ModelError> {
        super::check_lengths(self.content.as_deref(), self.embeds.as_deref(), 0)
    }

    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
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
    pub fn add_file(mut self, file: CreateAttachment<'a>) -> Self {
        self.attachments = self.attachments.add(file);
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        for file in files {
            self.attachments = self.attachments.add(file);
        }
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list. To append files, call
    /// [`Self::add_file`] or [`Self::add_files`] instead.
    pub fn files(mut self, files: impl IntoIterator<Item = CreateAttachment<'a>>) -> Self {
        self.attachments = EditAttachments::new();
        self.add_files(files)
    }

    /// Adds an embed to the message.
    pub fn add_embed(mut self, embed: CreateEmbed<'a>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().push(embed);
        self
    }

    /// Adds multiple embeds to the message.
    pub fn add_embeds(mut self, embeds: impl IntoIterator<Item = CreateEmbed<'a>>) -> Self {
        self.embeds.get_or_insert_with(Cow::default).to_mut().extend(embeds);
        self
    }

    /// Sets a single embed to include in the message.
    ///
    /// Calling this will overwrite the embed list. To append embeds, call [`Self::add_embed`]
    /// instead.
    pub fn embed(self, embed: CreateEmbed<'a>) -> Self {
        self.embeds(vec![embed])
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list. To append embeds, call
    /// [`Self::add_embeds`] instead.
    pub fn embeds(mut self, embeds: impl Into<Cow<'a, [CreateEmbed<'a>]>>) -> Self {
        self.embeds = Some(embeds.into());
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions(mut self, allowed_mentions: CreateAllowedMentions<'a>) -> Self {
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
        }

        self.flags = Some(flags);
        self
    }

    /// Adds a poll to the message. Only one poll can be added per message.
    ///
    /// See [`CreatePoll`] for more information on creating and configuring a poll.
    pub fn poll(mut self, poll: CreatePoll<'a, Ready>) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Sets the components of this message.
    pub fn components(mut self, components: impl Into<Cow<'a, [CreateActionRow<'a>]>>) -> Self {
        self.components = Some(components.into());
        self
    }
    super::button_and_select_menu_convenience_methods!(self.components);

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
    #[cfg(feature = "http")]
    pub async fn execute(
        mut self,
        http: &Http,
        message_id: Option<MessageId>,
        interaction_token: &str,
    ) -> Result<Message> {
        self.check_length()?;

        let files = self.attachments.take_files();

        if self.allowed_mentions.is_none() {
            self.allowed_mentions.clone_from(&http.default_allowed_mentions);
        }

        match message_id {
            Some(id) => http.edit_followup_message(interaction_token, id, &self, files).await,
            None => http.create_followup_message(interaction_token, &self, files).await,
        }
    }
}
