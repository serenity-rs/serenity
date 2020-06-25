//! A collection of default help commands for the framework.
//!
//! # Example
//!
//! Using the [`with_embeds`] function to have the framework's help message use
//! embeds:
//!
//! ```rust,no_run
//! use serenity::framework::standard::{
//!     StandardFramework,
//!     help_commands,
//!     Args,
//!     HelpOptions,
//!     CommandGroup,
//!     CommandResult,
//! };
//! use serenity::framework::standard::macros::help;
//! use serenity::model::prelude::{Message, UserId};
//! use serenity::client::{EventHandler, Context, Client};
//! use std::collections::HashSet;
//! use std::env;
//!
//! struct Handler;
//!
//! impl EventHandler for Handler {}
//!
//! #[help]
//! fn my_help(
//!    context: &Context,
//!    msg: &Message,
//!    args: Args,
//!    help_options: &'static HelpOptions,
//!    groups: &[&'static CommandGroup],
//!    owners: HashSet<UserId>
//! ) -> CommandResult {
//! #  #[cfg(all(feature = "cache", feature = "http"))]
//! # {
//!    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners);
//!    Ok(())
//! # }
//! #
//! # #[cfg(not(all(feature = "cache", feature = "http")))]
//! # Ok(())
//! }
//!
//! let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), Handler).unwrap();
//!
//! client.with_framework(StandardFramework::new()
//!     .help(&MY_HELP));
//! ```
//!
//! The same can be accomplished with no embeds by substituting `with_embeds`
//! with the [`plain`] function.
//!
//! [`plain`]: fn.plain.html
//! [`with_embeds`]: fn.with_embeds.html

#[cfg(all(feature = "cache", feature = "http"))]
use super::{
    Args, CommandGroup, CommandOptions, CheckResult,
    has_correct_roles, HelpBehaviour, HelpOptions,
    has_correct_permissions, OnlyIn,
    structures::Command as InternalCommand,
};
#[cfg(all(feature = "cache", feature = "http"))]
use crate::{
    cache::CacheRwLock,
    client::Context,
    framework::standard::CommonOptions,
    model::channel::Message,
    Error,
    http::Http,
    model::id::{ChannelId, UserId},
    utils::Colour,
};
#[cfg(all(feature = "cache", feature = "http"))]
use std::{
    borrow::Borrow,
    collections::HashSet,
    fmt::Write,
    ops::{Index, IndexMut},
};
#[cfg(all(feature = "cache", feature = "http"))]
use log::warn;

/// Macro to format a command according to a `HelpBehaviour` or
/// continue to the next command-name upon hiding.
#[cfg(all(feature = "cache", feature = "http"))]
macro_rules! format_command_name {
    ($behaviour:expr, $command_name:expr) => {
        match $behaviour {
            HelpBehaviour::Strike => format!("~~`{}`~~", $command_name),
            HelpBehaviour::Nothing => format!("`{}`", $command_name),
            HelpBehaviour::Hide => continue,
            HelpBehaviour::__Nonexhaustive => unreachable!(),
        }
    };
}

/// Wraps around `warn`-macro in order to keep
/// the literal same for all formats of help.
#[cfg(all(feature = "cache", feature = "http"))]
macro_rules! warn_about_failed_send {
    ($customised_help:expr, $error:expr) => {
        warn!("Failed to send {:?} because: {:?}", $customised_help, $error);
    }
}

/// A single group containing its name and all related commands that are eligible
/// in relation of help-settings measured to the user.
#[derive(Clone, Debug, Default)]
pub struct GroupCommandsPair {
    name: &'static str,
    prefixes: Vec<&'static str>,
    command_names: Vec<String>,
    sub_groups: Vec<GroupCommandsPair>,
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
    name: &'static str,
    group_name: &'static str,
    group_prefixes: &'a [&'static str],
    aliases: Vec<&'static str>,
    availability: &'a str,
    description: Option<&'static str>,
    usage: Option<&'static str>,
    usage_sample: Vec<&'static str>,
    checks: Vec<String>,
}

/// Contains possible suggestions in case a command could not be found
/// but are similar enough.
#[derive(Clone, Debug, Default)]
pub struct Suggestions(Vec<SuggestedCommandName>);

