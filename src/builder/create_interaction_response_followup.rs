use std::collections::HashMap;

use super::{CreateAllowedMentions, CreateEmbed};
use crate::json::Value;
use crate::{http::AttachmentType, utils};

#[derive(Clone, Debug, Default)]
pub struct CreateInteractionResponseFollowup<'a>(
    pub HashMap<&'static str, Value>,
    pub Vec<AttachmentType<'a>>,
);

impl<'a> CreateInteractionResponseFollowup<'a> {
    /// Set the content of the message.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    #[inline]
    pub fn content<D: ToString>(&mut self, content: D) -> &mut Self {
        self._content(content.to_string())
    }

    fn _content(&mut self, content: String) -> &mut Self {
        self.0.insert("content", Value::String(content));
        self
    }

    /// Override the default username of the webhook
    #[inline]
    pub fn username<D: ToString>(&mut self, username: D) -> &mut Self {
        self._username(username.to_string())
    }

    fn _username(&mut self, username: String) -> &mut Self {
        self.0.insert("username", Value::String(username));
        self
    }

    /// Override the default avatar of the webhook
    #[inline]
    pub fn avatar<D: ToString>(&mut self, avatar_url: D) -> &mut Self {
        self._avatar(avatar_url.to_string())
    }

    fn _avatar(&mut self, avatar_url: String) -> &mut Self {
        self.0.insert("avatar_url", Value::String(avatar_url));
        self
    }
    /// Set whether the message is text-to-speech.
    ///
    /// Think carefully before setting this to `true`.
    ///
    /// Defaults to `false`.
    pub fn tts(&mut self, tts: bool) -> &mut Self {
        self.0.insert("tts", Value::from(tts));
        self
    }

    /// Appends a file to the message.
    pub fn add_file<T: Into<AttachmentType<'a>>>(&mut self, file: T) -> &mut Self {
        self.1.push(file.into());
        self
    }

    /// Appends a list of files to the message.
    pub fn add_files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.1.extend(files.into_iter().map(|f| f.into()));
        self
    }

    /// Sets a list of files to include in the message.
    ///
    /// Calling this multiple times will overwrite the file list.
    /// To append files, call `add_file` or `add_files` instead.
    pub fn files<T: Into<AttachmentType<'a>>, It: IntoIterator<Item = T>>(
        &mut self,
        files: It,
    ) -> &mut Self {
        self.1 = files.into_iter().map(|f| f.into()).collect();
        self
    }

    /// Create an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let mut embed = CreateEmbed::default();
        f(&mut embed);
        self.set_embed(embed)
    }

    /// Set an embed for the message.
    pub fn set_embed(&mut self, embed: CreateEmbed) -> &mut Self {
        let map = utils::hashmap_to_json_map(embed.0);
        let embed = Value::from(map);

        self.0.insert("embed", embed);
        self
    }

    /// Set the allowed mentions for the message.
    pub fn allowed_mentions<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateAllowedMentions) -> &mut CreateAllowedMentions,
    {
        let mut allowed_mentions = CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        let map = utils::hashmap_to_json_map(allowed_mentions.0);
        let allowed_mentions = Value::from(map);

        self.0.insert("allowed_mentions", allowed_mentions);
        self
    }
}
