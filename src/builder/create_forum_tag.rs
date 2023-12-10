use crate::internal::prelude::*;
use crate::model::prelude::*;

/// [Discord docs](https://discord.com/developers/docs/resources/channel#forum-tag-object-forum-tag-structure)
///
/// Contrary to the [`ForumTag`] struct, only the name field is required.
#[must_use]
#[derive(Clone, Debug, Serialize)]
pub struct CreateForumTag {
    name: FixedString,
    moderated: bool,
    emoji_id: Option<EmojiId>,
    emoji_name: Option<FixedString>,
}

impl CreateForumTag {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into().into(),
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
                self.emoji_name = Some(unicode_emoji);
            },
        }
        self
    }
}
