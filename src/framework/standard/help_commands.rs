//! A collection of default help commands for the framework.
//!
//! # Example
//!
//! Using the [`with_embeds`] function to have the framework's help message use embeds:
//!
//! ```rust,no_run
//! use std::collections::HashSet;
//! use std::env;
//!
//! use serenity::client::{Client, Context, EventHandler};
//! use serenity::framework::standard::macros::help;
//! use serenity::framework::standard::{
//!     help_commands,
//!     Args,
//!     CommandGroup,
//!     CommandResult,
//!     HelpOptions,
//!     StandardFramework,
//! };
//! use serenity::model::prelude::{Message, UserId};
//!
//! struct Handler;
//!
//! impl EventHandler for Handler {}
//!
//! #[help]
//! async fn my_help(
//!     context: &Context,
//!     msg: &Message,
//!     args: Args,
//!     help_options: &'static HelpOptions,
//!     groups: &[&'static CommandGroup],
//!     owners: HashSet<UserId>,
//! ) -> CommandResult {
//! #  #[cfg(all(feature = "cache", feature = "http"))]
//! # {
//!     let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
//!     Ok(())
//! # }
//! #
//! # #[cfg(not(all(feature = "cache", feature = "http")))]
//! # Ok(())
//! }
//!
//! let framework = StandardFramework::new().help(&MY_HELP);
//! ```
//!
//! The same can be accomplished with no embeds by substituting `with_embeds` with the [`plain`]
//! function.

#[cfg(all(feature = "cache", feature = "http"))]
use std::{collections::HashSet, fmt::Write};

#[cfg(all(feature = "cache", feature = "http"))]
use futures::future::{BoxFuture, FutureExt};
#[cfg(all(feature = "cache", feature = "http"))]
use levenshtein::levenshtein;
#[cfg(all(feature = "cache", feature = "http"))]
use tracing::warn;

#[cfg(all(feature = "cache", feature = "http"))]
use super::structures::Command as InternalCommand;
#[cfg(all(feature = "cache", feature = "http"))]
use super::{
    has_correct_permissions,
    has_correct_roles,
    Args,
    Check,
    CommandGroup,
    CommandOptions,
    HelpBehaviour,
    HelpOptions,
    OnlyIn,
};
#[cfg(all(feature = "cache", feature = "http"))]
use crate::{
    builder::{CreateEmbed, CreateMessage},
    cache::Cache,
    client::Context,
    framework::standard::CommonOptions,
    http::CacheHttp,
    model::channel::Message,
    model::id::{ChannelId, UserId},
    model::Colour,
    Error,
};

/// Macro to format a command according to a [`HelpBehaviour`] or continue to the next command-name
/// upon hiding.
#[cfg(all(feature = "cache", feature = "http"))]
macro_rules! format_command_name {
    ($behaviour:expr, $command_name:expr) => {
        match $behaviour {
            HelpBehaviour::Strike => format!("~~`{}`~~", $command_name),
            HelpBehaviour::Nothing => format!("`{}`", $command_name),
            HelpBehaviour::Hide => continue,
        }
    };
}

/// A single group containing its name and all related commands that are eligible in relation of
/// help-settings measured to the user.
#[derive(Clone, Debug, Default)]
pub struct GroupCommandsPair {
    pub name: &'static str,
    pub prefixes: Vec<&'static str>,
    pub command_names: Vec<String>,
    pub summary: Option<&'static str>,
    pub sub_groups: Vec<GroupCommandsPair>,
}

/// A single suggested command containing its name and Levenshtein distance to the actual user's
/// searched command name.
#[derive(Clone, Debug, Default)]
pub struct SuggestedCommandName {
    pub name: String,
    pub levenshtein_distance: usize,
}

/// A single command containing all related pieces of information.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Command<'a> {
    pub name: &'static str,
    pub group_name: &'static str,
    pub group_prefixes: &'a [&'static str],
    pub sub_commands: Vec<String>,
    pub aliases: Vec<&'static str>,
    pub availability: &'a str,
    pub description: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub usage_sample: Vec<&'static str>,
    pub checks: Vec<String>,
}

/// Contains possible suggestions in case a command could not be found but are similar enough.
#[derive(Clone, Debug, Default)]
pub struct Suggestions(pub Vec<SuggestedCommandName>);

#[cfg(all(feature = "cache", feature = "http"))]
impl Suggestions {
    /// Immutably borrow inner [`Vec`].
    #[inline]
    #[must_use]
    pub fn as_vec(&self) -> &Vec<SuggestedCommandName> {
        &self.0
    }

