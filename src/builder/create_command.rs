#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for creating a new [`CommandOption`].
///
/// [`Self::kind`], [`Self::name`], and [`Self::description`] are required fields.
///
/// [`CommandOption`]: crate::model::application::CommandOption
///
/// [Discord docs](https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateCommandOption(CommandOption);

impl CreateCommandOption {
    /// Creates a new builder with the given option type, name, and description, leaving all other
    /// fields empty.
    pub fn new(
        kind: CommandOptionType,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self(CommandOption {
            kind,
            name: name.into().into(),
            name_localizations: None,
            description: description.into().into(),
            description_localizations: None,
            __generated_flags: CommandOptionGeneratedFlags::empty(),
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,

            channel_types: FixedArray::default(),
            choices: Vec::new(),
            options: Vec::new(),
        })
    }

    /// Sets the `CommandOptionType`, replacing the current value as set in [`Self::new`].
    pub fn kind(mut self, kind: CommandOptionType) -> Self {
        self.0.kind = kind;
        self
    }

    /// Sets the name of the option, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = name.into().into();
        self
    }

    /// Specifies a localized name of the option.
    ///
    /// ```rust
    /// # use serenity::builder::CreateCommandOption;
    /// # use serenity::model::application::CommandOptionType;
    /// # CreateCommandOption::new(CommandOptionType::Integer, "", "")
    /// .name("age")
    /// .name_localized("zh-CN", "岁数")
    /// # ;
    /// ```
    pub fn name_localized(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        let map = self.0.name_localizations.get_or_insert_with(Default::default);
        map.insert(locale.into(), name.into());
        self
    }

    /// Sets the description for the option, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.0.description = description.into().into();
        self
    }
    /// Specifies a localized description of the option.
    ///
    /// ```rust
    /// # use serenity::builder::CreateCommandOption;
    /// # use serenity::model::application::CommandOptionType;
    /// # CreateCommandOption::new(CommandOptionType::String, "", "")
    /// .description("Wish a friend a happy birthday")
    /// .description_localized("zh-CN", "祝你朋友生日快乐")
    /// # ;
    /// ```
    pub fn description_localized(
        mut self,
        locale: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let map = self.0.description_localizations.get_or_insert_with(Default::default);
        map.insert(locale.into(), description.into());
        self
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`.
    pub fn required(mut self, required: bool) -> Self {
        self.0.set_required(required);
        self
    }

    /// Adds an optional int-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice(self, name: impl Into<String>, value: i64) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::from(value),
            name_localizations: None,
        })
    }

    /// Adds a localized optional int-choice. See [`Self::add_int_choice`] for more info.
    pub fn add_int_choice_localized(
        self,
        name: impl Into<String>,
        value: i64,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::from(value),
            name_localizations: Some(
                locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
            ),
        })
    }

    /// Adds an optional string-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be up to 100 characters.
    pub fn add_string_choice(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::String(value.into()),
            name_localizations: None,
        })
    }

    /// Adds a localized optional string-choice. See [`Self::add_string_choice`] for more info.
    pub fn add_string_choice_localized(
        self,
        name: impl Into<String>,
        value: impl Into<String>,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::String(value.into()),
            name_localizations: Some(
                locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
            ),
        })
    }

    /// Adds an optional number-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice(self, name: impl Into<String>, value: f64) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::from(value),
            name_localizations: None,
        })
    }

    /// Adds a localized optional number-choice. See [`Self::add_number_choice`] for more info.
    pub fn add_number_choice_localized(
        self,
        name: impl Into<String>,
        value: f64,
        locales: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.add_choice(CommandOptionChoice {
            name: name.into().into(),
            value: Value::from(value),
            name_localizations: Some(
                locales.into_iter().map(|(l, n)| (l.into(), n.into())).collect(),
            ),
        })
    }

    fn add_choice(mut self, value: CommandOptionChoice) -> Self {
        self.0.choices.push(value);
        self
    }

    /// Optionally enable/disable autocomplete interactions for this option.
    ///
    /// **Notes**:
    /// - May not be set to `true` if `choices` are set
    /// - Options using `autocomplete` are not confined to only use given choices
    pub fn set_autocomplete(mut self, value: bool) -> Self {
        self.0.set_autocomplete(value);
        self
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// This will overwrite any existing sub-options. To add a sub-option to the existing list, use
    /// [`Self::add_sub_option`].
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group
    /// can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::CommandOptionType::SubCommand
    pub fn set_sub_options(
        mut self,
        sub_options: impl IntoIterator<Item = CreateCommandOption>,
    ) -> Self {
        self.0.options = sub_options.into_iter().map(|o| o.0).collect();
        self
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group
    /// can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::CommandOptionType::SubCommand
    pub fn add_sub_option(mut self, sub_option: CreateCommandOption) -> Self {
        self.0.options.push(sub_option.0);
        self
    }

    /// If the option is a [`Channel`], it will only be able to show these types.
    ///
    /// [`Channel`]: crate::model::application::CommandOptionType::Channel
    pub fn channel_types(mut self, channel_types: Vec<ChannelType>) -> Self {
        self.0.channel_types = channel_types.into();
        self
    }

    /// Sets the minimum permitted value for this integer option
    pub fn min_int_value(mut self, value: i64) -> Self {
        self.0.min_value = Some(value.into());
        self
    }

    /// Sets the maximum permitted value for this integer option
    pub fn max_int_value(mut self, value: i64) -> Self {
        self.0.max_value = Some(value.into());
        self
    }

    /// Sets the minimum permitted value for this number option
    pub fn min_number_value(mut self, value: f64) -> Self {
        self.0.min_value = serde_json::Number::from_f64(value);
        self
    }

    /// Sets the maximum permitted value for this number option
    pub fn max_number_value(mut self, value: f64) -> Self {
        self.0.max_value = serde_json::Number::from_f64(value);
        self
    }

    /// Sets the minimum permitted length for this string option.
    ///
    /// The value of `min_length` must be greater or equal to `0`.
    pub fn min_length(mut self, value: u16) -> Self {
        self.0.min_length = Some(value);

        self
    }

    /// Sets the maximum permitted length for this string option.
    ///
    /// The value of `max_length` must be greater or equal to `1`.
    pub fn max_length(mut self, value: u16) -> Self {
        self.0.max_length = Some(value);

        self
    }
}

/// A builder for creating a new [`Command`].
///
/// [`Self::name`] and [`Self::description`] are required fields.
///
/// [`Command`]: crate::model::application::Command
///
/// Discord docs:
/// - [global command](https://discord.com/developers/docs/interactions/application-commands#create-global-application-command-json-params)
/// - [guild command](https://discord.com/developers/docs/interactions/application-commands#create-guild-application-command-json-params)
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateCommand {
    name: FixedString,
    name_localizations: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    description_localizations: HashMap<String, String>,
    options: Vec<CreateCommandOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_member_permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dm_permission: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<CommandType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    integration_types: Option<Vec<InstallationContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contexts: Option<Vec<InteractionContext>>,
    nsfw: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    handler: Option<EntryPointHandlerType>,
}

impl CreateCommand {
    /// Creates a new builder with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            kind: None,

            name: name.into().into(),
            name_localizations: HashMap::new(),
            description: None,
            description_localizations: HashMap::new(),
            default_member_permissions: None,
            dm_permission: None,

            integration_types: None,
            contexts: None,

            options: Vec::new(),
            nsfw: false,
            handler: None,
        }
    }

    /// Specifies the name of the application command, replacing the current value as set in
    /// [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two
    /// global commands of the same app cannot have the same name. Two guild-specific commands of
    /// the same app cannot have the same name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into().into();
        self
    }

    /// Specifies a localized name of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateCommand::new("")
    /// .name("birthday")
    /// .name_localized("zh-CN", "生日")
    /// .name_localized("el", "γενέθλια")
    /// # ;
    /// ```
    pub fn name_localized(mut self, locale: impl Into<String>, name: impl Into<String>) -> Self {
        self.name_localizations.insert(locale.into(), name.into());
        self
    }

    /// Specifies the type of the application command.
    pub fn kind(mut self, kind: CommandType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Specifies the default permissions required to execute the command.
    pub fn default_member_permissions(mut self, permissions: Permissions) -> Self {
        self.default_member_permissions = Some(permissions.bits().to_string());
        self
    }

    /// Specifies if the command is available in DMs.
    #[cfg_attr(feature = "unstable_discord_api", deprecated = "Use contexts instead")]
    pub fn dm_permission(mut self, enabled: bool) -> Self {
        self.dm_permission = Some(enabled);
        self
    }

    /// Specifies the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Specifies a localized description of the application command.
    ///
    /// ```rust
    /// # serenity::builder::CreateCommand::new("")
    /// .description("Wish a friend a happy birthday")
    /// .description_localized("zh-CN", "祝你朋友生日快乐")
    /// # ;
    /// ```
    pub fn description_localized(
        mut self,
        locale: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.description_localizations.insert(locale.into(), description.into());
        self
    }

    /// Adds an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn add_option(mut self, option: CreateCommandOption) -> Self {
        self.options.push(option);
        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn set_options(mut self, options: Vec<CreateCommandOption>) -> Self {
        self.options = options;
        self
    }

    /// Adds an installation context that this application command can be used in.
    pub fn add_integration_type(mut self, integration_type: InstallationContext) -> Self {
        self.integration_types.get_or_insert_with(Vec::default).push(integration_type);
        self
    }

    /// Sets the installation contexts that this application command can be used in.
    pub fn integration_types(mut self, integration_types: Vec<InstallationContext>) -> Self {
        self.integration_types = Some(integration_types);
        self
    }

    /// Adds an interaction context that this application command can be used in.
    pub fn add_context(mut self, context: InteractionContext) -> Self {
        self.contexts.get_or_insert_with(Vec::default).push(context);
        self
    }

    /// Sets the interaction contexts that this application command can be used in.
    pub fn contexts(mut self, contexts: Vec<InteractionContext>) -> Self {
        self.contexts = Some(contexts);
        self
    }

    /// Whether this command is marked NSFW (age-restricted)
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = nsfw;
        self
    }

    /// Sets the command's entry point handler type. Only valid for commands of type
    /// [`PrimaryEntryPoint`].
    ///
    /// [`PrimaryEntryPoint`]: CommandType::PrimaryEntryPoint
    pub fn handler(mut self, handler: EntryPointHandlerType) -> Self {
        self.handler = Some(handler);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for CreateCommand {
    type Context<'ctx> = (Option<GuildId>, Option<CommandId>);
    type Built = Command;

    /// Create a [`Command`], overriding an existing one with the same name if it exists.
    ///
    /// Providing a [`GuildId`] will create a command in the corresponding [`Guild`]. Otherwise, a
    /// global command will be created.
    ///
    /// Providing a [`CommandId`] will edit the corresponding command.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given. See [Discord's docs] for more details.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Discord's docs]: https://discord.com/developers/docs/interactions/slash-commands
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        let http = cache_http.http();
        match ctx {
            (Some(guild_id), Some(cmd_id)) => {
                http.edit_guild_command(guild_id, cmd_id, &self).await
            },
            (Some(guild_id), None) => http.create_guild_command(guild_id, &self).await,
            (None, Some(cmd_id)) => http.edit_global_command(cmd_id, &self).await,
            (None, None) => http.create_global_command(&self).await,
        }
    }
}
