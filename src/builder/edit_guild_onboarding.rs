use std::borrow::Cow;

#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// A builder to edit the onboarding configuration of a guild.
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#modify-guild-onboarding)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildOnboarding<'a> {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    prompts: Cow<'a, [CreateOnboardingPrompt<'a>]>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    default_channel_ids: Cow<'a, [ChannelId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<OnboardingMode>,

    #[serde(skip)]
    audit_log_reason: Option<Cow<'a, str>>,
}

impl<'a> EditGuildOnboarding<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prompts shown during onboarding and in customize community.
    ///
    /// All prompts must be provided. It is not currently possible to add, remove, or edit
    /// individual prompts.
    ///
    /// **Note**: Maximum of 15 prompts.
    pub fn prompts(mut self, prompts: impl Into<Cow<'a, [CreateOnboardingPrompt<'a>]>>) -> Self {
        self.prompts = prompts.into();
        self
    }

    /// Sets the channels that members get opted into automatically.
    ///
    /// **Note**: Maximum of 500 channels.
    pub fn default_channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.default_channel_ids = channels.into();
        self
    }

    /// Whether onboarding is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the mode of onboarding.
    ///
    /// When onboarding is enabled, there must be at least 7 default channels and at least 5 of
    /// them must allow sending messages to the `@everyone` role. When the mode is switched to
    /// [`OnboardingMode::OnboardingAdvanced`], prompts also count toward these constraints.
    pub fn mode(mut self, mode: OnboardingMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets the request's audit log reason.
    pub fn audit_log_reason(mut self, reason: impl Into<Cow<'a, str>>) -> Self {
        self.audit_log_reason = Some(reason.into());
        self
    }

    /// Edits the guild's onboarding configuration.
    ///
    /// **Note**: Requires the [Manage Guild] and [Manage Roles] permissions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Manage Guild]: Permissions::MANAGE_GUILD
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<Onboarding> {
        http.edit_guild_onboarding(guild_id, &self, self.audit_log_reason.as_deref()).await
    }

    pub fn into_owned(self) -> EditGuildOnboarding<'static> {
        EditGuildOnboarding {
            prompts: self
                .prompts
                .into_owned()
                .into_iter()
                .map(CreateOnboardingPrompt::into_owned)
                .collect(),
            default_channel_ids: self.default_channel_ids.into_owned().into(),
            enabled: self.enabled,
            mode: self.mode,
            audit_log_reason: self.audit_log_reason.map(|reason| reason.into_owned().into()),
        }
    }
}

/// A builder for creating an [`OnboardingPrompt`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-onboarding-prompt-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateOnboardingPrompt<'a> {
    id: PromptId,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<OnboardingPromptType>,
    options: Cow<'a, [CreateOnboardingPromptOption<'a>]>,
    title: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    single_select: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_onboarding: Option<bool>,
}

