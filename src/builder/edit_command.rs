use std::borrow::Cow;
use std::collections::HashMap;

use crate::builder::CreateCommandOption;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// A builder for editing an existing [`Command`].
///
/// [`Command`]: crate::model::application::Command
///
/// Discord docs:
/// - [global command](https://discord.com/developers/docs/interactions/application-commands#edit-global-application-command)
/// - [guild command](https://discord.com/developers/docs/interactions/application-commands#edit-guild-application-command)
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditCommand<'a> {
    name: Option<Cow<'a, str>>,
    name_localizations: HashMap<Cow<'a, str>, Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Cow<'a, str>>,
    description_localizations: HashMap<Cow<'a, str>, Cow<'a, str>>,
    options: Cow<'a, [CreateCommandOption<'a>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_member_permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "unstable"))]
    dm_permission: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    integration_types: Option<Vec<InstallationContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contexts: Option<Vec<InteractionContext>>,
    nsfw: bool,
}

impl<'a> EditCommand<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies the name of the application command.
    ///
    /// **Note**: Must be between 1 and 32 lowercase characters, matching `r"^[\w-]{1,32}$"`. Two
    /// global commands of the same app cannot have the same name. Two guild-specific commands of
    /// the same app cannot have the same name.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Specifies a localized name of the application command.
    ///
    /// ```rust
    /// # serenity::builder::EditCommand::new()
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

    /// Specifies the default permissions required to execute the command.
    pub fn default_member_permissions(mut self, permissions: Permissions) -> Self {
        self.default_member_permissions = Some(permissions);
        self
    }

    /// Specifies if the command is available in DMs.
    #[cfg(not(feature = "unstable"))]
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

    /// Edit a [`Command`], overwriting an existing one with the same name if it exists.
    ///
    /// Providing a [`GuildId`] will edit a command in the corresponding [`Guild`]. Otherwise, a
    /// global command will be edited.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given. See [Discord's docs] for more details.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Discord's docs]: https://discord.com/developers/docs/interactions/slash-commands
    #[cfg(feature = "http")]
    pub async fn execute(
        self,
        http: &Http,
        command_id: CommandId,
        guild_id: Option<GuildId>,
    ) -> Result<Command> {
        match guild_id {
            Some(guild_id) => http.edit_guild_command(guild_id, command_id, &self).await,
            None => http.edit_global_command(command_id, &self).await,
        }
    }
}
