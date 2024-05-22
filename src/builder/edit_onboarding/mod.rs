use crate::model::guild::OnboardingMode;
use crate::model::id::ChannelId;
mod prompt_option_structure;
mod prompt_structure;

pub use prompt_option_structure::CreatePromptOption;
pub use prompt_structure::CreateOnboardingPrompt;

#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsPrompts;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsChannels;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsEnabled;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsMode;
#[derive(serde::Serialize, Clone, Debug)]
pub struct Ready;

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl Sealed for NeedsPrompts {}
    impl Sealed for NeedsChannels {}
    impl Sealed for NeedsEnabled {}
    impl Sealed for NeedsMode {}
    impl Sealed for Ready {}
}

use sealed::*;

#[derive(serde::Serialize, Clone, Debug)]
#[must_use = "Builders do nothing unless built"]
pub struct EditOnboarding<Stage: Sealed> {
    prompts: Vec<CreateOnboardingPrompt<prompt_structure::Ready>>,
    default_channel_ids: Vec<ChannelId>,
    enabled: bool,
    mode: OnboardingMode,

    #[serde(skip)]
    _stage: Stage,
}

impl Default for EditOnboarding<NeedsPrompts> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            prompts: Vec::new(),
            default_channel_ids: Vec::new(),
            enabled: true,
            mode: OnboardingMode::default(),

            _stage: NeedsPrompts,
        }
    }
}

impl EditOnboarding<NeedsPrompts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prompts(
        self,
        prompts: Vec<CreateOnboardingPrompt<prompt_structure::Ready>>,
    ) -> EditOnboarding<NeedsChannels> {
        EditOnboarding {
            prompts,
            default_channel_ids: self.default_channel_ids,
            enabled: self.enabled,
            mode: self.mode,

            _stage: NeedsChannels,
        }
    }
}

impl EditOnboarding<NeedsChannels> {
    pub fn default_channels(
        self,
        default_channel_ids: Vec<ChannelId>,
    ) -> EditOnboarding<NeedsEnabled> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids,
            enabled: self.enabled,
            mode: self.mode,

            _stage: NeedsEnabled,
        }
    }
}

impl EditOnboarding<NeedsEnabled> {
    pub fn enabled(self, enabled: bool) -> EditOnboarding<NeedsMode> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids: self.default_channel_ids,
            enabled,
            mode: self.mode,

            _stage: NeedsMode,
        }
    }
}

impl EditOnboarding<NeedsMode> {
    pub fn mode(self, mode: OnboardingMode) -> EditOnboarding<Ready> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids: self.default_channel_ids,
            enabled: self.enabled,
            mode,

            _stage: Ready,
        }
    }
}