impl<'a> CreateOnboardingPrompt<'a> {
    /// Creates a builder with the title and options, and optionally an Id, leaving all other
    /// fields empty.
    ///
    /// When `id` is set to `None`, a new Id will be generated and an [`OnboardingPrompt(Create)`]
    /// action will be added to the audit log.
    ///
    /// If an existing [`PromptId`] is specified, the Id will be preserved and an
    /// [`OnboardingPrompt(Update)`] action will be added to the audit log. However, any omitted
    /// fields will be reset to their defaults, so it is not possible to make partial edits. The
    /// only difference is the type of audit log action recorded.
    ///
    /// [`OnboardingPrompt(Create)`]: OnboardingPromptAction::Create
    /// [`OnboardingPrompt(Update)`]: OnboardingPromptAction::Update
    pub fn new(
        id: Option<PromptId>,
        title: impl Into<Cow<'a, str>>,
        options: impl Into<Cow<'a, [CreateOnboardingPromptOption<'a>]>>,
    ) -> Self {
        Self {
            id: id.unwrap_or_default(),
            kind: None,
            options: options.into(),
            title: title.into(),
            single_select: None,
            required: None,
            in_onboarding: None,
        }
    }

    /// Sets the Id of the prompt. See [`Self::new`] for details.
    pub fn id(mut self, id: PromptId) -> Self {
        self.id = id;
        self
    }

    /// Sets the type of the prompt.
    ///
    /// Defaults to [`OnboardingPromptType::MultipleChoice`].
    pub fn kind(mut self, kind: OnboardingPromptType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the options available within the prompt, replacing those set in [`Self::new`].
    ///
    /// **Note**: At least one option is required, with a maximum of 50.
    pub fn options(
        mut self,
        options: impl Into<Cow<'a, [CreateOnboardingPromptOption<'a>]>>,
    ) -> Self {
        self.options = options.into();
        self
    }

    /// Sets the title of the prompt, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Whether users are limited to selecting one option for the prompt.
    ///
    /// Defaults to `false`.
    pub fn single_select(mut self, single_select: bool) -> Self {
        self.single_select = Some(single_select);
        self
    }

    /// Whether the prompt is required before a user completes the onboarding flow.
    ///
    /// Defaults to `false`.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Whether the prompt is present in the onboarding flow.
    ///
    /// Defaults to `true`. If set to `false`, the prompt will only appear in the "Channels &
    /// Roles" tab.
    pub fn in_onboarding(mut self, in_onboarding: bool) -> Self {
        self.in_onboarding = Some(in_onboarding);
        self
    }

    pub fn into_owned(self) -> CreateOnboardingPrompt<'static> {
        CreateOnboardingPrompt {
            id: self.id,
            kind: self.kind,
            options: self
                .options
                .into_owned()
                .into_iter()
                .map(CreateOnboardingPromptOption::into_owned)
                .collect(),
            title: self.title.into_owned().into(),
            single_select: self.single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,
        }
    }
}

/// A builder for creating an [`OnboardingPromptOption`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-prompt-option-structure)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateOnboardingPromptOption<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<PromptOptionId>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    channel_ids: Cow<'a, [ChannelId]>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    role_ids: Cow<'a, [RoleId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_id: Option<EmojiId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_name: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji_animated: Option<bool>,
    title: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
}

impl<'a> CreateOnboardingPromptOption<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: None,
            channel_ids: Cow::default(),
            role_ids: Cow::default(),
            emoji: None,
            emoji_id: None,
            emoji_name: None,
            emoji_animated: None,
            title: title.into(),
            description: None,
        }
    }

    /// Sets the Id of the prompt option.
    ///
    /// If an existing [`PromptOptionId`] is specified, when updating the parent prompt with its
    /// [`PromptId`] also specified, the Id will be preserved and the changes will be recorded as
    /// updates in the audit log. However, any omitted fields will be reset to their defaults, so
    /// it is not possible to make partial edits. The only difference is the type of audit log
    /// action recorded.
    pub fn id(mut self, id: PromptOptionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the channels a member is added to when the option is selected.
    ///
    /// **Note**: At least one channel ___or___ role is required, with a maximum of 50 each.
    pub fn channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.channel_ids = channels.into();
        self
    }

    /// Sets the roles assigned to a member when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required, with a maximum of 50 each.
    pub fn roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.role_ids = roles.into();
        self
    }

    /// Sets the emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        let emoji = emoji.into();
        match emoji {
            ReactionType::Custom {
                animated,
                id,
                name,
            } => {
                self.emoji_id = Some(id);
                self.emoji_name = name.map(Into::into);
                self.emoji_animated = Some(animated);
            },
            ReactionType::Unicode(name) => self.emoji_name = Some(name.into()),
        }
        self
    }

    /// Sets the title of the option, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 50 characters.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the description of the option.
    ///
    /// **Note**: Maximum of 100 characters.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn into_owned(self) -> CreateOnboardingPromptOption<'static> {
        CreateOnboardingPromptOption {
            id: self.id,
            channel_ids: self.channel_ids.into_owned().into(),
            role_ids: self.role_ids.into_owned().into(),
            emoji: None,
            emoji_id: self.emoji_id,
            emoji_name: self.emoji_name.map(|name| name.into_owned().into()),
            emoji_animated: self.emoji_animated,
            title: self.title.into_owned().into(),
            description: self.description.map(|desc| desc.into_owned().into()),
        }
    }
}
