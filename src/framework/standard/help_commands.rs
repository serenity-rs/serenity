//! A collection of default help commands for the framework.
//!
//! # Example
//!
//! Using the [`with_embeds`] function to have the framework's help message use
//! embeds:
//!
//! ```rs,no_run
//! use serenity::framework::standard::help_commands;
//! use serenity::Client;
//! use std::env;
//!
//! let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap());
//! use serenity::framework::StandardFramework;
//!
//! client.with_framework(StandardFramework::new()
//!     .command("help", |c| c.exec_help(help_commands::with_embeds)));
//! ```
//!
//! The same can be accomplished with no embeds by substituting `with_embeds`
//! with the [`plain`] function.
//!
//! [`plain`]: fn.plain.html
//! [`with_embeds`]: fn.with_embeds.html

use client::Context;
#[cfg(feature = "cache")]
use framework::standard::{has_correct_roles, has_correct_permissions};
use model::{
    channel::Message,
    id::ChannelId,
};
use Error;
use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::BuildHasher,
    ops::{Index, IndexMut},
    sync::Arc,
    fmt::Write,
};
use super::command::InternalCommand;
use super::{
    Args,
    CommandGroup,
    CommandOrAlias,
    HelpOptions,
    CommandOptions,
    CommandError,
    HelpBehaviour,
};
use utils::Colour;

/// Macro to format a command according to a `HelpBehaviour` or
/// continue to the next command-name upon hiding.
macro_rules! format_command_name {
    ($behaviour:expr, $command_name:expr) => {
        match $behaviour {
            &HelpBehaviour::Strike => format!("~~`{}`~~", $command_name),
            &HelpBehaviour::Nothing => format!("`{}`", $command_name),
            &HelpBehaviour::Hide => continue,
        }
    };
}

/// Wraps around `warn`-macro in order to keep
/// the literal same for all formats of help.
macro_rules! warn_about_failed_send {
    ($customised_help:expr, $error:expr) => {
        warn!("Failed to send {:?} because: {:?}", $customised_help, $error);
    }
}

/// A single group containing its name and all related commands that are eligible
/// in relation of help-settings measured to the user.
#[derive(Clone, Debug, Default)]
pub struct GroupCommandsPair<'a> {
    name: &'a str,
    prefixes: Vec<String>,
    command_names: Vec<String>,
}

/// A single suggested command containing its name and Levenshtein distance
/// to the actual user's searched command name.
#[derive(Clone, Debug, Default)]
struct SuggestedCommandName {
    name: String,
    levenshtein_distance: usize,
}

/// A single command containing all related pieces of information.
#[derive(Clone, Debug)]
pub struct Command<'a> {
    name: &'a str,
    group_name: &'a str,
    aliases: Vec<String>,
    availability: &'a str,
    description: Option<String>,
    usage: Option<String>,
    usage_sample: Option<String>,
}

/// Contains possible suggestions in case a command could not be found
/// but are similar enough.
#[derive(Clone, Debug, Default)]
pub struct Suggestions(Vec<SuggestedCommandName>);

impl Suggestions {
    /// Immutably borrow inner `Vec`.
    #[inline]
    fn as_vec(&self) -> &Vec<SuggestedCommandName> {
        &self.0
    }

    /// Concats names of suggestions with a given `separator`.
    fn join(&self, separator: &str) -> String {
        let mut iter = self.as_vec().iter();

        let first_iter_element = match iter.next() {
            Some(first_iter_element) => first_iter_element,
            None => return String::new(),
        };

        let size = self.as_vec().iter().fold(0, |total_size, size| total_size + size.name.len());
        let byte_len_of_sep = self.as_vec().len().checked_sub(1).unwrap_or(0) * separator.len();
        let mut result = String::with_capacity(size + byte_len_of_sep);
        result.push_str(first_iter_element.name.borrow());

        for element in iter {
            result.push_str(&*separator);
            result.push_str(element.name.borrow());
        }

        result
    }
}