#[cfg(all(feature = "cache", feature = "http"))]
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

        let size = self
            .as_vec()
            .iter()
            .fold(0, |total_size, size| total_size + size.name.len());
        let byte_len_of_sep = self.as_vec().len().saturating_sub(1) * separator.len();
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
        groups: Vec<GroupCommandsPair>,
    },
    /// To display one specific command.
    SingleCommand { command: Command<'a> },
    /// To display failure in finding a fitting command.
    NoCommandFound { help_error_message: &'a str },
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Wraps around a `Vec<Vec<T>>` and provides access
/// via indexing of tuples representing x and y.
#[derive(Debug)]
#[cfg(all(feature = "cache", feature = "http"))]
struct Matrix {
    vec: Vec<usize>,
    width: usize,
}

#[cfg(all(feature = "cache", feature = "http"))]
impl Matrix {
    fn new(columns: usize, rows: usize) -> Matrix {
        Matrix {
            vec: vec![0; columns * rows],
            width: rows,
        }
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
impl Index<(usize, usize)> for Matrix {
    type Output = usize;

    fn index(&self, matrix_entry: (usize, usize)) -> &usize {
        &self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

#[cfg(all(feature = "cache", feature = "http"))]
impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, matrix_entry: (usize, usize)) -> &mut usize {
        &mut self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

/// Calculates and returns levenshtein distance between
/// two passed words.
#[cfg(all(feature = "cache", feature = "http"))]
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

/// Checks whether a user is member of required roles
/// and given the required permissions.
#[cfg(feature = "cache")]
pub fn has_all_requirements(
    cache: impl AsRef<CacheRwLock>,
    cmd: &CommandOptions,
    msg: &Message,
) -> bool {
    if let Some(guild) = msg.guild(&cache) {
        let guild = guild.read();

        if let Some(member) = guild.members.get(&msg.author.id) {

            if let Ok(permissions) = member.permissions(&cache) {

                return if cmd.allowed_roles.is_empty() {
                    permissions.administrator() || has_correct_permissions(&cache, &cmd, msg)
                } else {
                    permissions.administrator()
                        || (has_correct_roles(&cmd, &guild, member)
                            && has_correct_permissions(&cache, &cmd, msg))
                };
            }
        }
    }

    cmd.only_in != OnlyIn::Guild
}

/// Checks if `search_on` starts with `word` and is then cleanly followed by a
/// `" "`.
#[inline]
#[cfg(all(feature = "cache", feature = "http"))]
fn starts_with_whole_word(search_on: &str, word: &str) -> bool {
    search_on.starts_with(word) && search_on.get(word.len()..=word.len())
        .map_or(false, |slice| slice == " ")
}

#[inline]
#[cfg(all(feature = "cache", feature = "http"))]
fn find_any_command_matches(
    command: &'static InternalCommand,
    group: &CommandGroup,
    name_to_find: &mut String,
    found_prefix: &mut bool,
) -> Option<&'static str> {

    command
        .options
        .names
        .iter()
        .find(|command_name| {
            group
                .options
                .prefixes
                .iter()
                .any(|prefix| {
                    if *found_prefix || starts_with_whole_word(&name_to_find, &prefix) {

                        if !*found_prefix {
                            *found_prefix = true;
                            name_to_find.drain(..=prefix.len());
                        }

                        &name_to_find == command_name
                    } else {
                        false
                    }
                })
        }).cloned()
}


#[cfg(all(feature = "cache", feature = "http"))]
fn check_common_behaviour(
    cache: impl AsRef<CacheRwLock>,
    msg: &Message,
    options: &impl CommonOptions,
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    if !options.help_available() {
        return HelpBehaviour::Hide;
    }

    if options.only_in() == OnlyIn::Dm && !msg.is_private() ||
       options.only_in() == OnlyIn::Guild && msg.is_private() {
        return help_options.wrong_channel;
    }

    if options.owners_only() && !owners.contains(&msg.author.id) {
        return help_options.lacking_ownership;
    }

    if options.owner_privilege() && owners.contains(&msg.author.id) {
        return HelpBehaviour::Nothing;
    }

    if !has_correct_permissions(&cache, options, msg) {
        return help_options.lacking_permissions;
    }

    if let Some(guild) = msg.guild(&cache) {
        let guild = guild.read();

        if let Some(member) = guild.members.get(&msg.author.id) {
            if !has_correct_roles(options, &guild, &member) {
                return help_options.lacking_role;
            }
        }
    }

    HelpBehaviour::Nothing
}

#[cfg(all(feature = "cache", feature = "http"))]
fn check_command_behaviour(
    ctx: &Context,
    msg: &Message,
    options: &CommandOptions,
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    let b = check_common_behaviour(&ctx, msg, &options, owners, help_options);

    if b == HelpBehaviour::Nothing {
       for check in options.checks {
           if !check.check_in_help {
               break;
           }

           let mut args = Args::new("", &[]);

           if let CheckResult::Failure(_) = (check.function)(ctx, msg, &mut args, options) {
               return help_options.lacking_conditions;
           }
       }
    }

    b
}

#[cfg(all(feature = "cache", feature = "http"))]
#[allow(clippy::too_many_arguments)]
fn nested_group_command_search<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    name: &mut String,
    help_options: &'a HelpOptions,
    similar_commands: &mut Vec<SuggestedCommandName>,
    owners: &HashSet<UserId>,
) -> Result<CustomisedHelpData<'a>, ()> {
    for group in groups {
        let group = *group;
        let mut found: Option<&'static InternalCommand> = None;

        let group_behaviour = check_common_behaviour(
                &ctx,
                msg,
                &group.options,
                &owners,
                &help_options,
        );

        match &group_behaviour {
            HelpBehaviour::Nothing => (),
            _ => {
                continue;
            }
        }

        let mut found_group_prefix: bool = false;
        for command in group.options.commands {
            let command = *command;

            let search_command_name_matched = if group.options.prefixes.is_empty() {
                if starts_with_whole_word(&name, &group.name) {
                    name.drain(..=group.name.len());
                }

                command
                    .options
                    .names
                    .iter()
                    .find(|n| **n == name)
                    .cloned()
            } else {
                find_any_command_matches(
                    &command,
                    &group,
                    name,
                    &mut found_group_prefix
                )
            };

            if search_command_name_matched.is_some() {

                if HelpBehaviour::Nothing == check_command_behaviour(
                    ctx,
                    msg,
                    &command.options,
                    &owners,
                    &help_options,
                ) {
                    found = Some(command);
                } else {
                    break;
                }
            } else if help_options.max_levenshtein_distance > 0 {

                let command_name = if let Some(first_prefix) = group.options.prefixes.get(0) {
                    format!("{} {}", &first_prefix, &command.options.names[0])
                } else {
                    command.options.names[0].to_string()
                };

                let levenshtein_distance = levenshtein_distance(&command_name, &name);

                if levenshtein_distance <= help_options.max_levenshtein_distance
                    && HelpBehaviour::Nothing == check_command_behaviour(
                        ctx,
                        msg,
                        &command.options,
                        &owners,
                        &help_options,
                    )
                {
                    similar_commands.push(SuggestedCommandName {
                        name: command_name,
                        levenshtein_distance,
                    });
                }
            }
        }

        if let Some(command) = found {
            let options = &command.options;

            if !options.help_available {
                return Ok(CustomisedHelpData::NoCommandFound {
                    help_error_message: &help_options.no_help_available_text,
                });
            }

            let available_text = if options.only_in == OnlyIn::Dm {
                &help_options.dm_only_text
            } else if options.only_in == OnlyIn::Guild {
                &help_options.guild_only_text
            } else {
                &help_options.dm_and_guild_text
            };

            similar_commands
                .sort_unstable_by(|a, b| a.levenshtein_distance.cmp(&b.levenshtein_distance));

            let check_names: Vec<String> = command
                .options
                .checks
                .iter()
                .chain(group.options.checks.iter())
                .filter_map(|check| {
                    if check.display_in_help {
                        Some(check.name.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            return Ok(CustomisedHelpData::SingleCommand {
                command: Command {
                    name: options.names[0],
                    description: options.desc,
                    group_name: group.name,
                    group_prefixes: &group.options.prefixes,
                    checks: check_names,
                    aliases: options.names[1..].to_vec(),
                    availability: available_text,
                    usage: options.usage,
                    usage_sample: options.examples.to_vec(),
                },
            });
        }

        match nested_group_command_search(
            ctx,
            msg,
            &group.options.sub_groups,
            name,
            help_options,
            similar_commands,
            owners,
        ) {
            Ok(found) => return Ok(found),
            Err(()) => (),
        }

    }

    Err(())
}

/// Tries to extract a single command matching searched command name otherwise
/// returns similar commands.
#[cfg(feature = "cache")]
fn fetch_single_command<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    name: &str,
    help_options: &'a HelpOptions,
    owners: &HashSet<UserId>,
) -> Result<CustomisedHelpData<'a>, Vec<SuggestedCommandName>> {
    let mut similar_commands: Vec<SuggestedCommandName> = Vec::new();
    let mut name = name.to_string();

    match nested_group_command_search(
        ctx,
        msg,
        &groups,
        &mut name,
        &help_options,
        &mut similar_commands,
        &owners,
    ) {
        Ok(found) => Ok(found),
        Err(()) => Err(similar_commands),
    }
}

#[cfg(feature = "cache")]
#[allow(clippy::too_many_arguments)]
fn fill_eligible_commands<'a>(
    ctx: &Context,
    msg: &Message,
    commands: &[&'static InternalCommand],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
    group: &'a CommandGroup,
    to_fill: &mut GroupCommandsPair,
    highest_formatter: &mut HelpBehaviour,
) {
    to_fill.name = group.name;
    to_fill.prefixes = group.options.prefixes.to_vec();

    let group_behaviour = {
        if let HelpBehaviour::Hide = highest_formatter {
            HelpBehaviour::Hide
        } else {
            std::cmp::max(
                *highest_formatter,
                check_common_behaviour(
                    &ctx,
                    msg,
                    &group.options,
                    owners,
                    help_options,
                )
            )
        }
    };

    *highest_formatter = group_behaviour;

    for command in commands {
        let command = *command;
        let options = &command.options;
        let name = &options.names[0];

        match &group_behaviour {
            HelpBehaviour::Nothing => (),
            _ => {
                let name = format_command_name!(&group_behaviour, &name);
                to_fill.command_names.push(name);

                continue;
            }
        }

        let command_behaviour = check_command_behaviour(
            ctx,
            msg,
            &command.options,
            owners,
            help_options,
        );

        let name = format_command_name!(command_behaviour, &name);
        to_fill.command_names.push(name);
    }
}

/// Tries to fetch all commands visible to the user within a group and
/// its sub-groups.
#[cfg(feature = "cache")]
#[allow(clippy::too_many_arguments)]
fn fetch_all_eligible_commands_in_group<'a>(
    ctx: &Context,
    msg: &Message,
    commands: &[&'static InternalCommand],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
    group: &'a CommandGroup,
    highest_formatter: HelpBehaviour,
) -> GroupCommandsPair {
    let mut group_with_cmds = GroupCommandsPair::default();
    let mut highest_formatter = highest_formatter;

    fill_eligible_commands(
        ctx,
        msg,
        &commands,
        &owners,
        &help_options,
        &group,
        &mut group_with_cmds,
        &mut highest_formatter,
    );

    for sub_group in group.options.sub_groups {
        if HelpBehaviour::Hide == highest_formatter {
            break;
        } else if sub_group.options.commands.is_empty() && sub_group.options.sub_groups.is_empty() {
            continue;
        }

        let grouped_cmd = fetch_all_eligible_commands_in_group(
            ctx,
            msg,
            &sub_group.options.commands,
            &owners,
            &help_options,
            &sub_group,
            highest_formatter,
        );

        group_with_cmds.sub_groups.push(grouped_cmd);
    }

    group_with_cmds
}


/// Fetch groups with their commands.
#[cfg(feature = "cache")]
fn create_command_group_commands_pair_from_groups<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
) -> Vec<GroupCommandsPair> {
    let mut listed_groups: Vec<GroupCommandsPair> = Vec::default();

    for group in groups {
        let group = *group;

        let group_with_cmds = create_single_group(ctx, msg, group, &owners, &help_options);

        if !group_with_cmds.command_names.is_empty() || !group_with_cmds.sub_groups.is_empty() {
            listed_groups.push(group_with_cmds);
        }
    }

    listed_groups
}

/// Fetches a single group with its commands.
#[cfg(feature = "cache")]
fn create_single_group(
    ctx: &Context,
    msg: &Message,
    group: &CommandGroup,
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> GroupCommandsPair {
    let mut group_with_cmds = fetch_all_eligible_commands_in_group(
        ctx,
        &msg,
        &group.options.commands,
        &owners,
        &help_options,
        &group,
        HelpBehaviour::Nothing,
    );

    group_with_cmds.name = group.name;

    group_with_cmds
}

/// If `searched_group` is exact match on `group_name`,
/// this function returns `true` but does not trim.
/// Otherwise, it is treated as an optionally passed group-name and ends up
/// being removed from `searched_group`.
///
/// If a group has no prefixes, it is not required to be part of
/// `searched_group` to reach a sub-group of `group_name`.
#[cfg(feature = "cache")]
fn trim_prefixless_group(group_name: &str, searched_group: &mut String) -> bool {
    if group_name == searched_group.as_str() {
        return true;
    } else if starts_with_whole_word(&searched_group, &group_name) {
        searched_group.drain(..=group_name.len());
    }

    false
}

#[cfg(feature = "cache")]
#[allow(clippy::implicit_hasher)]
pub fn searched_lowercase<'a>(
    ctx: &Context,
    msg: &Message,
    args: &'a Args,
    group: &CommandGroup,
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
    searched_named_lowercase: &mut String,
) -> Option<CustomisedHelpData<'a>> {
    let is_prefixless_group = {
        group.options.prefixes.is_empty()
        && trim_prefixless_group(
            &group.name.to_lowercase(),
            searched_named_lowercase,
        )
    };
    let mut progressed = is_prefixless_group;
    let is_word_prefix = group
        .options
        .prefixes
        .iter()
        .any(|prefix| {
            if starts_with_whole_word(&searched_named_lowercase, &prefix) {
                searched_named_lowercase.drain(..=prefix.len());
                progressed = true;
            }

            prefix == searched_named_lowercase
        });

    if is_prefixless_group || is_word_prefix {
        let single_group =
            create_single_group(ctx, msg, &group, owners, &help_options);

        if !single_group.command_names.is_empty() {
            return Some(CustomisedHelpData::GroupedCommands {
                help_description: group
                    .options
                    .description
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                groups: vec![single_group],
            });
        }
    } else if progressed || group.options.prefixes.is_empty() {
        for sub_group in group.options.sub_groups {

            if let Some(found_set) = searched_lowercase(
                ctx,
                msg,
                args,
                sub_group,
                owners,
                help_options,
                searched_named_lowercase,
            ) {
                return Some(found_set);
            }
        }
    }

    None
}

/// Iterates over all commands and forges them into a `CustomisedHelpData`,
/// taking `HelpOptions` into consideration when deciding on whether a command
/// shall be picked and in what textual format.
#[cfg(feature = "cache")]
#[allow(clippy::implicit_hasher)]
pub fn create_customised_help_data<'a>(
    ctx: &Context,
    msg: &Message,
    args: &'a Args,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
) -> CustomisedHelpData<'a> {
    if !args.is_empty() {
        let name = args.message();

        return match fetch_single_command(ctx, msg, &groups, &name, &help_options, owners) {
            Ok(single_command) => single_command,
            Err(suggestions) => {
                let mut searched_named_lowercase = name.to_lowercase();

                for group in groups {

                    if let Some(found_command) = searched_lowercase(
                        ctx,
                        msg,
                        args,
                        group,
                        owners,
                        help_options,
                        &mut searched_named_lowercase,
                    ) {
                        return found_command;
                    }
                }

                if suggestions.is_empty() {
                    CustomisedHelpData::NoCommandFound {
                        help_error_message: &help_options.no_help_available_text,
                    }
                } else {
                    CustomisedHelpData::SuggestedCommands {
                        help_description: help_options.suggestion_text.to_string(),
                        suggestions: Suggestions(suggestions),
                    }
                }
            }
        };
    }

    let strikethrough_command_tip = if msg.is_private() {
        &help_options.strikethrough_commands_tip_in_guild
    } else {
        &help_options.strikethrough_commands_tip_in_dm
    };

    let description = if let Some(ref strikethrough_command_text) = strikethrough_command_tip {
        format!(
            "{}\n{}",
            &help_options.individual_command_tip, &strikethrough_command_text
        )
    } else {
        help_options.individual_command_tip.to_string()
    };

    let listed_groups = create_command_group_commands_pair_from_groups(
        ctx,
        msg,
        &groups,
        owners,
        &help_options,
    );

    if listed_groups.is_empty() {
        CustomisedHelpData::NoCommandFound {
            help_error_message: &help_options.no_help_available_text,
        }
    } else {
        CustomisedHelpData::GroupedCommands {
            help_description: description,
            groups: listed_groups,
        }
    }
}

/// Flattens a group with all its nested sub-groups into the passed `group_text`
/// buffer.
/// If `nest_level` is `0`, this function will skip the group's name.
#[cfg(all(feature = "cache", feature = "http"))]
fn flatten_group_to_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
) {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        let _ = writeln!(group_text,
            "{}__**{}**__",
            repeated_indent_str,
            group.name,
        );
    }

    if !group.prefixes.is_empty() {
        let _ = writeln!(group_text,
            "{}{}: `{}`",
            &repeated_indent_str,
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        );
    };

    let mut joined_commands = group
        .command_names
        .join(&format!("\n{}", &repeated_indent_str));


    if !group.command_names.is_empty() {
        joined_commands.insert_str(0, &repeated_indent_str);
    }

    let _ = writeln!(group_text, "{}", joined_commands);

    for sub_group in &group.sub_groups {

        if !(sub_group.command_names.is_empty() && sub_group.sub_groups.is_empty()) {
            let mut sub_group_text = String::default();

            flatten_group_to_string(
                &mut sub_group_text,
                &sub_group,
                nest_level + 1,
                &help_options,
            );

            let _ = write!(group_text, "{}", sub_group_text);
        }
    }
}

