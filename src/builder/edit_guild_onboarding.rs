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
    /// Creates an empty builder, equivalent to [`Self::default`].
    ///
    /// **Note**: Any changes to prompts or default channels will overwrite their existing
    /// configurations. To modify existing prompts or default channels, retrieve the current
    /// configuration with [`GuildId::get_onboarding`] and use [`Self::from`] instead, e.g.:
    ///
    /// ```rust
    /// # use serenity::model::id::{GuildId, PromptId};
    /// # use serenity::http::Http;
    /// # use serenity::builder::{EditGuildOnboarding, EditOnboardingPrompt};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let guild_id = GuildId::new(1);
    /// # let http: Http = unimplemented!();
    /// # let prompt_id = PromptId::default();
    /// let onboarding = guild_id.get_onboarding(&http).await?;
    /// let builder = EditGuildOnboarding::from(onboarding)
    ///     .edit_prompt(EditOnboardingPrompt::new(prompt_id).required(true));
    /// let edited_onboarding = guild_id.edit_onboarding(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// When only modifying `enabled` and/or `mode`, [`Self::new`] will not overwrite existing
    /// prompts or default channels, i.e., the following two builders produce the same result:
    ///
    /// ```rust
    /// # use serenity::model::prelude::{GuildId, OnboardingMode, PromptId};
    /// # use serenity::http::Http;
    /// # use serenity::builder::{EditGuildOnboarding, EditOnboardingPrompt};
    /// #
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let guild_id = GuildId::new(1);
    /// # let http: Http = unimplemented!();
    /// # let prompt_id = PromptId::default();
    /// # let onboarding = guild_id.get_onboarding(&http).await?;
    /// # let builder =
    /// EditGuildOnboarding::from(onboarding).enabled(true).mode(OnboardingMode::Default)
    /// # ;
    /// # let builder =
    /// EditGuildOnboarding::new().enabled(true).mode(OnboardingMode::Default)
    /// # ;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prompts shown during onboarding and in customize community.
    ///
    /// This will replace any existing prompts. To append a prompt, use [`Self::add_prompt`].
    ///
    /// **Note**: Maximum of 15 prompts.
    pub fn prompts(mut self, prompts: impl Into<Cow<'a, [CreateOnboardingPrompt<'a>]>>) -> Self {
        self.prompts = prompts.into();
        self
    }

    /// Adds a prompt shown during onboarding and in customize community.
    ///
    /// **Note**: Maximum of 15 prompts.
    pub fn add_prompt(mut self, prompt: CreateOnboardingPrompt<'a>) -> Self {
        self.prompts.to_mut().push(prompt);
        self
    }

    /// Removes a prompt shown during onboarding and in customize community.
    pub fn remove_prompt(mut self, prompt_id: PromptId) -> Self {
        self.prompts.to_mut().retain(|prompt| prompt.id != prompt_id);
        self
    }

    /// Edits a prompt shown during onboarding and in customize community.
    pub fn edit_prompt(mut self, prompt: EditOnboardingPrompt<'a>) -> Self {
        for existing in self.prompts.to_mut() {
            if existing.id == prompt.id {
                if prompt.kind.is_some() {
                    existing.kind = prompt.kind;
                }
                if !prompt.removed_options.is_empty() {
                    existing.options.to_mut().retain(|option| {
                        option.id.is_some_and(|id| !prompt.removed_options.contains(&id))
                    });
                }
                for edited_option in &prompt.edited_options {
                    let mut not_found = true;
                    for existing_option in existing.options.to_mut() {
                        if existing_option.id.is_some_and(|id| id == edited_option.id) {
                            if edited_option.replace_channels {
                                existing_option.channel_ids =
                                    edited_option.added_channel_ids.to_vec().into();
                            } else {
                                existing_option
                                    .channel_ids
                                    .to_mut()
                                    .retain(|id| !edited_option.removed_channel_ids.contains(id));
                                existing_option
                                    .channel_ids
                                    .to_mut()
                                    .extend(edited_option.added_channel_ids.to_vec());
                            }
                            if edited_option.replace_roles {
                                existing_option.role_ids =
                                    edited_option.added_role_ids.to_vec().into();
                            } else {
                                existing_option
                                    .role_ids
                                    .to_mut()
                                    .retain(|id| !edited_option.removed_role_ids.contains(id));
                                existing_option
                                    .role_ids
                                    .to_mut()
                                    .extend(edited_option.added_role_ids.to_vec());
                            }
                            if edited_option.emoji_id.is_some() {
                                existing_option.emoji_id = edited_option.emoji_id;
                            }
                            if let Some(emoji_name) = &edited_option.emoji_name {
                                existing_option.emoji_name = Some(emoji_name.to_string().into());
                            }
                            if edited_option.emoji_animated.is_some() {
                                existing_option.emoji_animated = edited_option.emoji_animated;
                            }
                            if let Some(title) = &edited_option.title {
                                existing_option.title = title.to_string().into();
                            }
                            if let Some(description) = &edited_option.description {
                                existing_option.description = Some(description.to_string().into());
                            }
                            not_found = false;
                            break;
                        }
                    }
                    if not_found {
                        tracing::warn!(
                            "specified PromptOptionId not found; discarding changes: {edited_option:?}"
                        );
                    }
                }
                existing.options.to_mut().extend(prompt.added_options.to_vec());
                if let Some(title) = prompt.title {
                    existing.title = title;
                }
                if prompt.single_select.is_some() {
                    existing.single_select = prompt.single_select;
                }
                if prompt.required.is_some() {
                    existing.required = prompt.required;
                }
                if prompt.in_onboarding.is_some() {
                    existing.in_onboarding = prompt.in_onboarding;
                }
                return self;
            }
        }
        tracing::warn!("specified PromptId not found; discarding changes: {prompt:?}");
        self
    }

    /// Sets the channels that members get opted into automatically.
    ///
    /// This will replace any existing default channels. To append a channel, use
    /// [`Self::add_default_channel`].
    ///
    /// **Note**: Maximum of 500 channels.
    pub fn default_channels(mut self, channel_ids: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.default_channel_ids = channel_ids.into();
        self
    }

    /// Adds a channel that members get opted into automatically.
    ///
    /// **Note**: Maximum of 500 channels.
    pub fn add_default_channel(mut self, channel_id: ChannelId) -> Self {
        self.default_channel_ids.to_mut().push(channel_id);
        self
    }

    /// Removes a channel that members get opted into automatically.
    pub fn remove_default_channel(mut self, channel_id: ChannelId) -> Self {
        self.default_channel_ids.to_mut().retain(|id| &channel_id != id);
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
    /// [`OnboardingMode::Advanced`], channels assigned through required pre-join prompts also
    /// count toward these constraints.
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

impl From<Onboarding> for EditGuildOnboarding<'_> {
    /// Creates a builder pre-filled with a guild's existing onboarding configuration.
    ///
    /// This will preserve any part of the existing configuration that is not modified via the
    /// builder methods.
    fn from(onboarding: Onboarding) -> Self {
        Self {
            prompts: onboarding.prompts.into_iter().map(Into::into).collect(),
            default_channel_ids: onboarding.default_channel_ids.into(),
            enabled: Some(onboarding.enabled),
            mode: Some(onboarding.mode),
            audit_log_reason: None,
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
    /// Creates a builder with the title and options, leaving all other fields empty.
    pub fn new(
        title: impl Into<Cow<'a, str>>,
        options: impl Into<Cow<'a, [CreateOnboardingPromptOption<'a>]>>,
    ) -> Self {
        Self {
            // This endpoint requires an Id even when creating a new prompt, but the specified Id
            // is not preserved so we just assign the default here.
            // See: https://github.com/discord/discord-api-docs/issues/6320
            id: PromptId::default(),
            kind: None,
            options: options.into(),
            title: title.into(),
            single_select: None,
            required: None,
            in_onboarding: None,
        }
    }

    /// Sets the type of the prompt.
    ///
    /// Defaults to [`OnboardingPromptType::MultipleChoice`].
    pub fn kind(mut self, kind: OnboardingPromptType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets the options available within the prompt.
    ///
    /// This will replace all options. To append an option, use [`Self::add_option`].
    ///
    /// **Note**: At least one option is required, with a maximum of 50.
    pub fn options(
        mut self,
        options: impl Into<Cow<'a, [CreateOnboardingPromptOption<'a>]>>,
    ) -> Self {
        self.options = options.into();
        self
    }

    /// Adds an option available within the prompt.
    ///
    /// **Note**: Maximum of 50 options.
    pub fn add_option(mut self, option: CreateOnboardingPromptOption<'a>) -> Self {
        self.options.to_mut().push(option);
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

impl From<OnboardingPrompt> for CreateOnboardingPrompt<'_> {
    fn from(prompt: OnboardingPrompt) -> Self {
        Self {
            id: prompt.id,
            kind: Some(prompt.kind),
            options: prompt.options.into_iter().map(Into::into).collect(),
            title: prompt.title.into(),
            single_select: Some(prompt.single_select),
            required: Some(prompt.required),
            in_onboarding: Some(prompt.in_onboarding),
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
    /// Creates a builder with the title, leaving all other fields empty.
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

    /// Sets the channels a member is added to when the option is selected.
    ///
    /// This will replace all channels. To append a channel, use [`Self::add_channel`].
    ///
    /// **Note**: At least one channel ___or___ role is required, with a maximum of 50 each.
    pub fn channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.channel_ids = channels.into();
        self
    }

    /// Adds a channel a member is added to when the option is selected.
    ///
    /// **Note**: Maximum of 50 channels.
    pub fn add_channel(mut self, channel_id: ChannelId) -> Self {
        self.channel_ids.to_mut().push(channel_id);
        self
    }

    /// Removes a channel a member is added to when the option is selected.
    ///
    /// **Note**: At least one channel ___or___ role is required.
    pub fn remove_channel(mut self, channel_id: ChannelId) -> Self {
        self.channel_ids.to_mut().retain(|id| &channel_id != id);
        self
    }

    /// Sets the roles assigned to a member when the option is selected.
    ///
    /// This will replace all roles. To append a role, use [`Self::add_role`].
    ///
    /// **Note**: At least one role ___or___ channel is required, with a maximum of 50 each.
    pub fn roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.role_ids = roles.into();
        self
    }

    /// Adds a role assigned to a member when the option is selected.
    ///
    /// **Note**: Maximum of 50 roles.
    pub fn add_role(mut self, role_id: RoleId) -> Self {
        self.role_ids.to_mut().push(role_id);
        self
    }

    /// Removes a role assigned to a member when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required.
    pub fn remove_role(mut self, role_id: RoleId) -> Self {
        self.role_ids.to_mut().retain(|id| &role_id != id);
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

impl From<OnboardingPromptOption> for CreateOnboardingPromptOption<'_> {
    fn from(option: OnboardingPromptOption) -> Self {
        let (emoji_id, emoji_name, emoji_animated) = option.emoji.map_or_else(
            || (option.emoji_id, option.emoji_name.map(Into::into), option.emoji_animated),
            |emoji| match emoji {
                ReactionType::Custom {
                    animated,
                    id,
                    name,
                } => (Some(id), name.map(Into::into), Some(animated)),
                ReactionType::Unicode(name) => (None, Some(name.into()), None),
            },
        );
        Self {
            id: Some(option.id),
            channel_ids: option.channel_ids.into(),
            role_ids: option.role_ids.into(),
            emoji: None,
            emoji_id,
            emoji_name,
            emoji_animated,
            title: option.title.into(),
            description: option.description.map(Into::into),
        }
    }
}

/// A builder for editing an [`OnboardingPrompt`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-onboarding-prompt-structure)
// Pseudo-builder used to populate CreateOnboardingPrompt, so no need to derive Serialize.
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct EditOnboardingPrompt<'a> {
    id: PromptId,
    kind: Option<OnboardingPromptType>,
    added_options: Cow<'a, [CreateOnboardingPromptOption<'a>]>,
    removed_options: Cow<'a, [PromptOptionId]>,
    edited_options: Vec<EditOnboardingPromptOption<'a>>,
    title: Option<Cow<'a, str>>,
    single_select: Option<bool>,
    required: Option<bool>,
    in_onboarding: Option<bool>,
}

impl<'a> EditOnboardingPrompt<'a> {
    /// Creates a builder with the Id, leaving all other fields empty.
    ///
    /// This builder modifies an existing [`OnboardingPrompt`]. Anything not explicitly modified
    /// by the builder will be left unchanged.
    ///
    /// **Note**: If a prompt with the specified Id does not exist, no changes will be made.
    pub fn new(id: PromptId) -> Self {
        Self {
            id,
            kind: None,
            added_options: Cow::default(),
            removed_options: Cow::default(),
            edited_options: Vec::new(),
            title: None,
            single_select: None,
            required: None,
            in_onboarding: None,
        }
    }

    /// Sets the type of the prompt.
    ///
    /// Defaults to [`OnboardingPromptType::MultipleChoice`].
    pub fn kind(mut self, kind: OnboardingPromptType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Adds one or more options available within the prompt.
    ///
    /// **Note**: At least one option is required, with a maximum of 50.
    pub fn add_options(
        mut self,
        options: impl Into<Cow<'a, [CreateOnboardingPromptOption<'a>]>>,
    ) -> Self {
        self.added_options = options.into();
        self
    }

    /// Removes one or more options available within the prompt.
    ///
    /// **Note**: At least one option is required.
    pub fn remove_options(mut self, options: impl Into<Cow<'a, [PromptOptionId]>>) -> Self {
        self.removed_options = options.into();
        self
    }

    /// Edits an option available within the prompt.
    pub fn edit_option(mut self, option: EditOnboardingPromptOption<'a>) -> Self {
        self.edited_options.push(option);
        self
    }

    /// Sets the title of the prompt.
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Whether users are limited to selecting one option for the prompt.
    pub fn single_select(mut self, single_select: bool) -> Self {
        self.single_select = Some(single_select);
        self
    }

    /// Whether the prompt is required before a user completes the onboarding flow.
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    /// Whether the prompt is present in the onboarding flow.
    ///
    /// If set to `false`, the prompt will only appear in the "Channels & Roles" tab.
    pub fn in_onboarding(mut self, in_onboarding: bool) -> Self {
        self.in_onboarding = Some(in_onboarding);
        self
    }

    pub fn into_owned(self) -> EditOnboardingPrompt<'static> {
        EditOnboardingPrompt {
            id: self.id,
            kind: self.kind,
            added_options: self
                .added_options
                .into_owned()
                .into_iter()
                .map(CreateOnboardingPromptOption::into_owned)
                .collect(),
            removed_options: self.removed_options.into_owned().into(),
            edited_options: self
                .edited_options
                .into_iter()
                .map(EditOnboardingPromptOption::into_owned)
                .collect(),
            title: self.title.map(|title| title.into_owned().into()),
            single_select: self.single_select,
            required: self.required,
            in_onboarding: self.in_onboarding,
        }
    }
}

