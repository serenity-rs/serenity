#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsPromptType;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsPromptOptions;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsTitle;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsSingleSelect;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsRequired;
#[derive(serde::Serialize, Clone, Debug)]
pub struct NeedsInOnboarding;
#[derive(serde::Serialize, Clone, Debug)]
pub struct Ready;

use super::prompt_option_structure::{self, CreatePromptOption};

mod sealed {
    use super::*;
    pub trait Sealed {}

    impl Sealed for NeedsPromptType {}
    impl Sealed for NeedsPromptOptions {}
    impl Sealed for NeedsTitle {}
    impl Sealed for NeedsSingleSelect {}
    impl Sealed for NeedsRequired {}
    impl Sealed for NeedsInOnboarding {}
    impl Sealed for Ready {}
}

use sealed::*;

use crate::all::OnboardingPromptType;

#[derive(serde::Serialize, Clone, Debug)]
#[must_use = "Builders do nothing unless built"]
pub struct CreateOnboardingPrompt<Stage: Sealed> {
    prompt_type: OnboardingPromptType,
    options: Vec<CreatePromptOption<prompt_option_structure::Ready>>,
    title: String,
    single_select: bool,
    required: bool,
    in_onboarding: bool,

    #[serde(skip)]
    _stage: Stage,
}

impl Default for CreateOnboardingPrompt<NeedsPromptType> {
    /// See the documentation of [`Self::new`].
    fn default() -> Self {
        // Producing dummy values is okay as we must transition through all `Stage`s before firing,
        // which fills in the values with real values.
        Self {
            prompt_type: OnboardingPromptType::Dropdown,
            options: Vec::new(),
            title: String::new(),
            single_select: true,
            required: true,
            in_onboarding: true,

            _stage: NeedsPromptType,
        }
    }
}

impl CreateOnboardingPrompt<NeedsPromptType> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prompt_type(
        self,
        prompt_type: OnboardingPromptType,
    ) -> CreateOnboardingPrompt<NeedsPromptOptions> {
        CreateOnboardingPrompt {
            prompt_type,
            options: self.options,
            title: self.title,
            single_select: self.single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,

            _stage: NeedsPromptOptions,
        }
    }
}

impl CreateOnboardingPrompt<NeedsPromptOptions> {
    pub fn options(
        self,
        options: Vec<CreatePromptOption<prompt_option_structure::Ready>>,
    ) -> CreateOnboardingPrompt<NeedsTitle> {
        CreateOnboardingPrompt {
            prompt_type: self.prompt_type,
            options,
            title: self.title,
            single_select: self.single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,

            _stage: NeedsTitle,
        }
    }
}

impl CreateOnboardingPrompt<NeedsTitle> {
    pub fn title(self, title: impl Into<String>) -> CreateOnboardingPrompt<NeedsSingleSelect> {
        CreateOnboardingPrompt {
            prompt_type: self.prompt_type,
            options: self.options,
            title: title.into(),
            single_select: self.single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,

            _stage: NeedsSingleSelect,
        }
    }
}

impl CreateOnboardingPrompt<NeedsSingleSelect> {
    pub fn single_select(self, single_select: bool) -> CreateOnboardingPrompt<NeedsRequired> {
        CreateOnboardingPrompt {
            prompt_type: self.prompt_type,
            options: self.options,
            title: self.title,
            single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,

            _stage: NeedsRequired,
        }
    }
}

impl CreateOnboardingPrompt<NeedsRequired> {
    pub fn required(self, required: bool) -> CreateOnboardingPrompt<NeedsInOnboarding> {
        CreateOnboardingPrompt {
            prompt_type: self.prompt_type,
            options: self.options,
            title: self.title,
            single_select: self.single_select,
            required,
            in_onboarding: self.in_onboarding,

            _stage: NeedsInOnboarding,
        }
    }
}

impl CreateOnboardingPrompt<NeedsInOnboarding> {
    pub fn in_onboarding(self, in_onboarding: bool) -> CreateOnboardingPrompt<Ready> {
        CreateOnboardingPrompt {
            prompt_type: self.prompt_type,
            options: self.options,
            title: self.title,
            single_select: self.single_select,
            required: self.required,
            in_onboarding,

            _stage: Ready,
        }
    }
}