    /// Concats names of suggestions with a given `separator`.
    #[must_use]
    pub fn join(&self, separator: &str) -> String {
        match self.as_vec().as_slice() {
            [] => String::new(),
            [one] => one.name.clone(),
            [first, rest @ ..] => {
                let size = first.name.len() + rest.iter().map(|e| e.name.len()).sum::<usize>();
                let sep_size = rest.len() * separator.len();

                let mut joined = String::with_capacity(size + sep_size);
                joined.push_str(&first.name);
                for e in rest {
                    joined.push_str(separator);
                    joined.push_str(&e.name);
                }
                joined
            },
        }
    }
}

/// Covers possible outcomes of a help-request and yields relevant data in customised textual
/// representation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CustomisedHelpData<'a> {
    /// To display suggested commands.
    SuggestedCommands { help_description: String, suggestions: Suggestions },
    /// To display groups and their commands by name.
    GroupedCommands { help_description: String, groups: Vec<GroupCommandsPair> },
    /// To display one specific command.
    SingleCommand { command: Command<'a> },
    /// To display failure in finding a fitting command.
    NoCommandFound { help_error_message: &'a str },
}

/// Checks whether a user is member of required roles and given the required permissions.
#[cfg(feature = "cache")]
pub fn has_all_requirements(cache: impl AsRef<Cache>, cmd: &CommandOptions, msg: &Message) -> bool {
    let cache = cache.as_ref();

    if let Some(guild_id) = msg.guild_id {
        if let Some(member) = cache.member(guild_id, msg.author.id) {
            if let Ok(permissions) = member.permissions(cache) {
                return if cmd.allowed_roles.is_empty() {
                    permissions.administrator() || has_correct_permissions(cache, &cmd, msg)
                } else if let Some(roles) = cache.guild_roles(guild_id) {
                    permissions.administrator()
                        || (has_correct_roles(&cmd, &roles, &member)
                            && has_correct_permissions(cache, &cmd, msg))
                } else {
                    warn!("Failed to find the guild and its roles.");

                    false
                };
            }
        }
    }

    cmd.only_in != OnlyIn::Guild
}

/// Checks if `search_on` starts with `word` and is then cleanly followed by a `" "`.
#[inline]
#[cfg(all(feature = "cache", feature = "http"))]
fn starts_with_whole_word(search_on: &str, word: &str) -> bool {
    search_on.starts_with(word)
        && search_on.get(word.len()..=word.len()).is_some_and(|slice| slice == " ")
}

// Decides how a listed help entry shall be displayed.
#[cfg(all(feature = "cache", feature = "http"))]
fn check_common_behaviour(
    cache: impl AsRef<Cache>,
    msg: &Message,
    options: &impl CommonOptions,
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    if !options.help_available() {
        return HelpBehaviour::Hide;
    }

    if options.only_in() == OnlyIn::Dm && !msg.is_private()
        || options.only_in() == OnlyIn::Guild && msg.is_private()
    {
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

    msg.guild(cache.as_ref())
        .and_then(|guild| {
            if let Some(member) = guild.members.get(&msg.author.id) {
                if !has_correct_roles(options, &guild.roles, member) {
                    return Some(help_options.lacking_role);
                }
            }

            None
        })
        .unwrap_or(HelpBehaviour::Nothing)
}

#[cfg(all(feature = "cache", feature = "http"))]
async fn check_command_behaviour(
    ctx: &Context,
    msg: &Message,
    options: &CommandOptions,
    group_checks: &[&Check],
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    let behaviour = check_common_behaviour(ctx, msg, &options, owners, help_options);

    if behaviour == HelpBehaviour::Nothing
        && (!options.owner_privilege || !owners.contains(&msg.author.id))
    {
        for check in group_checks.iter().chain(options.checks) {
            if !check.check_in_help {
                continue;
            }

            let mut args = Args::new("", &[]);

            if (check.function)(ctx, msg, &mut args, options).await.is_err() {
                return help_options.lacking_conditions;
            }
        }
    }

    behaviour
}

// This function will recursively go through all commands and their sub-commands, trying to find
// `name`. Similar commands will be collected into `similar_commands`.
#[cfg(all(feature = "cache", feature = "http"))]
#[allow(clippy::too_many_arguments)]
fn nested_commands_search<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    group: &'rec CommandGroup,
    commands: &'rec [&'static InternalCommand],
    name: &'rec mut String,
    help_options: &'a HelpOptions,
    similar_commands: &'rec mut Vec<SuggestedCommandName>,
    owners: &'rec HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
) -> BoxFuture<'rec, Option<&'a InternalCommand>> {
    async move {
        for command in commands {
            let mut command = *command;

            let search_command_name_matched = {
                let mut command_found = None;

                for command_name in command.options.names {
                    if name == *command_name {
                        command_found = Some(*command_name);

                        break;
                    }
                }

                if command_found.is_some() {
                    command_found
                } else {
                    // Since the command could not be found in the group, we now will identify if
                    // the command is actually using a sub-command. We iterate all command names
                    // and check if one matches, if it does, we potentially have a sub-command.
                    for command_name in command.options.names {
                        if starts_with_whole_word(name, command_name) {
                            name.drain(..=command_name.len());
                            break;
                        }

                        if help_options.max_levenshtein_distance > 0 {
                            let levenshtein_distance = levenshtein(command_name, name);

                            if levenshtein_distance <= help_options.max_levenshtein_distance
                                && HelpBehaviour::Nothing
                                    == check_command_behaviour(
                                        ctx,
                                        msg,
                                        command.options,
                                        group.options.checks,
                                        owners,
                                        help_options,
                                    )
                                    .await
                            {
                                similar_commands.push(SuggestedCommandName {
                                    name: (*command_name).to_string(),
                                    levenshtein_distance,
                                });
                            }
                        }
                    }

                    // We check all sub-command names in order to see if one variant has been
                    // issued to the help-system.
                    let name_str = name.as_str();
                    let sub_command_found = command
                        .options
                        .sub_commands
                        .iter()
                        .find(|n| n.options.names.contains(&name_str))
                        .copied();

                    // If we found a sub-command, we replace the parent with it. This allows the
                    // help-system to extract information from the sub-command.
                    if let Some(sub_command) = sub_command_found {
                        // Check parent command's behaviour and permission first before we consider
                        // the sub-command overwrite it.
                        if HelpBehaviour::Nothing
                            == check_command_behaviour(
                                ctx,
                                msg,
                                command.options,
                                group.options.checks,
                                owners,
                                help_options,
                            )
                            .await
                        {
                            command = sub_command;
                            Some(sub_command.options.names[0])
                        } else {
                            break;
                        }
                    } else {
                        match nested_commands_search(
                            ctx,
                            msg,
                            group,
                            command.options.sub_commands,
                            name,
                            help_options,
                            similar_commands,
                            owners,
                        )
                        .await
                        {
                            Some(found) => return Some(found),
                            None => None,
                        }
                    }
                }
            };

            if search_command_name_matched.is_some() {
                if HelpBehaviour::Nothing
                    == check_command_behaviour(
                        ctx,
                        msg,
                        command.options,
                        group.options.checks,
                        owners,
                        help_options,
                    )
                    .await
                {
                    return Some(command);
                }
                break;
            }
        }

        None
    }
    .boxed()
}