/// A builder for editing an [`OnboardingPromptOption`].
///
/// [Discord docs](https://docs.discord.com/developers/resources/guild#guild-onboarding-object-prompt-option-structure)
// Pseudo-builder used to populate CreateOnboardingPromptOption, so no need to derive Serialize.
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct EditOnboardingPromptOption<'a> {
    id: PromptOptionId,
    added_channel_ids: Cow<'a, [ChannelId]>,
    removed_channel_ids: Cow<'a, [ChannelId]>,
    replace_channels: bool,
    added_role_ids: Cow<'a, [RoleId]>,
    removed_role_ids: Cow<'a, [RoleId]>,
    replace_roles: bool,
    emoji_id: Option<EmojiId>,
    emoji_name: Option<Cow<'a, str>>,
    emoji_animated: Option<bool>,
    title: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
}

impl<'a> EditOnboardingPromptOption<'a> {
    /// Creates a builder with the Id, leaving all other fields empty.
    ///
    /// This builder modifies an existing [`OnboardingPromptOption`]. Anything not explicitly
    /// modified by the builder will be left unchanged.
    ///
    /// **Note**: If an option with the specified Id does not exist, no changes will be made.
    pub fn new(id: PromptOptionId) -> Self {
        Self {
            id,
            added_channel_ids: Cow::default(),
            removed_channel_ids: Cow::default(),
            replace_channels: false,
            added_role_ids: Cow::default(),
            removed_role_ids: Cow::default(),
            replace_roles: false,
            emoji_id: None,
            emoji_name: None,
            emoji_animated: None,
            title: None,
            description: None,
        }
    }

