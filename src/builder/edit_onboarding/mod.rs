#[cfg(feature = "http")]
use super::Builder;

use crate::model::guild::OnboardingMode;
use crate::model::id::ChannelId;

#[cfg(feature = "http")]
use crate::model::id::GuildId;
#[cfg(feature = "http")]
use crate::model::Permissions;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::model::guild::Onboarding;
#[cfg(feature = "http")]
use crate::internal::prelude::*;

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
pub struct EditOnboarding<'a, Stage: Sealed> {
    prompts: Vec<CreateOnboardingPrompt<prompt_structure::Ready>>,
    default_channel_ids: Vec<ChannelId>,
    enabled: bool,
    mode: OnboardingMode,

    #[serde(skip)]
    audit_log_reason: Option<&'a str>,

    #[serde(skip)]
    _stage: Stage,
}

impl<'a> Default for EditOnboarding<'a, NeedsPrompts> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            prompts: Vec::new(),
            default_channel_ids: Vec::new(),
            enabled: true,
            mode: OnboardingMode::default(),
            audit_log_reason: None,

            _stage: NeedsPrompts,
        }
    }
}

impl<'a> EditOnboarding<'a, NeedsPrompts> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prompts(
        self,
        prompts: Vec<CreateOnboardingPrompt<prompt_structure::Ready>>,
    ) -> EditOnboarding<'a, NeedsChannels> {
        EditOnboarding {
            prompts,
            default_channel_ids: self.default_channel_ids,
            enabled: self.enabled,
            mode: self.mode,
            audit_log_reason: self.audit_log_reason,

            _stage: NeedsChannels,
        }
    }
}

impl<'a> EditOnboarding<'a, NeedsChannels> {
    pub fn default_channels(
        self,
        default_channel_ids: Vec<ChannelId>,
    ) -> EditOnboarding<'a, NeedsEnabled> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids,
            enabled: self.enabled,
            mode: self.mode,
            audit_log_reason: self.audit_log_reason,

            _stage: NeedsEnabled,
        }
    }
}

impl<'a> EditOnboarding<'a, NeedsEnabled> {
    pub fn enabled(self, enabled: bool) -> EditOnboarding<'a, NeedsMode> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids: self.default_channel_ids,
            enabled,
            mode: self.mode,
            audit_log_reason: self.audit_log_reason,

            _stage: NeedsMode,
        }
    }
}

impl<'a> EditOnboarding<'a, NeedsMode> {
    pub fn mode(self, mode: OnboardingMode) -> EditOnboarding<'a, Ready> {
        EditOnboarding {
            prompts: self.prompts,
            default_channel_ids: self.default_channel_ids,
            enabled: self.enabled,
            mode,
            audit_log_reason: self.audit_log_reason,

            _stage: Ready,
        }
    }
}

impl<'a, Stage: Sealed> EditOnboarding<'a, Stage> {
    pub fn emoji(mut self, audit_log_reason: &'a str) -> Self {
        self.audit_log_reason = Some(audit_log_reason);
        self
    }
}


#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for EditOnboarding<'a, Ready> {
    type Context<'ctx> = GuildId;
    type Built = Onboarding;

    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(&cache_http, ctx, Permissions::MANAGE_GUILD | Permissions::MANAGE_ROLES)?;

        cache_http.http().set_guild_onboarding(ctx, &self, self.audit_log_reason).await
    }
}
