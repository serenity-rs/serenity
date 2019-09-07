#![deny(rust_2018_idioms)]
// FIXME: Remove this in a foreseeable future.
// Currently exists for backwards compatibility to previous Rust versions.
#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Lit, Token,
};

pub(crate) mod attributes;
pub(crate) mod consts;
pub(crate) mod structures;

#[macro_use]
pub(crate) mod util;

use attributes::*;
use consts::*;
use structures::*;
use util::*;

macro_rules! match_options {
    ($v:expr, $values:ident, $options:ident, $span:expr => [$($name:ident);*]) => {
        match $v {
            $(
                stringify!($name) => $options.$name = propagate_err!($crate::attributes::parse($values)),
            )*
            _ => {
                return Error::new($span, format_args!("invalid attribute: {:?}", $v))
                    .to_compile_error()
                    .into();
            },
        }
    };
}

/// The heart of the attribute-based framework.
///
/// This is a function attribute macro; if you attempt to use this on other Rust constructs, it won't work.
///
/// # Options
///
/// To alter how the framework will interpret the command,
/// you can provide options as attributes following this `#[command]` macro.
///
/// Each option has its own kind of data to stock and manipulate with.
/// They're given to the option either with the `#[option(...)]` or `#[option = ...]` syntaxes.
/// If an option doesn't require for any data to be supplied, then it's simply `#[option]`.
///
/// If the input to the option is malformed, the macro will give you can error, describing
/// the correct method for passing data, and what it should be.
///
/// The list of available options, is, as follows:
///
/// - `#[checks(idents)]`
/// Preconditions that must be met. Executed before the command's execution.
/// `idents` is a list of identifiers, seperated by a comma, referencing functions of the declaration:
/// `fn(&mut Context, &Message, &mut Args, &CommandOptions) -> serenity::framework::standard::CheckResult`
///
/// - `#[aliases(names)]`
/// A list of other names that can be used to execute this command.
/// In `serenity::framework::standard::CommandOptions`, these are put in the `names` field, right after the command's name.
///
/// - `#[description(desc)]`/`#[description = desc]`
/// A summary of the command.
///
/// - `#[usage(usg)]`/`#[usage = usg]`
/// Usage schema of the command.
///
/// - `#[example(ex)]`/`#[example = ex]`
/// Example of the command's usage.
///
/// - `#[min_args(min)]`, `#[max_args(max)]`, `#[num_args(min_and_max)]`
/// The minimum and/or maximum amount of arguments that the command should/can receive.
///
/// `num_args` is a helper attribute, serving as a shorthand for calling
/// `min_args` and `max_args` with the same amount of arguments.
///
/// - `#[required_permissions(perms)]`
/// A list of permissions that the user must have.
/// Refer to [Discord's offical documentation about available permissions](https://discordapp.com/developers/docs/topics/permissions).
///
/// - `#[allowed_roles(roles)]`
/// A list of strings (role names), seperated by a comma,
/// stating that only members of certain roles can execute this command.
///
/// - `#[help_available]`/`#[help_available(bool)]`
/// Whether this command should be displayed in the help message.
///
/// - `#[only_in(context)]`
/// Which context the command can only be executed in.
///
/// `context` can be of "guilds" or "dms" (direct messages).
///
/// - `#[bucket(name)]`/`#[bucket = name]`
/// What bucket should impact this command.
/// Refer to [the bucket example in the standard framework](https://docs.rs/serenity/*/serenity/framework/standard/struct.StandardFramework.html#method.bucket)
/// for its usage.
///
/// - `#[owners_only]`/`#[owners_only(bool)]`
/// Whether this command is exclusive to owners.
///
/// - `#[owner_privilege]`/`#[owner_privilege]`
/// Whether this command has a privilege for owners (i.e certain options are ignored for them).
///
/// - `#[sub_commands(commands)]`
/// A list of command names, separated by a comma, stating the subcommands of this command.
/// These are executed in the form: `this-command sub-command`
///
/// # Notes
/// The name of the command is parsed from the applied function,
/// or may be specified inside the `#[command]` attribute, a lá `#[command("foobar")]`.
///
/// This macro attribute generates static instances of `Command` and `CommandOptions`,
/// conserving the provided options.
///
/// The names of the instances are all uppercased names of the command name.
/// For example, with a name of "foo":
/// ```rust,ignore
/// pub static FOO_COMMAND_OPTIONS: CommandOptions = CommandOptions { ... };
/// pub static FOO_COMMAND: Command = Command { options: FOO_COMMAND_OPTIONS, ... };
/// ```
#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(input as CommandFun);

    let _name = if !attr.is_empty() {
        parse_macro_input!(attr as Lit).to_str()
    } else {
        fun.name.to_string()
    };

    let mut options = Options::new();

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = propagate_err!(parse_values(attribute));

        let name = values.name.to_string();
        let name = &name[..];

        match name {
            "num_args" => {
                let args = propagate_err!(u16::parse(values));

                options.min_args = AsOption(Some(args));
                options.max_args = AsOption(Some(args));
            }
            "example" => {
                options
                    .examples
                    .push(propagate_err!(attributes::parse(values)));
            }
            _ => {
                match_options!(name, values, options, span => [
                    checks;
                    bucket;
                    aliases;
                    description;
                    delimiters;
                    usage;
                    min_args;
                    max_args;
                    required_permissions;
                    allowed_roles;
                    help_available;
                    only_in;
                    owners_only;
                    owner_privilege;
                    sub_commands
                ]);
            }
        }
    }

    let Options {
        checks,
        bucket,
        aliases,
        description,
        delimiters,
        usage,
        examples,
        min_args,
        max_args,
        allowed_roles,
        required_permissions,
        help_available,
        only_in,
        owners_only,
        owner_privilege,
        sub_commands,
    } = options;

    propagate_err!(validate_declaration(&mut fun, DeclarFor::Command));

    let either = [
        parse_quote!(CommandResult),
        parse_quote!(serenity::framework::standard::CommandResult),
    ];

    propagate_err!(validate_return_type(&mut fun, either));

    let name = fun.name.clone();
    let options = name.with_suffix(COMMAND_OPTIONS);
    let sub_commands = sub_commands
        .into_iter()
        .map(|i| i.with_suffix(COMMAND))
        .collect::<Vec<_>>();

    let n = name.with_suffix(COMMAND);

    let cooked = fun.cooked.clone();
    let cooked2 = cooked.clone();

    let options_path = quote!(serenity::framework::standard::CommandOptions);
    let command_path = quote!(serenity::framework::standard::Command);

    (quote! {
        #(#cooked)*
        pub static #options: #options_path = #options_path {
            checks: #checks,
            bucket: #bucket,
            names: &[#_name, #(#aliases),*],
            desc: #description,
            delimiters: &[#(#delimiters),*],
            usage: #usage,
            examples: &[#(#examples),*],
            min_args: #min_args,
            max_args: #max_args,
            allowed_roles: &[#(#allowed_roles),*],
            required_permissions: #required_permissions,
            help_available: #help_available,
            only_in: #only_in,
            owners_only: #owners_only,
            owner_privilege: #owner_privilege,
            sub_commands: &[#(&#sub_commands),*],
        };

        #(#cooked2)*
        pub static #n: #command_path = #command_path {
            fun: #name,
            options: &#options,
        };

        #fun
    })
    .into()
}

/// A brother macro to [`command`], but for the help command.
/// An interface for simple browsing of all the available commands the bot provides,
/// and reading through specific information regarding a command.
///
/// As such, the options here will pertain in the help command's **layout** than its functionality.
///
/// # Options
///
/// - `#[suggestion_text(s)]`/`#[suggestion_text = s]`
/// For suggesting a command's name.
///
/// - `#[no_help_available_text(s)]`/`#[no_help_available_text = s]`
/// When help is unavailable for a command.
///
/// - `#[usage_label(s)]`/`#[usage_label = s]`
/// How should the command be used.
///
/// - `#[usage_sample_label(s)]`/`#[usage_sample_label = s]`
/// Actual sample label.
///
/// - `#[ungrouped_label(s)]`/`#[ungrouped_label = s]`
/// Ungrouped commands label.
///
/// - `#[description_label(s)]`/`#[description_label = s]`
/// Label at the start of the description.
///
/// - `#[grouped_label(s)]`/`#[grouped_label = s]`
/// Grouped commands label.
///
/// - `#[aliases_label(s)]`/`#[aliases_label = s]`
/// Label for a command's aliases.
///
/// - `#[guild_only_text(s)]`/`#[guild_only_text = s]`
/// When a command is specific to guilds only.
///
/// - `#[checks_label(s)]`/`#[checks_label = s]`
/// The header text when showing checks in the help command.
///
/// - `#[dm_only_text(s)]`/`#[dm_only_text = s]`
/// When a command is specific to dms only.
///
/// - `#[dm_and_guild_text(s)]`/`#[dm_guild_only_text = s]`
/// When a command is usable in both guilds and dms.
///
/// - `#[available_text(s)]`/`#[available_text = s]`
/// When a command is available.
///
/// - `#[command_not_found_text(s)]`/`#[command_not_found_text = s]`
/// When a command wasn't found.
///
/// - `#[individual_command_tip(s)]`/`#[individual_command_tip = s]`
/// How the user should access a command's details.
///
/// - `#[strikethrough_commands_tip_in_dm]`/`#[strikethrough_commands_tip_in_dm(s)]`/`#[strikethrough_commands_tip_in_dm = s]`
/// Reasoning behind strikethrough-commands.
///
/// If there wasn't any text passed, default text will be used instead.
///
/// *Only used in dms.*
///
/// - `#[strikethrough_commands_tip_in_guild]`/`#[strikethrough_commands_tip_in_guild(s)]`/`#[strikethrough_commands_tip_in_guild = s]`
/// Reasoning behind strikethrough-commands.
///
/// If there wasn't any text passed, default text will be used instead.
///
/// *Only used in guilds.*
///
/// - `#[group_prefix(s)]`/`#[group_prefix = s]`
/// For introducing a group's prefix
///
/// - `#[lacking_role(s)]`/`#[lacking_role = s]`
/// If a user lacks required roles, this will treat how commands will be displayed.
///
/// Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing` (leave be).
///
/// - `#[lacking_ownership(s)]`/`#[lacking_ownership = s]`
/// If a user lacks ownership, this will treat how these commands will be displayed.
///
/// Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing` (leave be).
///
/// - `#[lacking_permissions(s)]`/`#[lacking_role = s]`
/// If a user lacks permissions, this will treat how commands will be displayed.
///
/// Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing` (leave be).
///
/// - `#[embed_error_colour(n)]`
/// Colour that the help-embed will use upon an error.
///
/// Value is a name to one of the provided constants of the `Colour` struct.
///
///- `#[embed_success_colour(n)]`
/// Colour that the help-embed will use normally.
///
/// Value is a name to one of the provided constants of the `Colour` struct.
///
/// - `#[max_levenshtein_distance(n)]`
/// How much should the help command search for a similiar name.
///
/// Indicator for a nested guild. The prefix will be repeated based on what
/// kind of level the item sits. A sub-group would be level two, a sub-sub-group
/// would be level three.
/// - `#[indention_prefix = s]`
///
/// [`command`]: fn.command.html
#[proc_macro_attribute]
pub fn help(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(input as CommandFun);

    let names = if !attr.is_empty() {
        struct Names(Vec<String>);

        impl Parse for Names {
            fn parse(input: ParseStream<'_>) -> Result<Self> {
                let n: Punctuated<Lit, Token![,]> = input.parse_terminated(Lit::parse)?;
                Ok(Names(n.into_iter().map(|l| l.to_str()).collect()))
            }
        }
        let Names(names) = parse_macro_input!(attr as Names);

        names
    } else {
        vec!["help".to_string()]
    };

    let mut options = HelpOptions::default();

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = propagate_err!(parse_values(attribute));

        let name = values.name.to_string();
        let name = &name[..];

        match_options!(name, values, options, span => [
            suggestion_text;
            no_help_available_text;
            usage_label;
            usage_sample_label;
            ungrouped_label;
            grouped_label;
            aliases_label;
            description_label;
            guild_only_text;
            checks_label;
            dm_only_text;
            dm_and_guild_text;
            available_text;
            command_not_found_text;
            individual_command_tip;
            group_prefix;
            lacking_role;
            lacking_permissions;
            lacking_ownership;
            wrong_channel;
            embed_error_colour;
            embed_success_colour;
            strikethrough_commands_tip_in_dm;
            strikethrough_commands_tip_in_guild;
            max_levenshtein_distance;
            indention_prefix
        ]);
    }

    fn produce_strike_text(options: &HelpOptions, dm_or_guild: &str) -> Option<String> {
        use std::fmt::Write;

        let mut strike_text =
            String::from("~~`Strikethrough commands`~~ are unavailable because they");
        let mut is_any_option_strike = false;

        let mut concat_with_comma = if options.lacking_permissions == HelpBehaviour::Strike {
            is_any_option_strike = true;
            strike_text.push_str(" require permissions");

            true
        } else {
            false
        };

        if options.lacking_role == HelpBehaviour::Strike {
            is_any_option_strike = true;

            if concat_with_comma {
                strike_text.push_str(", require a specific role");
            } else {
                strike_text.push_str(" require a specific role");
                concat_with_comma = true;
            }
        }

        if options.wrong_channel == HelpBehaviour::Strike {
            is_any_option_strike = true;

            if concat_with_comma {
                let _ = write!(strike_text, ", or are limited to {}", dm_or_guild);
            } else {
                let _ = write!(strike_text, " are limited to {}", dm_or_guild);
            }
        }

        strike_text.push('.');

        if is_any_option_strike {
            Some(strike_text)
        } else {
            None
        }
    }

    if options.strikethrough_commands_tip_in_dm == Some(String::new()) {
        options.strikethrough_commands_tip_in_dm = produce_strike_text(&options, "direct messages");
    }

    if options.strikethrough_commands_tip_in_guild == Some(String::new()) {
        options.strikethrough_commands_tip_in_guild =
            produce_strike_text(&options, "guild messages");
    }

    let HelpOptions {
        suggestion_text,
        no_help_available_text,
        usage_label,
        usage_sample_label,
        ungrouped_label,
        grouped_label,
        aliases_label,
        description_label,
        guild_only_text,
        checks_label,
        dm_only_text,
        dm_and_guild_text,
        available_text,
        command_not_found_text,
        individual_command_tip,
        group_prefix,
        strikethrough_commands_tip_in_dm,
        strikethrough_commands_tip_in_guild,
        lacking_role,
        lacking_permissions,
        lacking_ownership,
        wrong_channel,
        embed_error_colour,
        embed_success_colour,
        max_levenshtein_distance,
        indention_prefix,
    } = options;

    let strikethrough_commands_tip_in_dm = AsOption(strikethrough_commands_tip_in_dm);
    let strikethrough_commands_tip_in_guild = AsOption(strikethrough_commands_tip_in_guild);

    propagate_err!(validate_declaration(&mut fun, DeclarFor::Help));

    let either = [
        parse_quote!(CommandResult),
        parse_quote!(serenity::framework::standard::CommandResult),
    ];

    propagate_err!(validate_return_type(&mut fun, either));

    let options = fun.name.with_suffix(HELP_OPTIONS);

    let n = fun.name.to_uppercase();
    let nn = fun.name.clone();

    let cooked = fun.cooked.clone();
    let cooked2 = cooked.clone();

    let options_path = quote!(serenity::framework::standard::HelpOptions);
    let command_path = quote!(serenity::framework::standard::HelpCommand);

    (quote! {
        #(#cooked)*
        pub static #options: #options_path = #options_path {
            names: &[#(#names),*],
            suggestion_text: #suggestion_text,
            no_help_available_text: #no_help_available_text,
            usage_label: #usage_label,
            usage_sample_label: #usage_sample_label,
            ungrouped_label: #ungrouped_label,
            grouped_label: #grouped_label,
            aliases_label: #aliases_label,
            description_label: #description_label,
            guild_only_text: #guild_only_text,
            checks_label: #checks_label,
            dm_only_text: #dm_only_text,
            dm_and_guild_text: #dm_and_guild_text,
            available_text: #available_text,
            command_not_found_text: #command_not_found_text,
            individual_command_tip: #individual_command_tip,
            group_prefix: #group_prefix,
            strikethrough_commands_tip_in_dm: #strikethrough_commands_tip_in_dm,
            strikethrough_commands_tip_in_guild: #strikethrough_commands_tip_in_guild,
            lacking_role: #lacking_role,
            lacking_permissions: #lacking_permissions,
            lacking_ownership: #lacking_ownership,
            wrong_channel: #wrong_channel,
            embed_error_colour: #embed_error_colour,
            embed_success_colour: #embed_success_colour,
            max_levenshtein_distance: #max_levenshtein_distance,
            indention_prefix: #indention_prefix,
        };

        #(#cooked2)*
        pub static #n: #command_path = #command_path {
            fun: #nn,
            options: &#options,
        };

        #fun
    })
    .into()
}

