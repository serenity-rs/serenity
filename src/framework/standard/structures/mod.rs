use std::{
    collections::HashSet,
    fmt,
};
use crate::client::Context;
use crate::model::{
    channel::Message,
    permissions::Permissions,
    id::UserId,
};
use crate::utils::Colour;
use super::Args;

mod check;
pub mod buckets;

pub use self::check::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnlyIn {
    Dm,
    Guild,
    None,
    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug, PartialEq)]
pub struct CommandOptions {
    /// A set of checks to be called prior to executing the command. The checks
    /// will short-circuit on the first check that returns `false`.
    pub checks: &'static [&'static Check],
    /// Ratelimit bucket.
    pub bucket: Option<&'static str>,
    /// Names that the command can be referred to.
    pub names: &'static [&'static str],
    /// Command description, used by other commands.
    pub desc: Option<&'static str>,
    /// Delimiters used to split the arguments of the command by.
    /// If empty, the [global delimiters](struct.Configuration.html#method.delimiters) are used.
    pub delimiters: &'static [&'static str],
    /// Command usage schema, used by other commands.
    pub usage: Option<&'static str>,
    /// Example arguments, used by other commands.
    pub examples: &'static [&'static str],
    /// Minimum amount of arguments that should be passed.
    pub min_args: Option<u16>,
    /// Maximum amount of arguments that can be passed.
    pub max_args: Option<u16>,
    /// Roles allowed to use this command.
    pub allowed_roles: &'static [&'static str],
    /// Permissions required to use this command.
    pub required_permissions: Permissions,
    /// Whether the command should be displayed in help list or not, used by other commands.
    pub help_available: bool,
    /// Whether the command can only be used in dms or guilds; or both.
    pub only_in: OnlyIn,
    /// Whether the command can only be used by owners or not.
    pub owners_only: bool,
    /// Whether the command treats owners as normal users.
    pub owner_privilege: bool,
    /// Other commands belonging to this command.
    pub sub_commands: &'static [&'static Command],
}

#[derive(Debug, Clone)]
pub struct CommandError(pub String);

impl<T: fmt::Display> From<T> for CommandError {
    #[inline]
    fn from(d: T) -> Self {
        CommandError(d.to_string())
    }
}

pub type CommandResult = ::std::result::Result<(), CommandError>;

pub type CommandFn = fn(&mut Context, &Message, Args) -> CommandResult;

pub struct Command {
    pub fun: CommandFn,
    pub options: &'static CommandOptions,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("options", &self.options)
            .finish()
    }
}

impl PartialEq for Command {
    #[inline]
    fn eq(&self, other: &Command) -> bool {
        (self.fun as usize == other.fun as usize) && (self.options == other.options)
    }
}

pub type HelpCommandFn = fn(
    &mut Context,
    &Message,
    Args,
    &'static HelpOptions,
    &[&'static CommandGroup],
    HashSet<UserId>,
) -> CommandResult;

pub struct HelpCommand {
    pub fun: HelpCommandFn,
    pub options: &'static HelpOptions,
}

impl fmt::Debug for HelpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelpCommand")
            .field("fun", &"<function>")
            .field("options", &self.options)
            .finish()
    }
}

impl PartialEq for HelpCommand {
    #[inline]
    fn eq(&self, other: &HelpCommand) -> bool {
        (self.fun as usize == other.fun as usize) && (self.options == other.options)
    }
}