// This function will recursively go through all groups and their groups, trying to find `name`.
// Similar commands will be collected into `similar_commands`.
#[cfg(all(feature = "cache", feature = "http"))]
fn nested_group_command_search<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    groups: &'rec [&'static CommandGroup],
    name: &'rec mut String,
    help_options: &'a HelpOptions,
    similar_commands: &'rec mut Vec<SuggestedCommandName>,
    owners: &'rec HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
) -> BoxFuture<'rec, Result<CustomisedHelpData<'a>, ()>> {
    async move {
        for group in groups {
            let group = *group;
            let group_behaviour =
                check_common_behaviour(ctx, msg, &group.options, owners, help_options);

            match &group_behaviour {
                HelpBehaviour::Nothing => (),
                _ => {
                    continue;
                },
            }

            if !group.options.prefixes.is_empty()
                && !group.options.prefixes.iter().any(|prefix| trim_prefixless_group(prefix, name))
            {
                continue;
            }

            let found = nested_commands_search(
                ctx,
                msg,
                group,
                group.options.commands,
                name,
                help_options,
                similar_commands,
                owners,
            )
            .await;

            if let Some(command) = found {
                let options = &command.options;

                if !options.help_available {
                    return Ok(CustomisedHelpData::NoCommandFound {
                        help_error_message: help_options.no_help_available_text,
                    });
                }

                let is_only = |only| group.options.only_in == only || options.only_in == only;

                let available_text = if is_only(OnlyIn::Dm) {
                    &help_options.dm_only_text
                } else if is_only(OnlyIn::Guild) {
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
                    .filter(|check| check.display_in_help)
                    .map(|check| check.name.to_string())
                    .collect();

                let sub_command_names: Vec<String> = options
                    .sub_commands
                    .iter()
                    .filter(|cmd| cmd.options.help_available)
                    .map(|cmd| cmd.options.names[0].to_string())
                    .collect();

                return Ok(CustomisedHelpData::SingleCommand {
                    command: Command {
                        name: options.names[0],
                        description: options.desc,
                        group_name: group.name,
                        group_prefixes: group.options.prefixes,
                        checks: check_names,
                        aliases: options.names[1..].to_vec(),
                        availability: available_text,
                        usage: options.usage,
                        usage_sample: options.examples.to_vec(),
                        sub_commands: sub_command_names,
                    },
                });
            }

            if let Ok(found) = nested_group_command_search(
                ctx,
                msg,
                group.options.sub_groups,
                name,
                help_options,
                similar_commands,
                owners,
            )
            .await
            {
                return Ok(found);
            }
        }

        Err(())
    }
    .boxed()
}

/// Tries to extract a single command matching searched command name otherwise returns similar
/// commands.
#[cfg(feature = "cache")]
async fn fetch_single_command<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    name: &'a str,
    help_options: &'a HelpOptions,
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
) -> Result<CustomisedHelpData<'a>, Vec<SuggestedCommandName>> {
    let mut similar_commands: Vec<SuggestedCommandName> = Vec::new();
    let mut name = name.to_string();

    nested_group_command_search(
        ctx,
        msg,
        groups,
        &mut name,
        help_options,
        &mut similar_commands,
        owners,
    )
    .await
    .map_err(|()| similar_commands)
}