/// Create a grouping of commands.
///
/// It is a prerequisite for all commands to be assigned under a common group,
/// before they may be executed by a user.
///
/// A group might have one or more *prefixes* set. This will necessitate for
/// one of the prefixes to appear before the group's command.
/// For example, for a general prefix `!`, a group prefix `foo` and a command `bar`,
/// the invocation would be `!foo bar`.
///
/// It might have some options apply to *all* of its commands. E.g. guild or dm only.
///
/// It may even couple other groups as well.
///
/// This group macro purports all of the said purposes above, applied onto a `struct`:
///
/// ```rust,no_run
/// use command_attr::{command, group};
///
/// # type CommandResult = ();
///
/// #[command]
/// fn bar() -> CommandResult {
///     println!("baz");
///
///     Ok(())
/// }
///
/// #[command]
/// fn answer_to_life() -> CommandResult {
///     println!("42");
///
///     Ok(())
/// }
///
/// #[group]
/// // All sub-groups must own at least one prefix.
/// #[prefix = "baz"]
/// #[commands(answer_to_life)]
/// struct Baz;
///
/// #[group]
/// #[commands(bar)]
/// // Case does not matter; the names will be all uppercased.
/// #[sub_groups(baz)]
/// struct Foo;
/// ```
///
/// # Options
///
/// These appear after `#[group]` as a series of attributes:
///
/// - `#[prefixes("foo", "bar", "baz")]`
/// The group's prefixes.
///
/// - `#[allowed_roles("foo", "bar", "baz")]`
/// Only which roles may execute this group's commands.
///
/// - `#[only_in(guilds/dms))]`
/// Whether this group's commands are restricted to `guilds` or `dms`.
///
/// - `#[owners_only(true/false)]`
/// If only the owners of the bot may execute this group's commands.
///
/// - `#[owner_privilege(true/false)]`
/// Whether the owners should be treated as normal users.
///
/// Default value is `true`
///
/// - `#[help_available(true/false)]`
/// Whether the group is visible to the help command.
///
/// Default value is `true`
///
/// - `#[checks(foo, bar, baz)]`
/// A set of preconditions that must be met before a group command's execution.
/// Refer to [`command`]'s `checks` documentation.
///
/// - `#[required_permissions(foo, bar, baz)]`
/// A set of permissions needed by the user before a group command's execution.
///
/// - `#[default_command(foobar_baz)]`
/// Command to be executed if none of the group's prefixes are given.
/// Identifier must refer to a `#[command]`'d function.
///
/// - `#[prefix("...")]`/`#[prefix = "..."]`
/// Assign a single prefix to this group.
///
/// - `#[description("...")]`/`#[description = "..."]`
/// The description of the group.
/// Used in the help command.
///
/// Similarly to [`command`], this macro generates static instances of the group
/// and its options. The identifiers of these instances are based off the name of the struct to differentiate
/// this group from others. This name is given as the default value of the group's `name` field,
/// used in the help command for display and browsing of the group.
/// It may also be passed as an argument to the macro. For example: `#[group("Banana Phone")]`.
///
/// [`command`]: #fn.command.html
#[proc_macro_attribute]
pub fn group(attr: TokenStream, input: TokenStream) -> TokenStream {
    let group = parse_macro_input!(input as GroupStruct);

    let name = if !attr.is_empty() {
        parse_macro_input!(attr as Lit).to_str()
    } else {
        group.name.to_string()
    };

    let mut options = GroupOptions::new();

    for attribute in &group.attributes {
        let span = attribute.span();
        let values = propagate_err!(parse_values(attribute));

        let name = values.name.to_string();
        let name = &name[..];

        match name {
            "prefix" => {
                options.prefixes = vec![propagate_err!(attributes::parse(values))];
            }
            _ => match_options!(name, values, options, span => [
                prefixes;
                only_in;
                owners_only;
                owner_privilege;
                help_available;
                allowed_roles;
                required_permissions;
                checks;
                default_command;
                description;
                commands;
                sub_groups
            ]),
        }
    }

    let GroupOptions {
        prefixes,
        only_in,
        owners_only,
        owner_privilege,
        help_available,
        allowed_roles,
        required_permissions,
        checks,
        default_command,
        description,
        commands,
        sub_groups,
    } = options;

    let cooked = group.cooked.clone();
    let cooked2 = cooked.clone();

    let n = group.name.with_suffix(GROUP);

    let default_command = default_command.map(|ident| {
        let i = ident.with_suffix(COMMAND);

        quote!(&#i)
    });

    let commands = commands
        .into_iter()
        .map(|c| c.with_suffix(COMMAND))
        .collect::<Vec<_>>();

    let sub_groups = sub_groups
        .into_iter()
        .map(|c| c.with_suffix(GROUP))
        .collect::<Vec<_>>();

    let options = group.name.with_suffix(GROUP_OPTIONS);
    let options_path = quote!(serenity::framework::standard::GroupOptions);
    let group_path = quote!(serenity::framework::standard::CommandGroup);

    (quote! {
        #(#cooked)*
        pub static #options: #options_path = #options_path {
            prefixes: &[#(#prefixes),*],
            only_in: #only_in,
            owners_only: #owners_only,
            owner_privilege: #owner_privilege,
            help_available: #help_available,
            allowed_roles: &[#(#allowed_roles),*],
            required_permissions: #required_permissions,
            checks: #checks,
            default_command: #default_command,
            description: #description,
            commands: &[#(&#commands),*],
            sub_groups: &[#(&#sub_groups),*],
        };

        #(#cooked2)*
        pub static #n: #group_path = #group_path {
            name: #name,
            options: &#options,
        };

        #group
    })
    .into()
}

#[proc_macro_attribute]
pub fn check(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(input as CommandFun);

    let mut name = "<fn>".to_string();
    let mut display_in_help = true;
    let mut check_in_help = true;

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = propagate_err!(parse_values(attribute));

        let n = values.name.to_string();
        let n = &n[..];

        match n {
            "name" => name = propagate_err!(attributes::parse(values)),
            "display_in_help" => display_in_help = propagate_err!(attributes::parse(values)),
            "check_in_help" => check_in_help = propagate_err!(attributes::parse(values)),
            _ => {
                return Error::new(span, format_args!("invalid attribute: {:?}", n))
                    .to_compile_error()
                    .into();
            }
        }
    }

    propagate_err!(validate_declaration(&mut fun, DeclarFor::Check));

    let either = [
        parse_quote!(CheckResult),
        parse_quote!(serenity::framework::standard::CheckResult),
    ];

    propagate_err!(validate_return_type(&mut fun, either));

    let n = fun.name.clone();
    let n2 = name.clone();
    let name = if name.is_empty() {
        fun.name.clone()
    } else {
        Ident::new(&name, Span::call_site())
    };
    let name = name.with_suffix(CHECK);

    let check = quote!(serenity::framework::standard::Check);

    (quote! {
        pub static #name: #check = #check {
            name: #n2,
            function: #n,
            display_in_help: #display_in_help,
            check_in_help: #check_in_help
        };

        #fun
    })
    .into()
}