/// Flattens a group with all its nested sub-groups into the passed `group_text`
/// buffer respecting the plain help format.
/// If `nest_level` is `0`, this function will skip the group's name.
#[cfg(all(feature = "cache", feature = "http"))]
fn flatten_group_to_plain_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
) {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        let _ = write!(group_text,
            "\n{}**{}**",
            repeated_indent_str,
            group.name,
        );
    }

    if group.prefixes.is_empty() {
        let _ = write!(group_text, ": ");
    } else {
        let _ = write!(group_text,
            " ({}: `{}`): ",
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        );
    }

    let joined_commands = group.command_names.join(", ");

    let _ = write!(group_text, "{}", joined_commands);

    for sub_group in &group.sub_groups {
        let mut sub_group_text = String::default();

        flatten_group_to_plain_string(
            &mut sub_group_text,
            &sub_group,
            nest_level + 1,
            &help_options,
        );

        let _ = write!(group_text, "{}", sub_group_text);
    }
}


/// Sends an embed listing all groups with their commands.
#[cfg(all(feature = "cache", feature = "http"))]
fn send_grouped_commands_embed(
    http: impl AsRef<Http>,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    help_description: &str,
    groups: &[GroupCommandsPair],
    colour: Colour,
) -> Result<Message, Error> {
    channel_id.send_message(&http, |m| {
        m.embed(|embed| {
            embed.colour(colour);
            embed.description(help_description);

            for group in groups {
                let mut embed_text = String::default();

                flatten_group_to_string(
                    &mut embed_text,
                    &group,
                    0,
                    &help_options,
                );

                embed.field(group.name, &embed_text, true);
            }

            embed
        });
        m
    })
}