#[cfg(feature = "cache")]
#[allow(clippy::too_many_arguments)]
async fn fill_eligible_commands<'a>(
    ctx: &Context,
    msg: &Message,
    commands: &[&'static InternalCommand],
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
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
                check_common_behaviour(ctx, msg, &group.options, owners, help_options),
            )
        }
    };

    *highest_formatter = group_behaviour;

    for command in commands {
        let command = *command;
        let options = &command.options;
        let name = &options.names[0];

        if group_behaviour != HelpBehaviour::Nothing {
            let name = format_command_name!(&group_behaviour, &name);
            to_fill.command_names.push(name);

            continue;
        }

        let command_behaviour = check_command_behaviour(
            ctx,
            msg,
            command.options,
            group.options.checks,
            owners,
            help_options,
        )
        .await;

        let name = format_command_name!(command_behaviour, &name);
        to_fill.command_names.push(name);
    }
}

/// Tries to fetch all commands visible to the user within a group and its sub-groups.
#[cfg(feature = "cache")]
#[allow(clippy::too_many_arguments)]
fn fetch_all_eligible_commands_in_group<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    commands: &'rec [&'static InternalCommand],
    owners: &'rec HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &'a HelpOptions,
    group: &'a CommandGroup,
    highest_formatter: HelpBehaviour,
) -> BoxFuture<'rec, GroupCommandsPair> {
    async move {
        let mut group_with_cmds = GroupCommandsPair::default();
        let mut highest_formatter = highest_formatter;

        fill_eligible_commands(
            ctx,
            msg,
            commands,
            owners,
            help_options,
            group,
            &mut group_with_cmds,
            &mut highest_formatter,
        )
        .await;

        for sub_group in group.options.sub_groups {
            if HelpBehaviour::Hide == highest_formatter {
                break;
            } else if sub_group.options.commands.is_empty()
                && sub_group.options.sub_groups.is_empty()
            {
                continue;
            }

            let grouped_cmd = fetch_all_eligible_commands_in_group(
                ctx,
                msg,
                sub_group.options.commands,
                owners,
                help_options,
                sub_group,
                highest_formatter,
            )
            .await;

            group_with_cmds.sub_groups.push(grouped_cmd);
        }

        group_with_cmds
    }
    .boxed()
}

/// Fetch groups with their commands.
#[cfg(feature = "cache")]
async fn create_command_group_commands_pair_from_groups(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &HelpOptions,
) -> Vec<GroupCommandsPair> {
    let mut listed_groups: Vec<GroupCommandsPair> = Vec::default();

    for group in groups {
        let group = *group;

        let group_with_cmds = create_single_group(ctx, msg, group, owners, help_options).await;

        if !group_with_cmds.command_names.is_empty() || !group_with_cmds.sub_groups.is_empty() {
            listed_groups.push(group_with_cmds);
        }
    }

    listed_groups
}

/// Fetches a single group with its commands.
#[cfg(feature = "cache")]
async fn create_single_group(
    ctx: &Context,
    msg: &Message,
    group: &CommandGroup,
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &HelpOptions,
) -> GroupCommandsPair {
    let mut group_with_cmds = fetch_all_eligible_commands_in_group(
        ctx,
        msg,
        group.options.commands,
        owners,
        help_options,
        group,
        HelpBehaviour::Nothing,
    )
    .await;

    group_with_cmds.name = group.name;
    group_with_cmds.summary = group.options.summary;

    group_with_cmds
}

