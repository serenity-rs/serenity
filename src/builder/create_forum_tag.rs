use std::borrow::Cow;

use crate::model::prelude::*;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum StrOrChar<'a> {
    Char(char),
    Str(Cow<'a, str>),
}

/// [Discord docs](https://discord.com/developers/docs/resources/channel#forum-tag-object-forum-tag-structure)
///
/// Contrary to the [`ForumTag`] struct, only the name field is required.
#[must_use]
#[derive(Clone, Debug, Serialize)]
pub struct CreateForumTag<'a> {
    name: Cow<'a, str>,
    moderated: bool,
    emoji_id: Option<EmojiId>,
    emoji_name: Option<StrOrChar<'a>>,
}

impl<'a> CreateForumTag<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            moderated: false,
            emoji_id: None,
            emoji_name: None,
        }
    }

    pub fn moderated(mut self, moderated: bool) -> Self {
        self.moderated = moderated;
        self
    }

    pub fn emoji(mut self, emoji: impl Into<ReactionType<'a>>) -> Self {
        let (emoji_id, emoji_name) = match emoji.into() {
            ReactionType::Custom {
                id, ..
            } => (Some(id), None),
            ReactionType::Unicode(unicode_emoji) => (None, Some(StrOrChar::Str(unicode_emoji))),
            ReactionType::UnicodeChar(unicode_char) => (None, Some(StrOrChar::Char(unicode_char))),
        };

        self.emoji_name = emoji_name;
        self.emoji_id = emoji_id;
        self
    }
}
