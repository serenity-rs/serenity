use super::command::{Help, HelpOptions, HelpFunction};
pub use super::{Args, CommandGroup, CommandOptions, CommandError, HelpBehaviour};

use utils::Colour;
use std::fmt::Write;
use std::sync::Arc;

pub struct CreateHelpCommand(pub HelpOptions, pub HelpFunction);

impl CreateHelpCommand {

    /// Sets a message displaying if input could not be found
    /// but a similar command is available.
    ///
    /// **Note**: `{}` will be substituted with the actual suggested command-name.
    /// Hence no `{}` results in no command-name.
    pub fn suggestion_text(mut self, text: &str) -> Self {
        self.0.suggestion_text = text.to_string();

        self
    }

    /// Sets a message displaying if there is no help available.
    pub fn no_help_available_text(mut self, text: &str) -> Self {
        self.0.no_help_available_text = text.to_string();

        self
    }

    /// Sets a label for usage of a command.
    pub fn usage_label(mut self, text: &str) -> Self {
        self.0.usage_label = text.to_string();

        self
    }

    /// Sets a label for the usage examples of a command.
    pub fn usage_sample_label(mut self, text: &str) -> Self {
        self.0.usage_sample_label = text.to_string();

        self
    }

    /// Sets a label for ungrouped-commands
    pub fn ungrouped_label(mut self, text: &str) -> Self {
        self.0.ungrouped_label = text.to_string();

        self
    }

    /// Sets a label for grouped-commands.
    pub fn grouped_label(mut self, text: &str) -> Self {
        self.0.grouped_label = text.to_string();

        self
    }

    /// Sets a label for aliases.
    pub fn aliases_label(mut self, text: &str) -> Self {
        self.0.aliases_label = text.to_string();

        self
    }

    /// Sets a message displaying if a command is only available
    /// in guilds.
    pub fn guild_only_text(mut self, text: &str) -> Self {
        self.0.guild_only_text = text.to_string();

        self
    }

    /// Sets a message displaying if a command is only available
    /// in direct messages (DMs);
    pub fn dm_only_text(mut self, text: &str) -> Self {
        self.0.dm_only_text = text.to_string();

        self
    }

    /// Sets a message displaying if a command is available in
    /// guilds and DMs.
    pub fn dm_and_guilds_text(mut self, text: &str) -> Self {
        self.0.dm_and_guild_text = text.to_string();

        self
    }

    /// Sets a message displaying if a command is available to use.
    pub fn available_text(mut self, text: &str) -> Self {
        self.0.available_text = text.to_string();

        self
    }

    /// Sets a message that will appear upon failing to find
    /// an individual command.
    /// As in: `{prefix}help {command_name}`, but a command or
    /// alias like `{command_name}` does not exist.
    ///
    /// **Note**: `{}` will be substituted with the actual suggested command-name.
    /// Hence no `{}` results in no command-name.
    pub fn command_not_found_text(mut self, text: &str) -> Self {
        self.0.command_not_found_text = text.to_string();

        self
    }

    /// Sets the message on top of the help-menu, informing the
    /// user how to obtain more information about a single command.
    pub fn individual_command_tip(mut self, text: &str) -> Self {
        self.0.individual_command_tip = text.to_string();

        self
    }

    /// Sets how the group-prexix shall be labeled.
    pub fn group_prefix(mut self, text: &str) -> Self {
        self.0.group_prefix = text.to_string();

        self
    }

    /// Sets how a command requiring roles, that a user is lacking,
    /// shall appear in the help-menu.
    pub fn lacking_role(mut self, behaviour: HelpBehaviour) -> Self {
        self.0.lacking_role = behaviour;

        self
    }

    /// Sets how a command requiring permission, that a user is lacking,
    /// shall be appear in the help-menu.
    pub fn lacking_permissions(mut self, behaviour: HelpBehaviour) -> Self {
        self.0.lacking_permissions = behaviour;

        self
    }

    /// Sets how a command requiring to be sent in either via DM
    /// or a guild should be treated in the help-menu.
    pub fn wrong_channel(mut self, behaviour: HelpBehaviour) -> Self {
        self.0.wrong_channel = behaviour;

        self
    }

    /// Sets the tip (or legend) explaining why some commands are striked.
    /// By default this is `Some(String)` and the `String` is empty resulting
    /// in an automated substitution based on your `HelpBehaviour`-settings.
    /// If set to `None`, no tip will be given nor will it be substituted.
    /// If set to a non-empty `Some(String)`, the `String` will be displayed as tip.
    pub fn striked_commands_tip(mut self, text: Option<String>) -> Self {
        self.0.striked_commands_tip = text;

        self
    }

    /// Sets the colour for the embed if no error occured.
    pub fn embed_success_colour(mut self, colour: Colour) -> Self {
        self.0.embed_success_colour = colour;

        self
    }

    /// Sets the colour for the embed if an error occured.
    pub fn embed_error_colour(mut self, colour: Colour) -> Self {
        self.0.embed_error_colour = colour;

        self
    }

    /// Finishes the creation of a help-command, returning `Help`.
    /// If `Some(String)` was set as `striked_commands_tip` and the `String` is empty,
    /// the creator will substitute content based on the `HelpBehaviour`-settings.
    #[cfg_attr(feature = "cargo-clippy", allow(useless_if_let_seq))]
    pub(crate) fn finish(self) -> Arc<Help> {
        if self.0.striked_commands_tip == Some(String::new()) {
            let mut strike_text = String::from("~~`Striked commands`~~ are unavailable because they");

            let mut concat_with_comma = if self.0.lacking_permissions == HelpBehaviour::Strike {
                let _ = write!(strike_text, " require permissions");
                true
            } else {
                false
            };

            if self.0.lacking_role == HelpBehaviour::Strike {

                if concat_with_comma {
                    let _ = write!(strike_text, ", require a specific role");
                } else {
                    let _ = write!(strike_text, " require a specific role");
                    concat_with_comma = true;
                }
            }

            if self.0.wrong_channel == HelpBehaviour::Strike {

                if concat_with_comma {
                    let _ = write!(strike_text, " or are limited to DM/guilds");
                } else {
                    let _ = write!(strike_text, " are limited to DM/guilds");
                }
            }

            let _ = write!(strike_text, ".");
            let CreateHelpCommand(options, function) = self.striked_commands_tip(Some(strike_text));
            return Arc::new(Help(function, Arc::new(options)))
        }
        let CreateHelpCommand(options, function) = self;

        Arc::new(Help(function, Arc::new(options)))
    }
}