/// Covers possible outcomes of a help-request and
/// yields relevant data in customised textual
/// representation.
#[derive(Clone, Debug)]
pub enum CustomisedHelpData<'a> {
    /// To display suggested commands.
    SuggestedCommands {
        help_description: String,
        suggestions: Suggestions,
    },
    /// To display groups and their commands by name.
    GroupedCommands {
        help_description: String,
        groups: Vec<GroupCommandsPair<'a>>,
    },
    /// To display one specific command.
    SingleCommand { command: Command<'a> },
    /// To display failure in finding a fitting command.
    NoCommandFound { help_error_message: &'a str },
}

/// Wraps around a `Vec<Vec<T>>` and provides access
/// via indexing of tuples representing x and y.
#[derive(Debug)]
struct Matrix {
    vec: Vec<usize>,
    width: usize,
}

impl Matrix {
    fn new(columns: usize, rows: usize) -> Matrix {
        Matrix {
            vec: vec![0; columns * rows],
            width: rows,
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = usize;

    fn index(&self, matrix_entry: (usize, usize)) -> &usize {
        &self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, matrix_entry: (usize, usize)) -> &mut usize {
        &mut self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

/// Calculates and returns levenshtein distance between
/// two passed words.
pub(crate) fn levenshtein_distance(word_a: &str, word_b: &str) -> usize {
    let len_a = word_a.chars().count();
    let len_b = word_b.chars().count();

    if len_a == 0 {
        return len_b;
    } else if len_b == 0 {
        return len_a;
    }

    let mut matrix = Matrix::new(len_b + 1, len_a + 1);

    for x in 0..len_a {
        matrix[(x + 1, 0)] = matrix[(x, 0)] + 1;
    }

    for y in 0..len_b {
        matrix[(0, y + 1)] = matrix[(0, y)] + 1;
    }

    for (x, char_a) in word_a.chars().enumerate() {

        for (y, char_b) in word_b.chars().enumerate() {

            matrix[(x + 1, y + 1)] = (matrix[(x, y + 1)] + 1)
                .min(matrix[(x + 1, y)] + 1)
                .min(matrix[(x, y)] + if char_a == char_b { 0 } else { 1 });
        }
    }

    matrix[(len_a, len_b)]
}

fn remove_aliases(cmds: &HashMap<String, CommandOrAlias>) -> HashMap<&String, &InternalCommand> {
    let mut result = HashMap::new();

    for (n, v) in cmds {
        if let CommandOrAlias::Command(ref cmd) = *v {
            result.insert(n, cmd);
        }
    }

    result
}

/// Checks whether a user is member of required roles
/// and given the required permissions.
#[cfg(feature = "cache")]
pub fn has_all_requirements(cmd: &Arc<CommandOptions>, msg: &Message) -> bool {
    if let Some(guild) = msg.guild() {
        let guild = guild.read();

        if let Some(member) = guild.members.get(&msg.author.id) {

            if let Ok(permissions) = member.permissions() {

                return if cmd.allowed_roles.is_empty() {
                    permissions.administrator() || has_correct_permissions(cmd, msg)
                } else {
                    permissions.administrator() || (has_correct_roles(cmd, &guild, member) && has_correct_permissions(cmd, msg))
                }
            }
        }
    }
    !cmd.guild_only
}

/// Checks whether a command would be visible, takes permissions, channel sent in,
/// and roles into consideration.
///
/// **Note**: A command is visible when it is either normally displayed or
/// strikethrough upon requested help by a user.
#[cfg(feature = "cache")]
pub fn is_command_visible(command_options: &Arc<CommandOptions>, msg: &Message, help_options: &HelpOptions) -> bool {
    if !command_options.dm_only && !command_options.guild_only
        || command_options.dm_only && msg.is_private()
        || command_options.guild_only && !msg.is_private()
    {

        if let Some(guild) = msg.guild() {
            let guild = guild.read();

            if let Some(member) = guild.members.get(&msg.author.id) {

                if command_options.help_available {

                    return if has_correct_permissions(command_options, msg) {

                        if has_correct_roles(command_options, &guild, &member) {
                            true
                        } else {
                            help_options.lacking_role != HelpBehaviour::Hide
                        }
                    } else {
                        help_options.lacking_permissions != HelpBehaviour::Hide
                    }
                }
            }
        } else if command_options.help_available {
            return if has_correct_permissions(command_options, msg) {
                true
            } else {
                help_options.lacking_permissions != HelpBehaviour::Hide
            }
        }
    } else {
        return help_options.wrong_channel != HelpBehaviour::Hide;
    }

    false
}

/// Tries to extract a single command matching searched command name otherwise
/// returns similar commands.
#[cfg(feature = "cache")]
fn fetch_single_command<'a, H: BuildHasher>(
    groups: &'a HashMap<String, Arc<CommandGroup>, H>,
    name: &str,
    help_options: &'a HelpOptions,
    msg: &Message,
) -> Result<CustomisedHelpData<'a>, Vec<SuggestedCommandName>> {
    let mut similar_commands: Vec<SuggestedCommandName> = Vec::new();

    for (group_name, group) in groups {
        let mut found: Option<(&String, &InternalCommand)> = None;

        for (command_name, command) in &group.commands {

            let search_command_name_matched = if let &Some(ref prefixes) = &group.prefixes {
                prefixes.iter().any(|prefix| {
                    format!("{} {}", prefix, command_name) == name
                })
            } else {
                name == *command_name
            };

            if search_command_name_matched {

                match *command {
                    CommandOrAlias::Command(ref cmd) => {
                        if is_command_visible(&cmd.options(), msg, help_options) {
                            found = Some((command_name, cmd));
                        } else {
                            break;
                        }
                    },
                    CommandOrAlias::Alias(ref name) => {
                        let actual_command = &group.commands[name];

                        match *actual_command {
                            CommandOrAlias::Command(ref cmd) => {
                                if is_command_visible(&cmd.options(), msg, help_options) {
                                    found = Some((name, cmd));
                                } else {
                                    break;
                                }
                            },
                            CommandOrAlias::Alias(ref name) => {
                                return Ok(CustomisedHelpData::SuggestedCommands {
                                    help_description: help_options
                                        .suggestion_text
                                        .replace("{}", name),
                                    suggestions: Suggestions::default(),
                                });
                            }
                        }
                    }
                }
            } else if help_options.max_levenshtein_distance > 0 {

                if let &CommandOrAlias::Command(ref cmd) = command {

                    let command_name = if let &Some(ref prefixes) = &group.prefixes {
                        if let Some(first_prefix) = prefixes.get(0) {
                            format!("{} {}",  &first_prefix, &command_name).to_string()
                        } else {
                            command_name.to_string()
                        }
                    } else {
                        command_name.to_string()
                    };

                    let levenshtein_distance = levenshtein_distance(&command_name, &name);

                    if levenshtein_distance <= help_options.max_levenshtein_distance
                        && is_command_visible(&cmd.options(), &msg, &help_options) {

                        similar_commands.push(SuggestedCommandName {
                            name: command_name,
                            levenshtein_distance,
                        });
                    }
                }
            }
        }

        if let Some((command_name, command)) = found {
            let command = command.options();

            if !command.help_available {
                return Ok(CustomisedHelpData::NoCommandFound {
                    help_error_message: &help_options.no_help_available_text,
                });
            }

            let available_text = if command.dm_only {
                &help_options.dm_only_text
            } else if command.guild_only {
                &help_options.guild_only_text
            } else {
                &help_options.dm_and_guild_text
            };

            similar_commands.sort_unstable_by(|a, b| a.levenshtein_distance.cmp(&b.levenshtein_distance));

            return Ok(CustomisedHelpData::SingleCommand {
                command: Command {
                    name: command_name,
                    description: command.desc.clone(),
                    group_name,
                    aliases: command.aliases.clone(),
                    availability: available_text,
                    usage: command.usage.clone(),
                    usage_sample: command.example.clone(),
                },
            });
        }
    }

    similar_commands.sort_unstable_by(|a, b| a.levenshtein_distance.cmp(&b.levenshtein_distance));

    Err(similar_commands)
}

/// Tries to extract a single command matching searched command name.
#[cfg(feature = "cache")]
fn fetch_all_eligible_commands_in_group<'a>(
    commands: &HashMap<&String, &InternalCommand>,
    command_names: &[&&String],
    help_options: &'a HelpOptions,
    msg: &Message,
) -> GroupCommandsPair<'a> {
    let mut group_with_cmds = GroupCommandsPair::default();

    for name in command_names {
        let name = **name;
        let cmd = &commands[&*name];
        let cmd = cmd.options();

        if !cmd.help_available {
            continue;
        }

        if !cmd.dm_only && !cmd.guild_only
            || cmd.dm_only && msg.is_private()
            || cmd.guild_only && !msg.is_private()
        {
            if has_correct_permissions(&cmd, msg) {

                if let Some(guild) = msg.guild() {
                    let guild = guild.read();

                    if let Some(member) = guild.members.get(&msg.author.id) {

                        if has_correct_roles(&cmd, &guild, &member) {
                            group_with_cmds.command_names.push(format!("`{}`", &name));
                        } else {
                            let name = format_command_name!(&help_options.lacking_role, &name);
                            group_with_cmds.command_names.push(name);
                        }
                    }
                } else {
                    group_with_cmds.command_names.push(format!("`{}`", &name));
                }
            } else {
                let name = format_command_name!(&help_options.lacking_permissions, &name);
                group_with_cmds.command_names.push(name);
            }
        } else {
            let name = format_command_name!(&help_options.wrong_channel, &name);
            group_with_cmds.command_names.push(name);
        }
    }

    group_with_cmds
}

/// Fetch groups with their commands.
#[cfg(feature = "cache")]
fn create_command_group_commands_pair_from_groups<'a, H: BuildHasher>(
    groups: &'a HashMap<String, Arc<CommandGroup>, H>,
    group_names: &[&'a String],
    msg: &Message,
    help_options: &'a HelpOptions,
) -> Vec<GroupCommandsPair<'a>> {
    let mut listed_groups: Vec<GroupCommandsPair> = Vec::default();

    for group_name in group_names {
        let group = &groups[&**group_name];

        let group_with_cmds = create_single_group(
            group,
            group_name,
            &msg,
            &help_options,
        );

        if !group_with_cmds.command_names.is_empty() {
            listed_groups.push(group_with_cmds);
        }
    }

    listed_groups
}