/// If `searched_group` is exactly matching `group_name`, this function returns `true` but does not
/// trim. Otherwise, it is treated as an optionally passed group-name and ends up being removed
/// from `searched_group`.
///
/// If a group has no prefixes, it is not required to be part of `searched_group` to reach a
/// sub-group of `group_name`.
#[cfg(feature = "cache")]
fn trim_prefixless_group(group_name: &str, searched_group: &mut String) -> bool {
    if group_name == searched_group.as_str() {
        return true;
    } else if starts_with_whole_word(searched_group, group_name) {
        searched_group.drain(..=group_name.len());
        return true;
    }

    false
}

#[cfg(feature = "cache")]
pub fn searched_lowercase<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    group: &'rec CommandGroup,
    owners: &'rec HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &'a HelpOptions,
    searched_named_lowercase: &'rec mut String,
) -> BoxFuture<'rec, Option<CustomisedHelpData<'a>>> {
    async move {
        let is_prefixless_group = {
            group.options.prefixes.is_empty()
                && trim_prefixless_group(&group.name.to_lowercase(), searched_named_lowercase)
        };
        let mut progressed = is_prefixless_group;
        let is_word_prefix = group.options.prefixes.iter().any(|prefix| {
            if starts_with_whole_word(searched_named_lowercase, prefix) {
                searched_named_lowercase.drain(..=prefix.len());
                progressed = true;
            }

            prefix == searched_named_lowercase
        });

        if is_prefixless_group || is_word_prefix {
            let single_group = create_single_group(ctx, msg, group, owners, help_options).await;

            if !single_group.command_names.is_empty() {
                return Some(CustomisedHelpData::GroupedCommands {
                    help_description: group
                        .options
                        .description
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    groups: vec![single_group],
                });
            }
        } else if progressed || group.options.prefixes.is_empty() {
            for sub_group in group.options.sub_groups {
                if let Some(found_set) = searched_lowercase(
                    ctx,
                    msg,
                    sub_group,
                    owners,
                    help_options,
                    searched_named_lowercase,
                )
                .await
                {
                    return Some(found_set);
                }
            }
        }

        None
    }
    .boxed()
}

/// Iterates over all commands and forges them into a [`CustomisedHelpData`], taking
/// [`HelpOptions`] into consideration when deciding on whether a command shall be picked and in
/// what textual format.
#[cfg(feature = "cache")]
pub async fn create_customised_help_data<'a>(
    ctx: &Context,
    msg: &Message,
    args: &'a Args,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
    help_options: &'a HelpOptions,
) -> CustomisedHelpData<'a> {
    if !args.is_empty() {
        let name = args.message();

        return match fetch_single_command(ctx, msg, groups, name, help_options, owners).await {
            Ok(single_command) => single_command,
            Err(suggestions) => {
                let mut searched_named_lowercase = name.to_lowercase();

                for group in groups {
                    if let Some(found_command) = searched_lowercase(
                        ctx,
                        msg,
                        group,
                        owners,
                        help_options,
                        &mut searched_named_lowercase,
                    )
                    .await
                    {
                        return found_command;
                    }
                }

                if suggestions.is_empty() {
                    CustomisedHelpData::NoCommandFound {
                        help_error_message: help_options.no_help_available_text,
                    }
                } else {
                    CustomisedHelpData::SuggestedCommands {
                        help_description: help_options.suggestion_text.to_string(),
                        suggestions: Suggestions(suggestions),
                    }
                }
            },
        };
    }

    let strikethrough_command_tip = if msg.is_private() {
        help_options.strikethrough_commands_tip_in_dm
    } else {
        help_options.strikethrough_commands_tip_in_guild
    };

    let description = if let Some(strikethrough_command_text) = strikethrough_command_tip {
        format!("{}\n{strikethrough_command_text}", help_options.individual_command_tip)
    } else {
        help_options.individual_command_tip.to_string()
    };

    let listed_groups =
        create_command_group_commands_pair_from_groups(ctx, msg, groups, owners, help_options)
            .await;

    if listed_groups.is_empty() {
        CustomisedHelpData::NoCommandFound {
            help_error_message: help_options.no_help_available_text,
        }
    } else {
        CustomisedHelpData::GroupedCommands {
            help_description: description,
            groups: listed_groups,
        }
    }
}

