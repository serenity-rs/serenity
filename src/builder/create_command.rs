use std::borrow::Cow;

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
pub struct CreateCommandOption<'a> {
    #[serde(rename = "type")]
    kind: CommandOptionType,
    name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name_localizations: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    description: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description_localizations: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    choices: Cow<'a, [CreateCommandOptionChoice<'a>]>,
    #[serde(default)]
    options: Cow<'a, [CreateCommandOption<'a>]>,
    #[serde(default)]
    channel_types: Cow<'a, [ChannelType]>,
    #[serde(default)]
    min_value: Option<serde_json::Number>,
    #[serde(default)]
    max_value: Option<serde_json::Number>,
    #[serde(default)]
    min_length: Option<u16>,
    #[serde(default)]
    max_length: Option<u16>,
    #[serde(default)]
    autocomplete: bool,
}

impl<'a> CreateCommandOption<'a> {
    /// Creates a new builder with the given option type, name, and description, leaving all other
    /// fields empty.
    pub fn new(
        kind: CommandOptionType,
        name: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            name_localizations: None,
            description: description.into(),
            description_localizations: None,
            required: false,
            autocomplete: false,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,

            channel_types: Cow::default(),
            choices: Cow::default(),
            options: Cow::default(),
        }
    }

    /// Sets the `CommandOptionType`, replacing the current value as set in [`Self::new`].
    pub fn kind(mut self, kind: CommandOptionType) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the name of the option, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
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
    pub fn name_localized(
        mut self,
        locale: impl Into<Cow<'a, str>>,
        name: impl Into<Cow<'a, str>>,
    ) -> Self {
        let map = self.name_localizations.get_or_insert_with(Default::default);
        map.insert(locale.into(), name.into());
        self
    }

    /// Sets the description for the option, replacing the current value as set in [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 100 characters.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = description.into();
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
        locale: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
    ) -> Self {
        let map = self.description_localizations.get_or_insert_with(Default::default);
        map.insert(locale.into(), description.into());
        self
    }

    /// Sets if this option is required or optional.
    ///
    /// **Note**: This defaults to `false`.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Adds an optional int-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_int_choice(self, name: impl Into<Cow<'a, str>>, value: i64) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: None,
        })
    }

    /// Adds a localized optional int-choice. See [`Self::add_int_choice`] for more info.
    pub fn add_int_choice_localized(
        self,
        name: impl Into<Cow<'a, str>>,
        value: i64,
        locales: impl Into<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    ) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: value.into(),
            name_localizations: Some(locales.into()),
        })
    }

    /// Adds an optional string-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be up to 100 characters.
    pub fn add_string_choice(
        self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<String>,
    ) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: Value::String(value.into()),
            name_localizations: None,
        })
    }

    /// Adds a localized optional string-choice. See [`Self::add_string_choice`] for more info.
    pub fn add_string_choice_localized(
        self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<String>,
        locales: impl Into<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    ) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: Value::String(value.into()),
            name_localizations: Some(locales.into()),
        })
    }

    /// Adds an optional number-choice.
    ///
    /// **Note**: There can be no more than 25 choices set. Name must be between 1 and 100
    /// characters. Value must be between -2^53 and 2^53.
    pub fn add_number_choice(self, name: impl Into<Cow<'a, str>>, value: f64) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: None,
        })
    }

    /// Adds a localized optional number-choice. See [`Self::add_number_choice`] for more info.
    pub fn add_number_choice_localized(
        self,
        name: impl Into<Cow<'a, str>>,
        value: f64,
        locales: impl Into<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    ) -> Self {
        self.add_choice(CreateCommandOptionChoice {
            name: name.into(),
            value: Value::from(value),
            name_localizations: Some(locales.into()),
        })
    }

    fn add_choice(mut self, value: CreateCommandOptionChoice<'a>) -> Self {
        self.choices.to_mut().push(value);
        self
    }

    /// Optionally enable/disable autocomplete interactions for this option.
    ///
    /// **Notes**:
    /// - May not be set to `true` if `choices` are set
    /// - Options using `autocomplete` are not confined to only use given choices
    pub fn set_autocomplete(mut self, value: bool) -> Self {
        self.autocomplete = value;
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
        sub_options: impl Into<Cow<'a, [CreateCommandOption<'a>]>>,
    ) -> Self {
        self.options = sub_options.into();
        self
    }

    /// If the option is a [`SubCommandGroup`] or [`SubCommand`], nested options are its parameters.
    ///
    /// **Note**: A command can have up to 25 subcommand groups, or subcommands. A subcommand group
    /// can have up to 25 subcommands. A subcommand can have up to 25 options.
    ///
    /// [`SubCommandGroup`]: crate::model::application::CommandOptionType::SubCommandGroup
    /// [`SubCommand`]: crate::model::application::CommandOptionType::SubCommand
    pub fn add_sub_option(mut self, sub_option: CreateCommandOption<'a>) -> Self {
        self.options.to_mut().push(sub_option);
        self
    }

    /// If the option is a [`Channel`], it will only be able to show these types.
    ///
    /// [`Channel`]: crate::model::application::CommandOptionType::Channel
    pub fn channel_types(mut self, channel_types: impl Into<Cow<'a, [ChannelType]>>) -> Self {
        self.channel_types = channel_types.into();
        self
    }

    /// Sets the minimum permitted value for this integer option
    pub fn min_int_value(mut self, value: i64) -> Self {
        self.min_value = Some(value.into());
        self
    }

    /// Sets the maximum permitted value for this integer option
    pub fn max_int_value(mut self, value: i64) -> Self {
        self.max_value = Some(value.into());
        self
    }

    /// Sets the minimum permitted value for this number option
    pub fn min_number_value(mut self, value: f64) -> Self {
        self.min_value = serde_json::Number::from_f64(value);
        self
    }

    /// Sets the maximum permitted value for this number option
    pub fn max_number_value(mut self, value: f64) -> Self {
        self.max_value = serde_json::Number::from_f64(value);
        self
    }

    /// Sets the minimum permitted length for this string option.
    ///
    /// The value of `min_length` must be greater or equal to `0`.
    pub fn min_length(mut self, value: u16) -> Self {
        self.min_length = Some(value);

        self
    }

    /// Sets the maximum permitted length for this string option.
    ///
    /// The value of `max_length` must be greater or equal to `1`.
    pub fn max_length(mut self, value: u16) -> Self {
        self.max_length = Some(value);

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
pub struct CreateCommand<'a> {
    name: Cow<'a, str>,
    name_localizations: HashMap<Cow<'a, str>, Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    description_localizations: HashMap<Cow<'a, str>, Cow<'a, str>>,
    options: Cow<'a, [CreateCommandOption<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_member_permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dm_permission: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    kind: Option<CommandType>,
    nsfw: bool,
}