/// Fetches a single group with its commands.
#[cfg(feature = "cache")]
fn create_single_group<'a>(
    group: &CommandGroup,
    group_name: &'a str,
    msg: &Message,
    help_options: &'a HelpOptions,
) -> GroupCommandsPair<'a> {
let commands = remove_aliases(&group.commands);
    let mut command_names = commands.keys().collect::<Vec<_>>();
    command_names.sort();

    let mut group_with_cmds = fetch_all_eligible_commands_in_group(
        &commands,
        &command_names,
        &help_options,
        &msg,
    );

    group_with_cmds.name = group_name;

    if let Some(ref prefixes) = group.prefixes {
        group_with_cmds.prefixes.extend_from_slice(&prefixes);
    }

    group_with_cmds
}

/// Iterates over all commands and forges them into a `CustomisedHelpData`
/// taking `HelpOptions` into consideration when deciding on whether a command
/// shall be picked and in what textual format.
#[cfg(feature = "cache")]
pub fn create_customised_help_data<'a, H: BuildHasher>(
    groups: &'a HashMap<String, Arc<CommandGroup>, H>,
    args: &'a Args,
    help_options: &'a HelpOptions,
    msg: &Message,
) -> CustomisedHelpData<'a> {
    if !args.is_empty() {
        let name = args.full();

        return match fetch_single_command(&groups, &name, &help_options, &msg) {
            Ok(single_command) => single_command,
            Err(suggestions) => {
                let searched_named_lowercase = name.to_lowercase();

                for (key, group) in groups {

                    if key.to_lowercase() == searched_named_lowercase
                        || group.prefixes.as_ref()
                            .map_or(false, |v| v.iter().any(|prefix|
                            *prefix == searched_named_lowercase)) {

                        let mut single_group = create_single_group(
                            &group,
                            &key,
                            &msg,
                            &help_options
                        );

                        if !single_group.command_names.is_empty() {
                            return CustomisedHelpData::GroupedCommands {
                                help_description: group.description.clone().unwrap_or_default(),
                                groups: vec![single_group],
                            };
                        }
                    }
                }

                if suggestions.is_empty() {
                    CustomisedHelpData::NoCommandFound {
                        help_error_message: &help_options.no_help_available_text,
                    }
                } else {
                    CustomisedHelpData::SuggestedCommands {
                        help_description: help_options.suggestion_text.clone(),
                        suggestions: Suggestions(suggestions),
                    }
                }
            },
        };
    }

    let strikethrough_command_tip = if msg.is_private() {
        &help_options.striked_commands_tip_in_guild
    } else {
        &help_options.striked_commands_tip_in_dm
    };

    let description = if let &Some(ref striked_command_text) = strikethrough_command_tip {
        format!(
            "{}\n{}",
            &help_options.individual_command_tip, &striked_command_text
        )
    } else {
        help_options.individual_command_tip.clone()
    };

    let mut group_names = groups.keys().collect::<Vec<_>>();
    group_names.sort();

    let listed_groups =
        create_command_group_commands_pair_from_groups(&groups, &group_names, &msg, &help_options);

    return if listed_groups.is_empty() {
        CustomisedHelpData::NoCommandFound {
            help_error_message: &help_options.no_help_available_text,
        }
    } else {
        CustomisedHelpData::GroupedCommands {
            help_description: description,
            groups: listed_groups,
        }
    };
}

