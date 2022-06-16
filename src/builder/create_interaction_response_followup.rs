#[cfg(not(feature = "model"))]
use std::marker::PhantomData;

use super::{CreateAllowedMentions, CreateComponents, CreateEmbed};
use crate::model::application::interaction::MessageFlags;
#[cfg(feature = "model")]
use crate::model::channel::AttachmentType;

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateInteractionResponseFollowup<'a> {
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
    #[cfg(feature = "model")]
    pub(crate) files: Vec<AttachmentType<'a>>,
    #[cfg(not(feature = "model"))]
    pub(crate) files: PhantomData<&'a ()>,
}

impl<'a> CreateInteractionResponseFollowup<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    /// Override the default username of the webhook
    #[inline]
    pub fn username(&mut self, username: impl Into<String>) -> &mut Self {
        self.username = Some(username.into());
        self
    }

    /// Override the default avatar of the webhook
    #[inline]
    pub fn avatar(&mut self, avatar_url: impl Into<String>) -> &mut Self {
        self.avatar_url = Some(avatar_url.into());
        self
    }

    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.tts = Some(tts);
        self
    }

    /// Appends a file to the message.
    #[cfg(feature = "model")]
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.add_files(vec![file]);
        self
    }

    /// Appends a list of files to the message.
    #[cfg(feature = "model")]
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call [`Self::add_file`] or [`Self::add_files`] instead.
    #[cfg(feature = "model")]
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.files = files.into_iter().map(Into::into).collect();
        self
    }

    /// Create an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.add_embed(embed)
    }

    /// Adds an embed to the message.
    pub fn add_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.embeds.push(embed);
        self
    }

    /// Adds multiple embeds to the message.
    pub fn add_embeds(&mut self, embeds: Vec<CreateEmbed>) -> &mut Self {
        self.embeds.extend(embeds);
        self
    }

    /// Sets a single embed to include in the message
    ///
    /// Calling this will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        self.set_embeds(vec![embed]);
        self
    }

    /// Sets a list of embeds to include in the message.
    ///
    /// Calling this multiple times will overwrite the embed list.
    /// To append embeds, call [`Self::add_embed`] instead.
    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        self.embeds = embeds.into_iter().collect();
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);

        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Sets the flags for the response.
    pub fn flags(&mut self, flags: MessageFlags) -> &mut Self {
        self.flags = Some(flags);
        self
    }

    /// Adds or removes the ephemeral flag
    pub fn ephemeral(&mut self, ephemeral: bool) -> &mut Self {
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
    pub fn components<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateComponents) -> &mut CreateComponents,
    {
        let mut components = CreateComponents::default();
        f(&mut components);

        self.set_components(components)
    }

    /// Sets the components of this message.
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.components = Some(components);
        self
    }
}