/// Sends embed showcasing information about a single command.
#[cfg(all(feature = "cache", feature = "http"))]
fn send_single_command_embed(
    http: impl AsRef<Http>,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    command: &Command<'_>,
    colour: Colour,
) -> Result<Message, Error> {
    channel_id.send_message(&http, |m| {
        m.embed(|embed| {
            embed.title(&command.name);
            embed.colour(colour);

            if let Some(ref desc) = command.description {
                embed.description(desc);
            }

            if let Some(ref usage) = command.usage {
                let full_usage_text = if let Some(first_prefix) = command.group_prefixes.get(0) {
                    format!("`{} {} {}`", first_prefix, command.name, usage)
                } else {
                    format!("`{} {}`", command.name, usage)
                };

                embed.field(&help_options.usage_label, full_usage_text, true);
            }

            if !command.usage_sample.is_empty() {
                let full_example_text =
                    if let Some(first_prefix) = command.group_prefixes.get(0) {
                        let format_example = |example| {
                            format!("`{} {} {}`\n", first_prefix, command.name, example)
                        };
                        command
                           .usage_sample
                           .iter()
                           .map(format_example)
                           .collect::<String>()
                    } else {
                        let format_example = |example| format!("`{} {}`\n", command.name, example);
                        command
                           .usage_sample
                           .iter()
                           .map(format_example)
                           .collect::<String>()
                    };
                embed.field(&help_options.usage_sample_label, full_example_text, true);
            }

            embed.field(&help_options.grouped_label, command.group_name, true);

            if !command.aliases.is_empty() {
                embed.field(
                    &help_options.aliases_label,
                    format!("`{}`", command.aliases.join("`, `")),
                    true,
                );
            }

            embed.field(&help_options.available_text, &command.availability, true);

            if !command.checks.is_empty() {
                embed.field(
                    &help_options.checks_label,
                    format!("`{}`", command.checks.join("`, `")),
                    true,
                );
            }

            embed
        });
        m
    })
}

