use std::borrow::Cow;

use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/channel#forum-tag-object-forum-tag-structure)
///
/// Contrary to the [`ForumTag`] struct, only the name field is required.
#[must_use]
#[derive(Clone, Debug, Serialize)]
pub struct CreateForumTag<'a> {
    name: Cow<'a, str>,
    moderated: bool,
    emoji_id: Option<EmojiId>,
    emoji_name: Option<Cow<'a, str>>,
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

    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        match emoji.into() {
            ReactionType::Custom {
                id, ..
            } => {
                self.emoji_id = Some(id);
                self.emoji_name = None;
            },
            ReactionType::Unicode(unicode_emoji) => {
                self.emoji_id = None;
                self.emoji_name = Some(unicode_emoji.into_string().into());
            },
        }
        self
    }
}