/// Describes the behaviour the help-command shall execute once it encounters
/// a command which the user or command fails to meet following criteria :
/// Lacking required permissions to execute the command.
/// Lacking required roles to execute the command.
/// The command can't be used in the current channel (as in `DM only` or `guild only`).
#[derive(Copy, Clone, Debug, PartialOrd, Ord, Eq, PartialEq)]
pub enum HelpBehaviour {
    /// The command will be displayed, hence nothing will be done.
    Nothing,
    /// Strikes a command by applying `~~{command_name}~~`.
    Strike,
    /// Does not list a command in the help-menu.
    Hide,
    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HelpOptions {
    /// Which names should the help command use for dispatching.
    /// Defaults to `["help"]`
    pub names: &'static [&'static str],
    /// Suggests a command's name.
    pub suggestion_text: &'static str,
    /// If no help is available, this text will be displayed.
    pub no_help_available_text: &'static str,
    /// How to use a command, `{usage_label}: {command_name} {args}`
    pub usage_label: &'static str,
    /// Actual sample label, `{usage_sample_label}: {command_name} {args}`
    pub usage_sample_label: &'static str,
    /// Text labeling ungrouped commands, `{ungrouped_label}: ...`
    pub ungrouped_label: &'static str,
    /// Text labeling the start of the description.
    pub description_label: &'static str,
    /// Text labeling grouped commands, `{grouped_label} {group_name}: ...`
    pub grouped_label: &'static str,
    /// Text labeling a command's alternative names (aliases).
    pub aliases_label: &'static str,
    /// Text specifying that a command is only usable in a guild.
    pub guild_only_text: &'static str,
    /// Text labeling a command's names of checks.
    pub checks_label: &'static str,
    /// Text specifying that a command is only usable in via DM.
    pub dm_only_text: &'static str,
    /// Text specifying that a command can be used via DM and in guilds.
    pub dm_and_guild_text: &'static str,
    /// Text expressing that a command is available.
    pub available_text: &'static str,
    /// Error-message once a command could not be found.
    /// Output-example (without whitespace between both substitutions: `{command_not_found_text}{command_name}`
    /// `{command_name}` describes user's input as in: `{prefix}help {command_name}`.
    pub command_not_found_text: &'static str,
    /// Explains the user on how to use access a single command's details.
    pub individual_command_tip: &'static str,
    /// Explains reasoning behind strikethrough-commands, see fields requiring `HelpBehaviour` for further information.
    /// If `HelpBehaviour::Strike` is unused, this field will evaluate to `None` during creation
    /// inside of the help macro.
    ///
    /// **Note**: Text is only used in direct messages.
    pub strikethrough_commands_tip_in_dm: Option<&'static str>,
    /// Explains reasoning behind strikethrough-commands, see fields requiring `HelpBehaviour` for further information.
    /// If `HelpBehaviour::Strike` is unused, this field will evaluate to `None` during creation
    /// inside of the help macro.
    ///
    /// **Note**: Text is only used in guilds.
    pub strikethrough_commands_tip_in_guild: Option<&'static str>,
    /// Announcing a group's prefix as in: {group_prefix} {prefix}.
    pub group_prefix: &'static str,
    /// If a user lacks required roles, this will treat how these commands will be displayed.
    pub lacking_role: HelpBehaviour,
    /// If a user lacks permissions, this will treat how these commands will be displayed.
    pub lacking_permissions: HelpBehaviour,
    /// If a user lacks ownership, this will treat how these commands will be displayed.
    pub lacking_ownership: HelpBehaviour,
    /// If conditions (of a check) may be lacking by the user, this will treat how these commands will be displayed.
    pub lacking_conditions: HelpBehaviour,
    /// If a user is using the help-command in a channel where a command is not available,
    /// this behaviour will be executed.
    pub wrong_channel: HelpBehaviour,
    /// Colour help-embed will use upon encountering an error.
    pub embed_error_colour: Colour,
    /// Colour help-embed will use if no error occurred.
    pub embed_success_colour: Colour,
    /// If not 0, help will check whether a command is similar to searched named.
    pub max_levenshtein_distance: usize,
    /// Help will use this as prefix to express how deeply nested a command or
    /// group is.
    pub indention_prefix: &'static str,
}

#[derive(Debug, PartialEq)]
pub struct GroupOptions {
    pub prefixes: &'static [&'static str],
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub help_available: bool,
    pub allowed_roles: &'static [&'static str],
    pub required_permissions: Permissions,
    pub checks: &'static [&'static Check],
    pub default_command: Option<&'static Command>,
    pub description: Option<&'static str>,
    pub commands: &'static [&'static Command],
    pub sub_groups: &'static [&'static CommandGroup],
}
#[derive(Debug, PartialEq)]
pub struct CommandGroup {
    pub name: &'static str,
    pub options: &'static GroupOptions,
}

#[cfg(test)]
#[cfg(all(feature = "cache", feature = "http"))]
mod levenshtein_tests {
    use super::HelpBehaviour;

    #[test]
    fn help_behaviour_eq() {
        assert_eq!(HelpBehaviour::Hide, std::cmp::max(HelpBehaviour::Hide, HelpBehaviour::Hide));
        assert_eq!(HelpBehaviour::Strike, std::cmp::max(HelpBehaviour::Strike, HelpBehaviour::Strike));
        assert_eq!(HelpBehaviour::Nothing, std::cmp::max(HelpBehaviour::Nothing, HelpBehaviour::Nothing));
    }

    #[test]
    fn help_behaviour_hide() {
        assert_eq!(HelpBehaviour::Hide, std::cmp::max(HelpBehaviour::Hide, HelpBehaviour::Nothing));
        assert_eq!(HelpBehaviour::Hide, std::cmp::max(HelpBehaviour::Hide, HelpBehaviour::Strike));
    }

    #[test]
    fn help_behaviour_strike() {
        assert_eq!(HelpBehaviour::Strike, std::cmp::max(HelpBehaviour::Strike, HelpBehaviour::Nothing));
        assert_eq!(HelpBehaviour::Hide, std::cmp::max(HelpBehaviour::Strike, HelpBehaviour::Hide));
    }

    #[test]
    fn help_behaviour_nothing() {
        assert_eq!(HelpBehaviour::Strike, std::cmp::max(HelpBehaviour::Nothing, HelpBehaviour::Strike));
        assert_eq!(HelpBehaviour::Hide, std::cmp::max(HelpBehaviour::Nothing, HelpBehaviour::Hide));
    }
}