/// Sends embed listing commands that are similar to the sent one.
#[cfg(all(feature = "cache", feature = "http"))]
fn send_suggestion_embed(
    http: impl AsRef<Http>,
    channel_id: ChannelId,
    help_description: &str,
    suggestions: &Suggestions,
    colour: Colour,
) -> Result<Message, Error> {
    let text = help_description
        .replace("{}", &suggestions.join("`, `"))
        .to_string();

    channel_id.send_message(&http, |m| {
        m.embed(|e| {
            e.colour(colour);
            e.description(text);
            e
        });
        m
    })
}

/// Sends an embed explaining fetching commands failed.
#[cfg(all(feature = "cache", feature = "http"))]
fn send_error_embed(
    http: impl AsRef<Http>,
    channel_id: ChannelId,
    input: &str,
    colour: Colour,
) -> Result<Message, Error> {
    channel_id.send_message(&http, |m| {
        m.embed(|e| {
            e.colour(colour);
            e.description(input);
            e
        });
        m
    })
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
/// use std::{collections::HashSet, hash::BuildHasher};
/// use serenity::{framework::standard::{Args, CommandGroup, CommandResult,
///     StandardFramework, macros::help, HelpOptions,
///     help_commands::*}, model::prelude::*,
/// };
///
/// #[help]
/// fn my_help(
///     context: &Context,
///     msg: &Message,
///     args: Args,
///     help_options: &'static HelpOptions,
///     groups: &[&'static CommandGroup],
///     owners: HashSet<UserId>
/// ) -> CommandResult {
///     let _ = with_embeds(context, msg, args, &help_options, groups, owners);
///     Ok(())
/// }
///
/// client.with_framework(StandardFramework::new()
///     .help(&MY_HELP));
/// ```
#[cfg(all(feature = "cache", feature = "http"))]
#[allow(clippy::implicit_hasher)]
pub fn with_embeds(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> Option<Message> {
    let formatted_help =
        create_customised_help_data(ctx, msg, &args, &groups, &owners, help_options);

    let response_result = match formatted_help {
        CustomisedHelpData::SuggestedCommands {
            ref help_description,
            ref suggestions,
        } => send_suggestion_embed(
            &ctx.http,
            msg.channel_id,
            &help_description,
            &suggestions,
            help_options.embed_error_colour,
        ),
        CustomisedHelpData::NoCommandFound {
            ref help_error_message,
        } => send_error_embed(
            &ctx.http,
            msg.channel_id,
            help_error_message,
            help_options.embed_error_colour,
        ),
        CustomisedHelpData::GroupedCommands {
            ref help_description,
            ref groups,
        } => send_grouped_commands_embed(
            &ctx.http,
            &help_options,
            msg.channel_id,
            &help_description,
            &groups,
            help_options.embed_success_colour,
        ),
        CustomisedHelpData::SingleCommand { ref command } => send_single_command_embed(
            &ctx.http,
            &help_options,
            msg.channel_id,
            &command,
            help_options.embed_success_colour,
        ),
        CustomisedHelpData::__Nonexhaustive => unreachable!(),
    };

    match response_result {
        Ok(response) => Some(response),
        Err(why) => {
            warn_about_failed_send!(&formatted_help, why);
            None
        },
    }
}

/// Turns grouped commands into a `String` taking plain help format into account.
#[cfg(all(feature = "cache", feature = "http"))]
fn grouped_commands_to_plain_string(
    help_options: &HelpOptions,
    help_description: &str,
    groups: &[GroupCommandsPair],
) -> String {
    let mut result = "__**Commands**__\n".to_string();
    let _ = writeln!(result, "{}", &help_description);

    for group in groups {
        let _ = write!(result, "\n**{}**", &group.name);

        flatten_group_to_plain_string(
            &mut result,
            &group,
            0,
            &help_options,
        );
    }

    result
}

/// Turns a single command into a `String` taking plain help format into account.
#[cfg(all(feature = "cache", feature = "http"))]
fn single_command_to_plain_string(help_options: &HelpOptions, command: &Command<'_>) -> String {
    let mut result = String::default();
    let _ = writeln!(result, "__**{}**__", command.name);

    if !command.aliases.is_empty() {
        let _ = writeln!(
            result,
            "**{}**: `{}`",
            help_options.aliases_label,
            command.aliases.join("`, `")
        );
    }

    if let Some(ref description) = command.description {
        let _ = writeln!(
            result,
            "**{}**: {}",
            help_options.description_label, description
        );
    };

    if let Some(ref usage) = command.usage {
        if let Some(first_prefix) = command.group_prefixes.get(0) {
            let _ = writeln!(
                result,
                "**{}**: `{} {} {}`",
                help_options.usage_label, first_prefix, command.name, usage
            );
        } else {
            let _ = writeln!(
                result,
                "**{}**: `{} {}`",
                help_options.usage_label, command.name, usage
            );
        }
    }

    if !command.usage_sample.is_empty() {
        if let Some(first_prefix) = command.group_prefixes.get(0) {
            let format_example = |example| {
                let _ = writeln!(
                    result,
                    "**{}**: `{} {} {}`",
                    help_options.usage_sample_label, first_prefix, command.name, example
                );
            };
            command
                .usage_sample
                .iter()
                .for_each(format_example);
        } else {
            let format_example = |example| {
                let _ = writeln!(
                    result,
                    "**{}**: `{} {}`",
                    help_options.usage_sample_label, command.name, example
                );
            };
            command
                .usage_sample
                .iter()
                .for_each(format_example);
        }
    }

    let _ = writeln!(
        result,
        "**{}**: {}",
        help_options.grouped_label, command.group_name
    );
    let _ = writeln!(
        result,
        "**{}**: {}",
        help_options.available_text, command.availability
    );

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
/// use std::{collections::HashSet, hash::BuildHasher};
/// use serenity::{framework::standard::{Args, CommandGroup, CommandResult,
///     StandardFramework, macros::help, HelpOptions,
///     help_commands::*}, model::prelude::*,
/// };
///
/// #[help]
/// fn my_help(
///     context: &Context,
///     msg: &Message,
///     args: Args,
///     help_options: &'static HelpOptions,
///     groups: &[&'static CommandGroup],
///     owners: HashSet<UserId>
/// ) -> CommandResult {
///     let _ = plain(context, msg, args, &help_options, groups, owners);
///     Ok(())
/// }
///
/// client.with_framework(StandardFramework::new()
///     .help(&MY_HELP));
/// ```
#[cfg(all(feature = "cache", feature = "http"))]
#[allow(clippy::implicit_hasher)]
pub fn plain(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> Option<Message> {
    let formatted_help =
        create_customised_help_data(ctx, msg, &args, &groups, &owners, help_options);

    let result = match formatted_help {
        CustomisedHelpData::SuggestedCommands {
            ref help_description,
            ref suggestions,
        } => help_description.replace("{}", &suggestions.join("`, `")),
        CustomisedHelpData::NoCommandFound {
            ref help_error_message,
        } => help_error_message.to_string(),
        CustomisedHelpData::GroupedCommands {
            ref help_description,
            ref groups,
        } => grouped_commands_to_plain_string(&help_options, &help_description, &groups),
        CustomisedHelpData::SingleCommand { ref command } => {
            single_command_to_plain_string(&help_options, &command)
        },
        CustomisedHelpData::__Nonexhaustive => unreachable!(),
    };

    match msg.channel_id.say(&ctx, result) {
        Ok(response) => Some(response),
        Err(why) => {
            warn_about_failed_send!(&formatted_help, why);
            None
        }
    }
}

#[cfg(test)]
#[cfg(all(feature = "cache", feature = "http"))]
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
#[cfg(all(feature = "cache", feature = "http"))]
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
