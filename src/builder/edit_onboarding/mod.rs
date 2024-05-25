#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::guild::{Onboarding, OnboardingMode};
use crate::model::prelude::*;
#[cfg(feature = "http")]
use crate::model::Permissions;

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

    /// The onboarding prompts that users can select the options of.
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
    /// The list of default channels the user will have regardless of the answers given.
    ///
    /// There are restrictions that apply only when onboarding is enabled, but these vary depending
    /// on the current [Self::mode].
    ///
    /// If the default mode is set, you must provide at least 7 channels, 5 of which must allow
    /// @everyone to read and send messages. if advanced is set, the restrictions apply across the
    /// default channels and the [Self::prompts], provided that they supply the remaining required
    /// channels.
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
    /// Whether onboarding is enabled or not.
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
    /// The current onboarding mode that controls where the readable channels are set.
    ///
    /// If the default mode is set, you must provide at least 7 channels, 5 of which must allow
    /// @everyone to read and send messages. if advanced is set, the restrictions apply across the
    /// default channels and the [Self::prompts], provided that they supply the remaining required
    /// channels.
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
    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, audit_log_reason: &'a str) -> Self {
        self.audit_log_reason = Some(audit_log_reason);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl<'a> Builder for EditOnboarding<'a, Ready> {
    type Context<'ctx> = GuildId;
    type Built = Onboarding;

    /// Sets [`Onboarding`] in the guild.
    ///
    /// **Note**: Requires the [Manage Roles] and [Manage Guild] permissions.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns a [`ModelError::InvalidPermissions`] if the current user
    /// lacks permission. Otherwise returns [`Error::Http`], as well as if invalid data is given.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    async fn execute(
        mut self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        #[cfg(feature = "cache")]
        crate::utils::user_has_guild_perms(
            &cache_http,
            ctx,
            Permissions::MANAGE_GUILD | Permissions::MANAGE_ROLES,
        )?;

        cache_http.http().set_guild_onboarding(ctx, &self, self.audit_log_reason).await
    }
}