    /// Adds one or more channels a member is added to when the option is selected.
    ///
    /// **Note**: At least one channel ___or___ role is required, with a maximum of 50 each.
    pub fn add_channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.added_channel_ids = channels.into();
        self
    }

    /// Removes one or more channels a member is added to when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required.
    pub fn remove_channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.removed_channel_ids = channels.into();
        self
    }

    /// Sets the channels a member is added to when the option is selected, replacing all
    /// previously set channels.
    ///
    /// **Note**: At least one channel ___or___ role is required, with a maximum of 50 each.
    pub fn replace_channels(mut self, channels: impl Into<Cow<'a, [ChannelId]>>) -> Self {
        self.added_channel_ids = channels.into();
        self.replace_channels = true;
        self
    }

    /// Adds one or more roles assigned to a member when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required, with a maximum of 50 each.
    pub fn add_roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.added_role_ids = roles.into();
        self
    }

    /// Removes one or more roles assigned to a member when the option is selected.
    ///
    /// **Note**: At least one role ___or___ channel is required.
    pub fn remove_roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.removed_role_ids = roles.into();
        self
    }

    /// Sets the roles assigned to a member when the option is selected, replacing all previously
    /// set roles.
    ///
    /// **Note**: At least one role ___or___ channel is required, with a maximum of 50 each.
    pub fn replace_roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.added_role_ids = roles.into();
        self.replace_roles = true;
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

    /// Sets the title of the option.
    ///
    /// **Note**: Must be between 1 and 50 characters.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the description of the option.
    ///
    /// **Note**: Maximum of 100 characters.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn into_owned(self) -> EditOnboardingPromptOption<'static> {
        EditOnboardingPromptOption {
            id: self.id,
            added_channel_ids: self.added_channel_ids.into_owned().into(),
            removed_channel_ids: self.removed_channel_ids.into_owned().into(),
            replace_channels: self.replace_channels,
            added_role_ids: self.added_role_ids.into_owned().into(),
            removed_role_ids: self.removed_role_ids.into_owned().into(),
            replace_roles: self.replace_roles,
            emoji_id: self.emoji_id,
            emoji_name: self.emoji_name.map(|name| name.into_owned().into()),
            emoji_animated: self.emoji_animated,
            title: self.title.map(|title| title.into_owned().into()),
            description: self.description.map(|desc| desc.into_owned().into()),
        }
    }
}