/// Sends an embed listing all groups with their commands.
fn send_grouped_commands_embed(
    help_options: &HelpOptions,
    channel_id: ChannelId,
    help_description: &str,
    groups: &[GroupCommandsPair],
    colour: Colour,
) -> Result<Message, Error> {
    channel_id.send_message(|m| {
        m.embed(|mut embed| {
            embed = embed.colour(colour).description(help_description);

            for group in groups {
                let joined_command_text_body = group.command_names.join("\n");

                let field_text = match group.prefixes.len() {
                    0 => joined_command_text_body,
                    _ => format!(
                        "{}: `{}`\n{}",
                        help_options.group_prefix,
                        group.prefixes.join("`, `"),
                        joined_command_text_body
                    ),
                };

                embed = embed.field(group.name, field_text, true);
            }

            embed
        })
    })
}

/// Sends embed showcasing information about a single command.
fn send_single_command_embed(
    help_options: &HelpOptions,
    channel_id: ChannelId,
    command: &Command,
    colour: Colour,
) -> Result<Message, Error> {
    channel_id.send_message(|m| {
        m.embed(|mut embed| {
            embed = embed.title(&command.name).colour(colour);

            if let &Some(ref desc) = &command.description {
                embed = embed.description(desc);
            }

            if let &Some(ref usage) = &command.usage {
                embed = embed.field(&help_options.usage_label, usage, true);
            }

            if let Some(ref usage_sample) = command.usage_sample {
                let value = format!("`{} {}`", command.name, usage_sample);

                embed = embed.field(&help_options.usage_sample_label, value, true);
            }

            embed = embed.field(&help_options.grouped_label, command.group_name, true);

            if !command.aliases.is_empty() {
                embed = embed.field(
                    &help_options.aliases_label,
                    format!("`{}`", command.aliases.join("`, `")),
                    true,
                );
            }

            embed.field(&help_options.available_text, &command.availability, true)
        })
    })
}

