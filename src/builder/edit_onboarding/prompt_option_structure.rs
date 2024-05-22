use crate::model::channel::ReactionType;
use crate::model::id::{ChannelId, RoleId};

#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsChannels;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsRoles;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsTitle;
#[derive(serde::Serialize, Clone, Debug)]
pub struct Ready;

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl Sealed for NeedsChannels {}
    impl Sealed for NeedsRoles {}
    impl Sealed for NeedsTitle {}
    impl Sealed for Ready {}
}

use sealed::*;

#[derive(serde::Serialize, Clone, Debug)]
#[must_use = "Builders do nothing unless built"]
pub struct CreatePromptOption<Stage: Sealed> {
    channel_ids: Vec<ChannelId>,
    role_ids: Vec<RoleId>,
    emoji: Option<ReactionType>,
    title: String,
    description: Option<String>,

    #[serde(skip)]
    _stage: Stage,
}

impl Default for CreatePromptOption<NeedsChannels> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            channel_ids: Vec::new(),
            role_ids: Vec::new(),
            emoji: None,
            title: String::new(),
            description: None,

            _stage: NeedsChannels,
        }
    }
}

impl CreatePromptOption<NeedsChannels> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn channels(self, channel_ids: Vec<ChannelId>) -> CreatePromptOption<NeedsRoles> {
        CreatePromptOption {
            channel_ids,
            role_ids: self.role_ids,
            emoji: self.emoji,
            title: self.title,
            description: self.description,

            _stage: NeedsRoles,
        }
    }
}

impl CreatePromptOption<NeedsRoles> {
    pub fn roles(self, role_ids: Vec<RoleId>) -> CreatePromptOption<NeedsTitle> {
        CreatePromptOption {
            channel_ids: self.channel_ids,
            role_ids,
            emoji: self.emoji,
            title: self.title,
            description: self.description,

            _stage: NeedsTitle,
        }
    }
}

impl CreatePromptOption<NeedsTitle> {
    pub fn title(self, title: impl Into<String>) -> CreatePromptOption<Ready> {
        CreatePromptOption {
            channel_ids: self.channel_ids,
            role_ids: self.role_ids,
            emoji: self.emoji,
            title: title.into(),
            description: self.description,

            _stage: Ready,
        }
    }
}

impl<Stage: Sealed> CreatePromptOption<Stage> {
    pub fn emoji(mut self, emoji: ReactionType) -> Self {
        self.emoji = Some(emoji);
        self
    }

    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }
}
