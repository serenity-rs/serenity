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
/// This is a function attribute macro. Using this on other Rust constructs won't work.
///
/// ## Options
///
/// To alter how the framework will interpret the command,
/// you can provide options as attributes following this `#[command]` macro.
///
/// Each option has its own kind of data to stock and manipulate with.
/// They're given to the option either with the `#[option(...)]` or `#[option = ...]` syntaxes.
/// If an option doesn't require for any data to be supplied, then it's simply an empty `#[option]`.
///
/// If the input to the option is malformed, the macro will give you can error, describing
/// the correct method for passing data, and what it should be.
///
/// The list of available options, is, as follows:
///
/// | Syntax                                                                       | Description                                                                                              | Argument explanation                                                                                                                                                                                                             |
/// | ---------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
/// | `#[checks(identifiers)]`                                                     | Preconditions that must met before the command's execution.                                              | `identifiers` is a comma separated list of identifiers referencing functions marked by the `#[check]` macro                                                                                                                      |
/// | `#[aliases(names)]`                                                          | Alternative names to refer to this command.                                                              | `names` is a comma separate list of desired aliases.                                                                                                                                                                             |
/// | `#[description(desc)]` </br> `#[description = desc]`                         | The command's description or summary.                                                                    | `desc` is a string describing the command.                                                                                                                                                                                       |
/// | `#[usage(use)]` </br> `#[usage = use]`                                       | The command's intended usage.                                                                            | `use` is a string stating the schema for the command's usage.                                                                                                                                                                    |
/// | `#[example(ex)]` </br> `#[example = ex]`                                     | An example of the command's usage. May be called multiple times to add many examples at once.            | `ex` is a string                                                                                                                                                                                                                 |
/// | `#[min_args(min)]` </br> `#[max_args(max)]` </br> `#[num_args(min_and_max)]` | The expected length of arguments that the command must receive in order to function correctly.           | `min`, `max` and `min_and_max` are 16-bit, unsigned integers.                                                                                                                                                                    |
/// | `#[required_permissions(perms)]`                                             | Set of permissions the user must possess.                                                                | `perms` is a comma separated list of permission names.</br> These can be found at [Discord's official documentation](https://discordapp.com/developers/docs/topics/permissions).                                                 |
/// | `#[allowed_roles(roles)]`                                                    | Set of roles the user must possess.                                                                      | `roles` is a comma separated list of role names.                                                                                                                                                                                 |
/// | `#[help_available]` </br> `#[help_available(b)]`                             | If the command should be displayed in the help message.                                                  | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                  |
/// | `#[only_in(ctx)]`                                                            | Which environment the command can be executed in.                                                        | `ctx` is a string with the accepted values `guild`/`guilds` and `dm`/`dms` (Direct Message).                                                                                                                                     |
/// | `#[bucket(name)]` </br> `#[bucket = name]`                                   | What bucket will impact this command.                                                                    | `name` is a string containing the bucket's name.</br> Refer to [the bucket example in the standard framework](https://docs.rs/serenity/*/serenity/framework/standard/struct.StandardFramework.html#method.bucket) for its usage. |
/// | `#[owners_only]` </br> `#[owners_only(b)]`                                   | If this command is exclusive to owners.                                                                  | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                 |
/// | `#[owner_privilege]` </br> `#[owner_privilege(b)]`                           | If owners can bypass certain options.                                                                    | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                 |
/// | `#[sub_commands(commands)]`                                                  | The sub or children commands of this command. They are executed in the form: `this-command sub-command`. | `commands` is a comma separated list of identifiers referencing functions marked by the `#[command]` macro.                                                                                                                      |
///
/// Documentation comments (`///`) applied onto the function are interpreted as sugar for the
/// `#[description]` option. When more than one application of the option is performed,
/// the text is delimited by newlines. This mimics the behaviour of regular doc-comments,
/// which are sugar for the `#[doc = "..."]` attribute.
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
            "description" => {
                let arg: String = propagate_err!(attributes::parse(values));

                if let Some(desc) = &mut options.description.0 {
                    use std::fmt::Write;

                    let _ = write!(desc, "\n{}", arg.trim_matches(' '));
                } else {
                    options.description = AsOption(Some(arg));
                }
            }
            _ => {
                match_options!(name, values, options, span => [
                    checks;
                    bucket;
                    aliases;
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
/// ## Options
///
/// | Syntax                                                                                                                                        | Description                                                                                                                                                                                                                                      | Argument explanation                                                                                       |
/// |-----------------------------------------------------------------------------------------------------------------------------------------------| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------|
/// | `#[suggestion_text(s)]` </br> `#[suggestion_text = s]`                                                                                        | When suggesting a command's name                                                                                                                                                                                                                 | `s` is a string                                                                                            |
/// | `#[no_help_available_text(s)]` </br> `#[no_help_available_text = s]`                                                                          | When help is unavailable for a command.                                                                                                                                                                                                          | `s` is a string                                                                                            |
/// | `#[usage_label(s)]` </br> `#[usage_label = s]`                                                                                                | How should the command be used.                                                                                                                                                                                                                  | `s` is a string                                                                                            |
/// | `#[usage_sample_label(s)]` </br> `#[usage_sample_label = s]`                                                                                  | Actual sample label.                                                                                                                                                                                                                             | `s` is a string                                                                                            |
/// | `#[ungrouped_label(s)]` </br> `#[ungrouped_label = s]`                                                                                        | Ungrouped commands label.                                                                                                                                                                                                                        | `s` is a string                                                                                            |
/// | `#[grouped_label(s)]` </br> `#[grouped_label = s]`                                                                                            | Grouped commands label.                                                                                                                                                                                                                          | `s` is a string                                                                                            |
/// | `#[description_label(s)]` </br> `#[description_label = s]`                                                                                    | Label at the start of the description.                                                                                                                                                                                                           | `s` is a string                                                                                            |
/// | `#[aliases_label(s)]` </br> `#[aliases_label= s]`                                                                                             | Label for a command's aliases.                                                                                                                                                                                                                   | `s` is a string                                                                                            |
/// | `#[guild_only_text(s)]` </br> `#[guild_only_text = s]`                                                                                        | When a command is specific to guilds only.                                                                                                                                                                                                       | `s` is a string                                                                                            |
/// | `#[checks_label(s)]` </br> `#[checks_label = s]`                                                                                              | The header text when showing checks in the help command.                                                                                                                                                                                         | `s` is a string                                                                                            |
/// | `#[dm_only_text(s)]` </br> `#[dm_only_text = s]`                                                                                              | When a command is specific to dms only.                                                                                                                                                                                                          | `s` is a string                                                                                            |
/// | `#[dm_and_guild_text(s)]` </br> `#[dm_and_guild_text = s]`                                                                                    | When a command is usable in both guilds and dms.                                                                                                                                                                                                 | `s` is a string                                                                                            |
/// | `#[available_text(s)]` </br> `#[available_text = s]`                                                                                          | When a command is available.                                                                                                                                                                                                                     | `s` is a string                                                                                            |
/// | `#[command_not_found_text(s)]` </br> `#[command_not_found_text = s]`                                                                          | When a command wasn't found.                                                                                                                                                                                                                     | `s` is a string                                                                                            |
/// | `#[individual_command_tip(s)]` </br> `#[individual_command_tip = s]`                                                                          | How the user should access a command's details.                                                                                                                                                                                                  | `s` is a string                                                                                            |
/// | `#[strikethrough_commands_tip_in_dm]` </br>  `#[strikethrough_commands_tip_in_dm(s)]` </br>`#[strikethrough_commands_tip_in_dm = s]`          | Reasoning behind strikethrough-commands.</br> *Only used in dms.*                                                                                                                                                                                | `s` is a string. If there wasn't any text passed, default text will be used instead.                       |
/// |  `#[strikethrough_commands_tip_in_guild]` </br>`#[strikethrough_commands_tip_in_guild(s)]` </br> `#[strikethrough_commands_tip_in_guild = s]` | Reasoning behind strikethrough-commands.</br> *Only used in guilds.*                                                                                                                                                                             | `s` is a string. If there wasn't any text passed, default text will be used instead.                       |
/// | `#[group_prefix(s)]` </br> `#[group_prefix = s]`                                                                                              | For introducing a group's prefix                                                                                                                                                                                                                 | `s` is a string                                                                                            |
/// | `#[lacking_role(s)]` </br> `#[lacking_role = s]`                                                                                              | If a user lacks required roles, this will treat how commands will be displayed.                                                                                                                                                                  | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[lacking_ownership(s)]` </br> `#[lacking_ownership = s]`                                                                                    | If a user lacks ownership, this will treat how these commands will be displayed.                                                                                                                                                                 | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[lacking_permissions(s)]` </br> `#[lacking_permissions = s]`                                                                                | If a user lacks permissions, this will treat how commands will be displayed.                                                                                                                                                                     | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[embed_error_colour(n)]`                                                                                                                    | Colour that the help-embed will use upon an error.                                                                                                                                                                                               | `n` is a name to one of the provided constants of the `Colour` struct.                                     |
/// | `#[embed_success_colour(n)]`                                                                                                                  | Colour that the help-embed will use normally.                                                                                                                                                                                                    | `n` is a name to one of the provided constants of the `Colour` struct.                                     |
/// | `#[max_levenshtein_distance(n)]`                                                                                                              | How much should the help command search for a similiar name.</br> Indicator for a nested guild. The prefix will be repeated based on what kind of level the item sits. A sub-group would be level two, a sub-sub-group would be level three.     | `n` is a 64-bit, unsigned integer.                                                                         |
/// | `#[indention_prefix(s)]` </br> `#[indention_prefix = s]`                                                                                      | The prefix used to express how deeply nested a command or group is.                                                                                                                                                                              | `s` is a string                                                                                            |
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
            lacking_conditions;
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

        if options.lacking_conditions == HelpBehaviour::Strike {
            is_any_option_strike = true;

            if concat_with_comma {
                strike_text.push_str(", require certain conditions");
            } else {
                strike_text.push_str(" require certain conditions");
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
        lacking_conditions,
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
            lacking_conditions: #lacking_conditions,
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
/// ## Options
///
/// These appear after `#[group]` as a series of attributes:
///
/// | Syntax                                               | Description                                                                        | Argument explanation                                                                                                                                                                 |
/// |------------------------------------------------------|------------------------------------------------------------------------------------| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `#[prefixes(prefs)]`                                 | Text that must appear   before an invocation of a command of this group may occur. | `prefs` is a comma separated list of strings                                                                                                                                         |
/// | `#[prefix(pref)]`                                    | Assign just a single prefix.                                                       | `pref` is a string                                                                                                                                                                   |
/// | `#[allowed_roles(roles)]`                            | Set of roles the user must possess                                                 | `roles` is a comma separated list of strings containing role names                                                                                                                   |
/// | `#[only_in(ctx)]`                                    | Which environment the command can be executed in.                                  | `ctx` is a string with the accepted values `guild`/`guilds` and `dm`/ `dms` (Direct Message).                                                                                        |
/// | `#[owners_only]` </br> `#[owners_only(b)]`           | If this command is exclusive to owners.                                            | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[owner_privilege]` </br> `#[owner_privlege(b)]`    | If owners can bypass certain options.                                              | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[help_available]` </br> `#[help_available(b)]`     | If the group should be displayed in the help message.                              | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[checks(identifiers)]`                             | Preconditions that must met before the command's execution.                        | `identifiers` is a comma separated list of identifiers referencing functions marked by the `#[check]` macro                                                                          |
/// | `#[required_permissions(perms)]`                     | Set of permissions the user must possess.                                          | `perms` is a comma separated list of permission names.</br> These can be found at [Discord's official documentation](https://discordapp.com/developers/docs/topics/permissions).     |
/// | `#[default_command(cmd)]`                            | A command to execute if none of the group's prefixes are given.                    | `cmd` is an identifier referencing a function marked by the `#[command]` macro                                                                                                       |
/// | `#[description(desc)]` </br> `#[description = desc]` | The group's description or summary.                                                | `desc` is a string describing the group.                                                                                                                                             |
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
            "description" => {
                let arg: String = propagate_err!(attributes::parse(values));

                if let Some(desc) = &mut options.description.0 {
                    use std::fmt::Write;

                    let _ = write!(desc, "\n{}", arg.trim_matches(' '));
                } else {
                    options.description = AsOption(Some(arg));
                }
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