/// Sends embed listing commands that are similar to the sent one.
fn send_suggestion_embed(
    channel_id: ChannelId,
    help_description: &str,
    suggestions: &Suggestions,
    colour: Colour,
) -> Result<Message, Error> {
    let text = format!("{}", help_description.replace("{}", &suggestions.join("`, `")));

    channel_id.send_message(|m| m.embed(|e| e.colour(colour).description(text)))
}

/// Sends an embed explaining fetching commands failed.
fn send_error_embed(channel_id: ChannelId, input: &str, colour: Colour) -> Result<Message, Error> {
    channel_id.send_message(|m| m.embed(|e| e.colour(colour).description(input)))
}

/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::with_embeds));
/// ```
#[cfg(feature = "cache")]
pub fn with_embeds<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    args: &Args
) -> Result<(), CommandError> {
    let formatted_help = create_customised_help_data(&groups, args, help_options, msg);

    if let Err(why) = match &formatted_help {
        &CustomisedHelpData::SuggestedCommands { ref help_description, ref suggestions } =>
            send_suggestion_embed(
                msg.channel_id,
                &help_description,
                &suggestions,
                help_options.embed_error_colour,
            ),
        &CustomisedHelpData::NoCommandFound { ref help_error_message } =>
            send_error_embed(
                msg.channel_id,
                help_error_message,
                help_options.embed_error_colour,
            ),
        &CustomisedHelpData::GroupedCommands { ref help_description, ref groups } =>
            send_grouped_commands_embed(
                &help_options,
                msg.channel_id,
                &help_description,
                &groups,
                help_options.embed_success_colour,
            ),
        &CustomisedHelpData::SingleCommand { ref command } =>
            send_single_command_embed(
                &help_options,
                msg.channel_id,
                &command,
                help_options.embed_success_colour,
            ),
    } {
        warn_about_failed_send!(&formatted_help, why);
    }

    Ok(())
}