impl<'a> CreateCommand<'a> {
    /// Creates a new builder with the given name, leaving all other fields empty.
    pub fn new(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: None,

            name: name.into(),
            name_localizations: HashMap::new(),
            description: None,
            description_localizations: HashMap::new(),
            default_member_permissions: None,
            dm_permission: None,

            options: Cow::default(),
            nsfw: false,
        }
    }

    /// Specifies the name of the application command, replacing the current value as set in
    /// [`Self::new`].
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two
    /// global commands of the same app cannot have the same name. Two guild-specific commands of
    /// the same app cannot have the same name.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
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
    pub fn name_localized(
        mut self,
        locale: impl Into<Cow<'a, str>>,
        name: impl Into<Cow<'a, str>>,
    ) -> Self {
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
        self.default_member_permissions = Some(permissions);
        self
    }

    /// Specifies if the command is available in DMs.
    pub fn dm_permission(mut self, enabled: bool) -> Self {
        self.dm_permission = Some(enabled);
        self
    }

    /// Specifies the description of the application command.
    ///
    /// **Note**: Must be between 1 and 100 characters long.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
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
        locale: impl Into<Cow<'a, str>>,
        description: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.description_localizations.insert(locale.into(), description.into());
        self
    }

    /// Adds an application command option for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn add_option(mut self, option: CreateCommandOption<'a>) -> Self {
        self.options.to_mut().push(option);
        self
    }

    /// Sets all the application command options for the application command.
    ///
    /// **Note**: Application commands can have up to 25 options.
    pub fn set_options(mut self, options: impl Into<Cow<'a, [CreateCommandOption<'a>]>>) -> Self {
        self.options = options.into();
        self
    }

    /// Whether this channel is marked NSFW (age-restricted)
    pub fn nsfw(mut self, nsfw: bool) -> Self {
        self.nsfw = nsfw;
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for CreateCommand<'_> {
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

#[derive(Clone, Debug, Serialize)]
struct CreateCommandOptionChoice<'a> {
    pub name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_localizations: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    pub value: Value,
}