/// Flattens a group with all its nested sub-groups into the passed `group_text` buffer. If
/// `nest_level` is `0`, this function will skip the group's name.
#[cfg(all(feature = "cache", feature = "http"))]
fn flatten_group_to_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
) -> Result<(), Error> {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        writeln!(group_text, "{repeated_indent_str}__**{}**__", group.name,)?;
    }

    let mut summary_or_prefixes = false;

    if let Some(group_summary) = group.summary {
        writeln!(group_text, "{}*{group_summary}*", &repeated_indent_str)?;
        summary_or_prefixes = true;
    }

    if !group.prefixes.is_empty() {
        writeln!(
            group_text,
            "{}{}: `{}`",
            &repeated_indent_str,
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        )?;
        summary_or_prefixes = true;
    };

    if summary_or_prefixes {
        writeln!(group_text)?;
    }

    let mut joined_commands = group.command_names.join(&format!("\n{repeated_indent_str}"));

    if !group.command_names.is_empty() {
        joined_commands.insert_str(0, &repeated_indent_str);
    }

    writeln!(group_text, "{joined_commands}")?;

    for sub_group in &group.sub_groups {
        if !(sub_group.command_names.is_empty() && sub_group.sub_groups.is_empty()) {
            let mut sub_group_text = String::default();

            flatten_group_to_string(&mut sub_group_text, sub_group, nest_level + 1, help_options)?;

            write!(group_text, "{sub_group_text}")?;
        }
    }

    Ok(())
}

/// Flattens a group with all its nested sub-groups into the passed `group_text` buffer respecting
/// the plain help format. If `nest_level` is `0`, this function will skip the group's name.
#[cfg(all(feature = "cache", feature = "http"))]
fn flatten_group_to_plain_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
) {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        write!(group_text, "\n{repeated_indent_str}**{}**", group.name).unwrap();
    }

    if group.prefixes.is_empty() {
        group_text.push_str(": ");
    } else {
        write!(
            group_text,
            " ({}: `{}`): ",
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        ).unwrap();
    }

    let joined_commands = group.command_names.join(", ");

    group_text.push_str(&joined_commands);

    for sub_group in &group.sub_groups {
        let mut sub_group_text = String::default();

        flatten_group_to_plain_string(&mut sub_group_text, sub_group, nest_level + 1, help_options);

        group_text.push_str(&sub_group_text);
    }
}

/// Sends an embed listing all groups with their commands.
#[cfg(all(feature = "cache", feature = "http"))]
async fn send_grouped_commands_embed(
    cache_http: impl CacheHttp,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    help_description: &str,
    groups: &[GroupCommandsPair],
    colour: Colour,
) -> Result<Message, Error> {
    // creating embed outside message builder since flatten_group_to_string may return an error.

    let mut embed = CreateEmbed::new().colour(colour).description(help_description);
    for group in groups {
        let mut embed_text = String::default();

        flatten_group_to_string(&mut embed_text, group, 0, help_options)?;

        embed = embed.field(group.name, embed_text, true);
    }

    let builder = CreateMessage::new().embed(embed);
    channel_id.send_message(cache_http, builder).await
}

/// Sends embed showcasing information about a single command.
#[cfg(all(feature = "cache", feature = "http"))]
async fn send_single_command_embed(
    cache_http: impl CacheHttp,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    command: &Command<'_>,
    colour: Colour,
) -> Result<Message, Error> {
    let mut embed = CreateEmbed::new().title(command.name).colour(colour);

    if let Some(desc) = command.description {
        embed = embed.description(desc);
    }

    if let Some(usage) = command.usage {
        let full_usage_text = if let Some(first_prefix) = command.group_prefixes.first() {
            format!("`{first_prefix} {} {usage}`", command.name)
        } else {
            format!("`{} {usage}`", command.name)
        };

        embed = embed.field(help_options.usage_label, full_usage_text, true);
    }

    if !command.usage_sample.is_empty() {
        let full_example_text = if let Some(first_prefix) = command.group_prefixes.first() {
            let format_example = |example| format!("`{first_prefix} {} {example}`\n", command.name);
            command.usage_sample.iter().map(format_example).collect::<String>()
        } else {
            let format_example = |example| format!("`{} {example}`\n", command.name);
            command.usage_sample.iter().map(format_example).collect::<String>()
        };
        embed = embed.field(help_options.usage_sample_label, full_example_text, true);
    }

    embed = embed.field(help_options.grouped_label, command.group_name, true);

    if !command.aliases.is_empty() {
        embed = embed.field(
            help_options.aliases_label,
            format!("`{}`", command.aliases.join("`, `")),
            true,
        );
    }

    if !help_options.available_text.is_empty() && !command.availability.is_empty() {
        embed = embed.field(help_options.available_text, command.availability, true);
    }

    if !command.checks.is_empty() {
        embed = embed.field(
            help_options.checks_label,
            format!("`{}`", command.checks.join("`, `")),
            true,
        );
    }

    if !command.sub_commands.is_empty() {
        embed = embed.field(
            help_options.sub_commands_label,
            format!("`{}`", command.sub_commands.join("`, `")),
            true,
        );
    }

    let builder = CreateMessage::new().embed(embed);
    channel_id.send_message(cache_http, builder).await
}