/// Turns grouped commands into a `String` taking plain help format into account.
fn grouped_commands_to_plain_string(
    help_options: &HelpOptions,
    help_description: &str,
    groups: &[GroupCommandsPair]) -> String
{
    let mut result = "**Commands**\n".to_string();
    let _ = writeln!(result, "{}", &help_description);

    for group in groups {
        let _ = write!(result, "\n**{}**", &group.name);

        if !group.prefixes.is_empty() {
            let _ = write!(result, " ({}: `{}`)", &help_options.group_prefix, &group.prefixes.join("`, `"));
        }

        let _ = write!(result, ": {}", group.command_names.join(" "));
    }

    result
}

/// Turns a single command into a `String` taking plain help format into account.
fn single_command_to_plain_string(help_options: &HelpOptions, command: &Command) -> String {
    let mut result = String::default();
    let _ = writeln!(result, "**{}**", command.name);

    if !command.aliases.is_empty() {
        let _ = writeln!(result, "**{}**: `{}`", help_options.aliases_label, command.aliases.join("`, `"));
    }

    if let &Some(ref description) = &command.description {
        let _ = writeln!(result, "**{}**: {}", help_options.description_label, description);
    };

    if let &Some(ref usage) = &command.usage {
        let _ = writeln!(result, "**{}**: {}", help_options.usage_label, usage);
    }

    if let &Some(ref usage_sample) = &command.usage_sample {
        let _ = writeln!(result, "**{}**: `{} {}`", help_options.usage_sample_label, command.name, usage_sample);
    }

    let _ = writeln!(result, "**{}**: {}", help_options.grouped_label, command.group_name);
    let _ = writeln!(result, "**{}**: {}", help_options.available_text, command.availability);

    result
}