/// Sends embed listing commands that are similar to the sent one.
#[cfg(all(feature = "cache", feature = "http"))]
async fn send_suggestion_embed(
    cache_http: impl CacheHttp,
    channel_id: ChannelId,
    help_description: &str,
    suggestions: &Suggestions,
    colour: Colour,
) -> Result<Message, Error> {
    let text = help_description.replace("{}", &suggestions.join("`, `"));

    let embed = CreateEmbed::new().colour(colour).description(text);
    let builder = CreateMessage::new().embed(embed);
    channel_id.send_message(cache_http, builder).await
}

/// Sends an embed explaining fetching commands failed.
#[cfg(all(feature = "cache", feature = "http"))]
async fn send_error_embed(
    cache_http: impl CacheHttp,
    channel_id: ChannelId,
    input: &str,
    colour: Colour,
) -> Result<Message, Error> {
    let embed = CreateEmbed::new().colour(colour).description(input);
    let builder = CreateMessage::new().embed(embed);
    channel_id.send_message(cache_http, builder).await
}

/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with [`StandardFramework::help`]:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// use std::collections::HashSet;
/// use std::hash::BuildHasher;
///
/// use serenity::framework::standard::help_commands::*;
/// use serenity::framework::standard::macros::help;
/// use serenity::framework::standard::{
///     Args,
///     CommandGroup,
///     CommandResult,
///     HelpOptions,
///     StandardFramework,
/// };
/// use serenity::model::prelude::*;
///
/// #[help]
/// async fn my_help(
///     context: &Context,
///     msg: &Message,
///     args: Args,
///     help_options: &'static HelpOptions,
///     groups: &[&'static CommandGroup],
///     owners: HashSet<UserId>,
/// ) -> CommandResult {
///     let _ = with_embeds(context, msg, args, &help_options, groups, owners).await?;
///     Ok(())
/// }
///
/// let framework = StandardFramework::new().help(&MY_HELP);
/// ```
///
/// # Errors
///
/// Returns the same errors as [`ChannelId::send_message`].
///
/// [`StandardFramework::help`]: crate::framework::standard::StandardFramework::help
#[cfg(all(feature = "cache", feature = "http"))]
pub async fn with_embeds(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
) -> Result<Message, Error> {
    let formatted_help =
        create_customised_help_data(ctx, msg, &args, groups, &owners, help_options).await;

    match formatted_help {
        CustomisedHelpData::SuggestedCommands {
            ref help_description,
            ref suggestions,
        } => {
            send_suggestion_embed(
                &ctx.http,
                msg.channel_id,
                help_description,
                suggestions,
                help_options.embed_error_colour,
            )
            .await
        },
        CustomisedHelpData::NoCommandFound {
            help_error_message,
        } => {
            send_error_embed(
                &ctx.http,
                msg.channel_id,
                help_error_message,
                help_options.embed_error_colour,
            )
            .await
        },
        CustomisedHelpData::GroupedCommands {
            ref help_description,
            ref groups,
        } => {
            send_grouped_commands_embed(
                &ctx.http,
                help_options,
                msg.channel_id,
                help_description,
                groups,
                help_options.embed_success_colour,
            )
            .await
        },
        CustomisedHelpData::SingleCommand {
            ref command,
        } => {
            send_single_command_embed(
                &ctx.http,
                help_options,
                msg.channel_id,
                command,
                help_options.embed_success_colour,
            )
            .await
        },
    }
}

/// Turns grouped commands into a [`String`] taking plain help format into account.
#[cfg(all(feature = "cache", feature = "http"))]
fn grouped_commands_to_plain_string(
    help_options: &HelpOptions,
    help_description: &str,
    groups: &[GroupCommandsPair],
) -> String {
    let mut result = "__**Commands**__\n".to_string();

    result.push_str(help_description);
    result.push('\n');

    for group in groups {
        write!(result, "\n**{}**", &group.name).unwrap();

        flatten_group_to_plain_string(&mut result, group, 0, help_options);
    }

    result
}