/// Posts formatted text displaying each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// # struct Handler;
/// #
/// # impl EventHandler for Handler {}
/// # let mut client = Client::new("token", Handler).unwrap();
/// #
/// use serenity::framework::standard::{StandardFramework, help_commands};
///
/// client.with_framework(StandardFramework::new()
///     .help(help_commands::plain));
/// ```
#[cfg(feature = "cache")]
pub fn plain<H: BuildHasher>(
    _: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>, H>,
    args: &Args
) -> Result<(), CommandError> {
    let formatted_help = create_customised_help_data(&groups, args, help_options, msg);

    let result = match &formatted_help {
        &CustomisedHelpData::SuggestedCommands { ref help_description, ref suggestions } =>
            format!("{}: `{}`", help_description, suggestions.join("`, `")),
        &CustomisedHelpData::NoCommandFound { ref help_error_message } =>
            help_error_message.to_string(),
        &CustomisedHelpData::GroupedCommands { ref help_description, ref groups } =>
            grouped_commands_to_plain_string(&help_options, &help_description, &groups),
        &CustomisedHelpData::SingleCommand { ref command } => {
            single_command_to_plain_string(&help_options, &command)
        },
    };

    if let Err(why) = msg.channel_id.say(result) {
        warn_about_failed_send!(&formatted_help, why);
    };

    Ok(())
}


#[cfg(test)]
mod levenshtein_tests {
    use super::levenshtein_distance;

    #[test]
    fn reflexive() {
        let word_a = "rusty ferris";
        let word_b = "rusty ferris";
        assert_eq!(0, levenshtein_distance(&word_a, &word_b));

        let word_a = "";
        let word_b = "";
        assert_eq!(0, levenshtein_distance(&word_a, &word_b));

        let word_a = "rusty ferris";
        let word_b = "RuSty FerriS";
        assert_eq!(4, levenshtein_distance(&word_a, &word_b));
    }

    #[test]
    fn symmetric() {
        let word_a = "ferris";
        let word_b = "rusty ferris";
        assert_eq!(6, levenshtein_distance(&word_a, &word_b));

        let word_a = "rusty ferris";
        let word_b = "ferris";
        assert_eq!(6, levenshtein_distance(&word_a, &word_b));

        let word_a = "";
        let word_b = "ferris";
        assert_eq!(6, levenshtein_distance(&word_a, &word_b));

        let word_a = "ferris";
        let word_b = "";
        assert_eq!(6, levenshtein_distance(&word_a, &word_b));
    }

    #[test]
    fn transitive() {
        let word_a = "ferris";
        let word_b = "turbo fish";
        let word_c = "unsafe";

        let distance_of_a_c = levenshtein_distance(&word_a, &word_c);
        let distance_of_a_b = levenshtein_distance(&word_a, &word_b);
        let distance_of_b_c = levenshtein_distance(&word_b, &word_c);

        assert!(distance_of_a_c <= (distance_of_a_b + distance_of_b_c));
    }
}

#[cfg(test)]
mod matrix_tests {
    use super::Matrix;

    #[test]
    fn index_mut() {
        let mut matrix = Matrix::new(5, 5);
        assert_eq!(matrix[(1, 1)], 0);

        matrix[(1, 1)] = 10;
        assert_eq!(matrix[(1, 1)], 10);
    }

    #[test]
    #[should_panic(expected = "the len is 4 but the index is 9")]
    fn panic_index_too_high() {
        let matrix = Matrix::new(2, 2);
        matrix[(3, 3)];
    }

    #[test]
    #[should_panic(expected = "the len is 0 but the index is 0")]
    fn panic_indexing_when_empty() {
        let matrix = Matrix::new(0, 0);
        matrix[(0, 0)];
    }
}