/// Turns a single command into a [`String`] taking plain help format into account.
#[cfg(all(feature = "cache", feature = "http"))]
fn single_command_to_plain_string(help_options: &HelpOptions, command: &Command<'_>) -> String {
    let mut result = String::new();

    writeln!(result, "__**{}**__", command.name).unwrap();

    if !command.aliases.is_empty() {
        write!(result, "**{}**: `{}`", help_options.aliases_label, command.aliases.join("`, `"))
            .unwrap();
    }

    if let Some(description) = command.description {
        writeln!(result, "**{}**: {description}", help_options.description_label).unwrap();
    };

    if let Some(usage) = command.usage {
        if let Some(first_prefix) = command.group_prefixes.first() {
            writeln!(
                result,
                "**{}**: `{first_prefix} {} {usage}`",
                help_options.usage_label, command.name
            )
            .unwrap();
        } else {
            writeln!(result, "**{}**: `{} {usage}`", help_options.usage_label, command.name)
                .unwrap();
        }
    }

    if !command.usage_sample.is_empty() {
        if let Some(first_prefix) = command.group_prefixes.first() {
            let format_example = |example| {
                writeln!(
                    result,
                    "**{}**: `{first_prefix} {} {example}`",
                    help_options.usage_sample_label, command.name
                )
                .unwrap();
            };
            command.usage_sample.iter().for_each(format_example);
        } else {
            let format_example = |example| {
                writeln!(
                    result,
                    "**{}**: `{} {example}`",
                    help_options.usage_sample_label, command.name
                )
                .unwrap();
            };
            command.usage_sample.iter().for_each(format_example);
        }
    }

    writeln!(result, "**{}**: {}", help_options.grouped_label, command.group_name).unwrap();

    if !help_options.available_text.is_empty() && !command.availability.is_empty() {
        writeln!(result, "**{}**: {}", help_options.available_text, command.availability).unwrap();
    }

    if !command.sub_commands.is_empty() {
        writeln!(
            result,
            "**{}**: `{}`",
            help_options.sub_commands_label,
            command.sub_commands.join("`, `"),
        )
        .unwrap();
    }

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
/// use std::collections::HashSet;
/// use std::hash::BuildHasher;
///
/// use serenity::framework::standard::help_commands::*;
/// use serenity::framework::standard::macros::help;
/// use serenity::framework::standard::{
///     Args,
///     CommandGroup,
///     CommandResult,
///     HelpOptions,
///     StandardFramework,
/// };
/// use serenity::model::prelude::*;
///
/// #[help]
/// async fn my_help(
///     context: &Context,
///     msg: &Message,
///     args: Args,
///     help_options: &'static HelpOptions,
///     groups: &[&'static CommandGroup],
///     owners: HashSet<UserId>,
/// ) -> CommandResult {
///     let _ = plain(context, msg, args, &help_options, groups, owners).await?;
///     Ok(())
/// }
///
/// let framework = StandardFramework::new().help(&MY_HELP);
/// ```
/// # Errors
///
/// Returns the same errors as [`ChannelId::send_message`].
#[cfg(all(feature = "cache", feature = "http"))]
pub async fn plain(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId, impl std::hash::BuildHasher + Send + Sync>,
) -> Result<Message, Error> {
    let formatted_help =
        create_customised_help_data(ctx, msg, &args, groups, &owners, help_options).await;

    let result = match formatted_help {
        CustomisedHelpData::SuggestedCommands {
            ref help_description,
            ref suggestions,
        } => help_description.replace("{}", &suggestions.join("`, `")),
        CustomisedHelpData::NoCommandFound {
            help_error_message,
        } => help_error_message.to_string(),
        CustomisedHelpData::GroupedCommands {
            ref help_description,
            ref groups,
        } => grouped_commands_to_plain_string(help_options, help_description, groups),
        CustomisedHelpData::SingleCommand {
            ref command,
        } => single_command_to_plain_string(help_options, command),
    };

    msg.channel_id.say(&ctx, result).await
}

#[cfg(test)]
#[cfg(all(feature = "cache", feature = "http"))]
mod tests {
    use super::{SuggestedCommandName, Suggestions};

    #[test]
    fn suggestions_join() {
        let names = vec![
            SuggestedCommandName {
                name: "aa".to_owned(),
                levenshtein_distance: 0,
            },
            SuggestedCommandName {
                name: "bbb".to_owned(),
                levenshtein_distance: 0,
            },
            SuggestedCommandName {
                name: "cccc".to_owned(),
                levenshtein_distance: 0,
            },
        ];

        let actual = Suggestions(names).join(", ");

        assert_eq!(actual, "aa, bbb, cccc");
        assert_eq!(actual.capacity(), 13);
    }
}
