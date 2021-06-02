#![deny(rust_2018_idioms)]
#![deny(broken_intra_doc_links)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Ident,
    Lit,
    Token,
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
/// | `#[aliases(names)]`                                                          | Alternative names to refer to this command.                                                              | `names` is a comma separated list of desired aliases.                                                                                                                                                                             |
/// | `#[description(desc)]` </br> `#[description = desc]`                         | The command's description or summary.                                                                    | `desc` is a string describing the command.                                                                                                                                                                                       |
/// | `#[usage(use)]` </br> `#[usage = use]`                                       | The command's intended usage.                                                                            | `use` is a string stating the schema for the command's usage.                                                                                                                                                                    |
/// | `#[example(ex)]` </br> `#[example = ex]`                                     | An example of the command's usage. May be called multiple times to add many examples at once.            | `ex` is a string                                                                                                                                                                                                                 |
/// | `#[delimiters(delims)]`                                                      | Argument delimiters specific to this command. Overrides the global list of delimiters in the framework.  | `delims` is a comma separated list of strings |
/// | `#[min_args(min)]` </br> `#[max_args(max)]` </br> `#[num_args(min_and_max)]` | The expected length of arguments that the command must receive in order to function correctly.           | `min`, `max` and `min_and_max` are 16-bit, unsigned integers.                                                                                                                                                                    |
/// | `#[required_permissions(perms)]`                                             | Set of permissions the user must possess.                                                                | `perms` is a comma separated list of permission names.</br> These can be found at [Discord's official documentation](https://discord.com/developers/docs/topics/permissions).                                                 |
/// | `#[allowed_roles(roles)]`                                                    | Set of roles the user must possess.                                                                      | `roles` is a comma separated list of role names.                                                                                                                                                                                 |
/// | `#[help_available]` </br> `#[help_available(b)]`                             | If the command should be displayed in the help message.                                                  | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                  |
/// | `#[only_in(ctx)]`                                                            | Which environment the command can be executed in.                                                        | `ctx` is a string with the accepted values `guild`/`guilds` and `dm`/`dms` (Direct Message).                                                                                                                                     |
/// | `#[bucket(name)]` </br> `#[bucket = name]`                                   | What bucket will impact this command.                                                                    | `name` is a string containing the bucket's name.</br> Refer to [the bucket example in the standard framework](https://docs.rs/serenity/*/serenity/framework/standard/struct.StandardFramework.html#method.bucket) for its usage. |
/// | `#[owners_only]` </br> `#[owners_only(b)]`                                   | If this command is exclusive to owners.                                                                  | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                  |
/// | `#[owner_privilege]` </br> `#[owner_privilege(b)]`                           | If owners can bypass certain options.                                                                    | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                                                                  |
/// | `#[sub_commands(commands)]`                                                  | The sub or children commands of this command. They are executed in the form: `this-command sub-command`. | `commands` is a comma separated list of identifiers referencing functions marked by the `#[command]` macro.                                                                                                                      |
///
/// Documentation comments (`///`) applied onto the function are interpreted as sugar for the
/// `#[description]` option. When more than one application of the option is performed,
/// the text is delimited by newlines. This mimics the behaviour of regular doc-comments,
/// which are sugar for the `#[doc = "..."]` attribute. If you wish to join lines together,
/// however, you have to end the previous lines with `\$`.
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
            },
            "example" => {
                options.examples.push(propagate_err!(attributes::parse(values)));
            },
            "description" => {
                let line: String = propagate_err!(attributes::parse(values));
                util::append_line(&mut options.description, line);
            },
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
            },
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

    propagate_err!(create_declaration_validations(&mut fun, DeclarFor::Command));

    let res = parse_quote!(serenity::framework::standard::CommandResult);
    create_return_type_validation(&mut fun, res);

    let visibility = fun.visibility;
    let name = fun.name.clone();
    let options = name.with_suffix(COMMAND_OPTIONS);
    let sub_commands = sub_commands.into_iter().map(|i| i.with_suffix(COMMAND)).collect::<Vec<_>>();
    let body = fun.body;
    let ret = fun.ret;

    let n = name.with_suffix(COMMAND);

    let cooked = fun.cooked.clone();

    let options_path = quote!(serenity::framework::standard::CommandOptions);
    let command_path = quote!(serenity::framework::standard::Command);

    populate_fut_lifetimes_on_refs(&mut fun.args);
    let args = fun.args;

    (quote! {
        #(#cooked)*
        #[allow(missing_docs)]
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

        #(#cooked)*
        #[allow(missing_docs)]
        pub static #n: #command_path = #command_path {
            fun: #name,
            options: &#options,
        };

        #(#cooked)*
        #[allow(missing_docs)]
        #visibility fn #name<'fut> (#(#args),*) -> ::serenity::futures::future::BoxFuture<'fut, #ret> {
            use ::serenity::futures::future::FutureExt;

            async move {
                let _output: #ret = { #(#body)* };
                #[allow(unreachable_code)]
                _output
            }.boxed()
        }
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
/// | `#[sub_commands_label(s)]` </br> `#[sub_commands_label = s]`                                                                                  | Sub commands label.                                                                                                          | `s` is a string
/// | `#[description_label(s)]` </br> `#[description_label = s]`                                                                                    | Label at the start of the description.                                                                                                                                                                                                           | `s` is a string                                                                                            |
/// | `#[aliases_label(s)]` </br> `#[aliases_label= s]`                                                                                             | Label for a command's aliases.                                                                                                                                                                                                                   | `s` is a string                                                                                            |
/// | `#[guild_only_text(s)]` </br> `#[guild_only_text = s]`                                                                                        | When a command is specific to guilds only.                                                                                                                                                                                                       | `s` is a string                                                                                            |
/// | `#[checks_label(s)]` </br> `#[checks_label = s]`                                                                                              | The header text when showing checks in the help command.                                                                                                                                                                                         | `s` is a string                                                                                            |
/// | `#[dm_only_text(s)]` </br> `#[dm_only_text = s]`                                                                                              | When a command is specific to dms only.                                                                                                                                                                                                          | `s` is a string                                                                                            |
/// | `#[dm_and_guild_text(s)]` </br> `#[dm_and_guild_text = s]`                                                                                    | When a command is usable in both guilds and dms.                                                                                                                                                                                                 | `s` is a string                                                                                            |
/// | `#[available_text(s)]` </br> `#[available_text = s]`                                                                                          | When a command is available.                                                                                                                                                                                                                     | `s` is a string                                                                                            |
/// | `#[command_not_found_text(s)]` </br> `#[command_not_found_text = s]`                                                                          | When a command wasn't found.                                                                                                                                                                                                                     | `s` is a string                                                                                            |
/// | `#[individual_command_tip(s)]` </br> `#[individual_command_tip = s]`                                                                          | How the user should access a command's details.                                                                                                                                                                                                  | `s` is a string                                                                                            |
/// | `#[strikethrough_commands_tip_in_dm(s)]` </br>  `#[strikethrough_commands_tip_in_dm = s]`                                                     | Reasoning behind strikethrough-commands.</br> *Only used in dms.*                                                                                                                                                                                | `s` is a string. If not provided, default text will be used instead.                                       |
/// | `#[strikethrough_commands_tip_in_guild(s)]` </br> `#[strikethrough_commands_tip_in_guild = s]`                                                | Reasoning behind strikethrough-commands.</br> *Only used in guilds.*                                                                                                                                                                             | `s` is a string. If not provided, default text will be used instead.                                       |
/// | `#[group_prefix(s)]` </br> `#[group_prefix = s]`                                                                                              | For introducing a group's prefix                                                                                                                                                                                                                 | `s` is a string                                                                                            |
/// | `#[lacking_role(s)]` </br> `#[lacking_role = s]`                                                                                              | If a user lacks required roles, this will treat how commands will be displayed.                                                                                                                                                                  | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[lacking_ownership(s)]` </br> `#[lacking_ownership = s]`                                                                                    | If a user lacks ownership, this will treat how these commands will be displayed.                                                                                                                                                                 | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[lacking_permissions(s)]` </br> `#[lacking_permissions = s]`                                                                                | If a user lacks permissions, this will treat how commands will be displayed.                                                                                                                                                                     | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[lacking_conditions(s)]` </br> `#[lacking_conditions = s]`                                                                                  | If conditions (of a check) may be lacking by the user, this will treat how these commands will be displayed.                                                                                                                                     | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[wrong_channel(s)]` </br> `#[wrong_channel = s]`                                                                                            | If a user is using the help-command in a channel where a command is not available, this behaviour will be executed.                                                                                                                              | `s` is a string. Accepts `strike` (strikethroughs), `hide` (will not be listed) or `nothing`(leave be).    |
/// | `#[embed_error_colour(n)]`                                                                                                                    | Colour that the help-embed will use upon an error.                                                                                                                                                                                               | `n` is a name to one of the provided constants of the `Colour` struct or an RGB value `#RRGGBB`.           |
/// | `#[embed_success_colour(n)]`                                                                                                                  | Colour that the help-embed will use normally.                                                                                                                                                                                                    | `n` is a name to one of the provided constants of the `Colour` struct or an RGB value `#RRGGBB`.           |
/// | `#[max_levenshtein_distance(n)]`                                                                                                              | How much should the help command search for a similiar name.</br> Indicator for a nested guild. The prefix will be repeated based on what kind of level the item sits. A sub-group would be level two, a sub-sub-group would be level three.     | `n` is a 64-bit, unsigned integer.                                                                         |
/// | `#[indention_prefix(s)]` </br> `#[indention_prefix = s]`                                                                                      | The prefix used to express how deeply nested a command or group is.                                                                                                                                                                              | `s` is a string                                                                                            |
///
/// [`command`]: macro@command
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

    // Revert the change for the names of documentation attributes done when
    // parsing the function input with `CommandFun`.
    util::rename_attributes(&mut fun.attributes, "description", "doc");

    // Additionally, place the documentation attributes to the `cooked` list
    // to prevent the macro from rejecting them as invalid attributes.
    {
        let mut i = 0;
        while i < fun.attributes.len() {
            if fun.attributes[i].path.is_ident("doc") {
                fun.cooked.push(fun.attributes.remove(i));
                continue;
            }

            i += 1;
        }
    }

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
            sub_commands_label;
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

    if options.strikethrough_commands_tip_in_dm == None {
        options.strikethrough_commands_tip_in_dm = produce_strike_text(&options, "direct messages");
    }

    if options.strikethrough_commands_tip_in_guild == None {
        options.strikethrough_commands_tip_in_guild =
            produce_strike_text(&options, "server messages");
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
        sub_commands_label,
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

    propagate_err!(create_declaration_validations(&mut fun, DeclarFor::Help));

    let res = parse_quote!(serenity::framework::standard::CommandResult);
    create_return_type_validation(&mut fun, res);

    let options = fun.name.with_suffix(HELP_OPTIONS);

    let n = fun.name.to_uppercase();
    let nn = fun.name.clone();

    let cooked = fun.cooked.clone();

    let options_path = quote!(serenity::framework::standard::HelpOptions);
    let command_path = quote!(serenity::framework::standard::HelpCommand);

    let body = fun.body;
    let ret = fun.ret;
    populate_fut_lifetimes_on_refs(&mut fun.args);
    let args = fun.args;

    (quote! {
        #(#cooked)*
        #[allow(missing_docs)]
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
            sub_commands_label: #sub_commands_label,
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

        #(#cooked)*
        #[allow(missing_docs)]
        pub static #n: #command_path = #command_path {
            fun: #nn,
            options: &#options,
        };

        #(#cooked)*
        #[allow(missing_docs)]
        pub fn #nn<'fut>(#(#args),*) -> ::serenity::futures::future::BoxFuture<'fut, #ret> {
            use ::serenity::futures::future::FutureExt;

            async move {
                let _output: #ret = { #(#body)* };
                #[allow(unreachable_code)]
                _output
            }.boxed()
        }
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
/// ```rust,ignore
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
/// | `#[commands(commands)]`                              | Set of commands belonging to this group.                                           | `commands` is a comma separated list of identifiers referencing functions marked by the `#[command]` macro                                                                           |
/// | `#[sub_groups(subs)]`                                | Set of sub groups belonging to this group.                                         | `subs` is a comma separated list of identifiers referencing structs marked by the `#[group]` macro                                                                                   |
/// | `#[prefixes(prefs)]`                                 | Text that must appear   before an invocation of a command of this group may occur. | `prefs` is a comma separated list of strings                                                                                                                                         |
/// | `#[prefix(pref)]`                                    | Assign just a single prefix.                                                       | `pref` is a string                                                                                                                                                                   |
/// | `#[allowed_roles(roles)]`                            | Set of roles the user must possess                                                 | `roles` is a comma separated list of strings containing role names                                                                                                                   |
/// | `#[only_in(ctx)]`                                    | Which environment the command can be executed in.                                  | `ctx` is a string with the accepted values `guild`/`guilds` and `dm`/ `dms` (Direct Message).                                                                                        |
/// | `#[owners_only]` </br> `#[owners_only(b)]`           | If this command is exclusive to owners.                                            | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[owner_privilege]` </br> `#[owner_privilege(b)]`   | If owners can bypass certain options.                                              | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[help_available]` </br> `#[help_available(b)]`     | If the group should be displayed in the help message.                              | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.                                                                                                      |
/// | `#[checks(identifiers)]`                             | Preconditions that must met before the command's execution.                        | `identifiers` is a comma separated list of identifiers referencing functions marked by the `#[check]` macro                                                                          |
/// | `#[required_permissions(perms)]`                     | Set of permissions the user must possess.                                          | `perms` is a comma separated list of permission names.</br> These can be found at [Discord's official documentation](https://discord.com/developers/docs/topics/permissions).        |
/// | `#[default_command(cmd)]`                            | A command to execute if none of the group's prefixes are given.                    | `cmd` is an identifier referencing a function marked by the `#[command]` macro                                                                                                       |
/// | `#[description(desc)]` </br> `#[description = desc]` | The group's description or summary.                                                | `desc` is a string describing the group.                                                                                                                                             |
/// | `#[summary(desc)]` </br> `#[summary = desc]`         | A summary group description displayed when shown multiple groups.                  | `desc` is a string summaryly describing the group.                                                                                                                                   |
///
/// Documentation comments (`///`) applied onto the struct are interpreted as sugar for the
/// `#[description]` option. When more than one application of the option is performed,
/// the text is delimited by newlines. This mimics the behaviour of regular doc-comments,
/// which are sugar for the `#[doc = "..."]` attribute. If you wish to join lines together,
/// however, you have to end the previous lines with `\$`.
///
/// Similarly to [`command`], this macro generates static instances of the group
/// and its options. The identifiers of these instances are based off the name of the struct to differentiate
/// this group from others. This name is given as the default value of the group's `name` field,
/// used in the help command for display and browsing of the group.
/// It may also be passed as an argument to the macro. For example: `#[group("Banana Phone")]`.
///
/// [`command`]: macro@command

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
            },
            "description" => {
                let line: String = propagate_err!(attributes::parse(values));
                util::append_line(&mut options.description, line);
            },
            "summary" => {
                let arg: String = propagate_err!(attributes::parse(values));

                if let Some(desc) = &mut options.summary.0 {
                    use std::fmt::Write;

                    let _ = write!(desc, "\n{}", arg.trim_matches(' '));
                } else {
                    options.summary = AsOption(Some(arg));
                }
            },
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
        summary,
        commands,
        sub_groups,
    } = options;

    let cooked = group.cooked.clone();

    let n = group.name.with_suffix(GROUP);

    let default_command = default_command.map(|ident| {
        let i = ident.with_suffix(COMMAND);

        quote!(&#i)
    });

    let commands = commands.into_iter().map(|c| c.with_suffix(COMMAND)).collect::<Vec<_>>();

    let sub_groups = sub_groups.into_iter().map(|c| c.with_suffix(GROUP)).collect::<Vec<_>>();

    let options = group.name.with_suffix(GROUP_OPTIONS);
    let options_path = quote!(serenity::framework::standard::GroupOptions);
    let group_path = quote!(serenity::framework::standard::CommandGroup);

    (quote! {
        #(#cooked)*
        #[allow(missing_docs)]
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
            summary: #summary,
            commands: &[#(&#commands),*],
            sub_groups: &[#(&#sub_groups),*],
        };

        #(#cooked)*
        #[allow(missing_docs)]
        pub static #n: #group_path = #group_path {
            name: #name,
            options: &#options,
        };

        #group
    })
    .into()
}

/// A macro for marking a function as a condition checker to groups and commands.
///
/// ## Options
///
/// | Syntax                                             | Description                                                              | Argument explanation                                                                 |
/// |----------------------------------------------------|--------------------------------------------------------------------------|--------------------------------------------------------------------------------------|
/// | `#[name(s)]` </br> `#[name = s]`                   | How the check should be listed in help.                                  | `s` is a string. If this option isn't provided, the value is assumed to be `"<fn>"`. |
/// | `#[display_in_help]` </br> `#[display_in_help(b)]` | If the check should be listed in help. Has no effect on `check_in_help`. | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.      |
/// | `#[check_in_help]` </br> `#[check_in_help(b)]`     | If the check should be evaluated in help.                                | `b` is a boolean. If no boolean is provided, the value is assumed to be `true`.      |
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
            },
        }
    }

    propagate_err!(create_declaration_validations(&mut fun, DeclarFor::Check));

    let res = parse_quote!(std::result::Result<(), serenity::framework::standard::Reason>);
    create_return_type_validation(&mut fun, res);

    let n = fun.name.clone();
    let n2 = name.clone();
    let visibility = fun.visibility;
    let name = if name == "<fn>" { fun.name.clone() } else { Ident::new(&name, Span::call_site()) };
    let name = name.with_suffix(CHECK);

    let check = quote!(serenity::framework::standard::Check);
    let cooked = fun.cooked;
    let body = fun.body;
    let ret = fun.ret;
    populate_fut_lifetimes_on_refs(&mut fun.args);
    let args = fun.args;

    (quote! {
        #[allow(missing_docs)]
        pub static #name: #check = #check {
            name: #n2,
            function: #n,
            display_in_help: #display_in_help,
            check_in_help: #check_in_help
        };

        #(#cooked)*
        #[allow(missing_docs)]
        #visibility fn #n<'fut>(#(#args),*) -> ::serenity::futures::future::BoxFuture<'fut, #ret> {
            use ::serenity::futures::future::FutureExt;

            async move {
                let _output: #ret = { #(#body)* };
                #[allow(unreachable_code)]
                _output
            }.boxed()
        }
    })
    .into()
}

/// A macro that transforms `async` functions (and closures) into plain functions, whose
/// return type is a boxed [`Future`].
///
/// # Transformation
///
/// The macro transforms an `async` function, which may look like this:
///
/// ```rust,no_run
/// async fn foo(n: i32) -> i32 {
///     n + 4
/// }
/// ```
///
/// into this (some details omitted):
///
/// ```rust,no_run
/// use std::future::Future;
/// use std::pin::Pin;
///
/// fn foo(n: i32) -> Pin<Box<dyn std::future::Future<Output = i32>>> {
///     Box::pin(async move {
///         n + 4
///     })
/// }
/// ```
///
/// This transformation also applies to closures, which are converted more simply. For instance,
/// this closure:
///
/// ```rust,no_run
/// # #![feature(async_closure)]
/// #
/// async move |x: i32| {
///     x * 2 + 4
/// }
/// # ;
/// ```
///
/// is changed to:
///
/// ```rust,no_run
/// |x: i32| {
///     Box::pin(async move {
///         x * 2 + 4
///     })
/// }
/// # ;
/// ```
///
/// ## How references are handled
///
/// When a function contains references, their lifetimes are constrained to the returned
/// [`Future`]. If the above `foo` function had `&i32` as a parameter, the transformation would be
/// instead this:
///
/// ```rust,no_run
/// use std::future::Future;
/// use std::pin::Pin;
///
/// fn foo<'fut>(n: &'fut i32) -> Pin<Box<dyn std::future::Future<Output = i32> + 'fut>> {
///     Box::pin(async move {
///         *n + 4
///     })
/// }
/// ```
///
/// Explicitly specifying lifetimes (in the parameters or in the return type) or complex usage of
/// lifetimes (e.g. `'a: 'b`) is not supported.
///
/// # Necessity for the macro
///
/// The macro performs the transformation to permit the framework to store and invoke the functions.
///
/// Functions marked with the `async` keyword will wrap their return type with the [`Future`] trait,
/// which a state-machine generated by the compiler for the function will implement. This complicates
/// matters for the framework, as [`Future`] is a trait. Depending on a type that implements a trait
/// is done with two methods in Rust:
///
/// 1. static dispatch - generics
/// 2. dynamic dispatch - trait objects
///
/// First method is infeasible for the framework. Typically, the framework will contain a plethora
/// of different commands that will be stored in a single list. And due to the nature of generics,
/// generic types can only resolve to a single concrete type. If commands had a generic type for
/// their function's return type, the framework would be unable to store commands, as only a single
/// [`Future`] type from one of the commands would get resolved, preventing other commands from being
/// stored.
///
/// Second method involves heap allocations, but is the only working solution. If a trait is
/// object-safe (which [`Future`] is), the compiler can generate a table of function pointers
/// (a vtable) that correspond to certain implementations of the trait. This allows to decide
/// which implementation to use at runtime. Thus, we can use the interface for the [`Future`] trait,
/// and avoid depending on the underlying value (such as its size). To opt-in to dynamic dispatch,
/// trait objects must be used with a pointer, like references (`&` and `&mut`) or `Box`. The
/// latter is what's used by the macro, as the ownership of the value (the state-machine) must be
/// given to the caller, the framework in this case.
///
/// The macro exists to retain the normal syntax of `async` functions (and closures), while
/// granting the user the ability to pass those functions to the framework, like command functions
/// and hooks (`before`, `after`, `on_dispatch_error`, etc.).
///
/// # Notes
///
/// If applying the macro on an `async` closure, you will need to enable the `async_closure`
/// feature. Inputs to procedural macro attributes must be valid Rust code, and `async`
/// closures are not stable yet.
///
/// [`Future`]: std::future::Future
#[proc_macro_attribute]
pub fn hook(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let hook = parse_macro_input!(input as Hook);

    match hook {
        Hook::Function(mut fun) => {
            let cooked = fun.cooked;
            let visibility = fun.visibility;
            let fun_name = fun.name;
            let body = fun.body;
            let ret = fun.ret;

            populate_fut_lifetimes_on_refs(&mut fun.args);
            let args = fun.args;

            (quote! {
                #(#cooked)*
                #[allow(missing_docs)]
                #visibility fn #fun_name<'fut>(#(#args),*) -> ::serenity::futures::future::BoxFuture<'fut, #ret> {
                    use ::serenity::futures::future::FutureExt;

                    async move {
                        let _output: #ret = { #(#body)* };
                        #[allow(unreachable_code)]
                        _output
                    }.boxed()
                }
            })
                .into()
        },
        Hook::Closure(closure) => {
            let cooked = closure.cooked;
            let args = closure.args;
            let ret = closure.ret;
            let body = closure.body;

            (quote! {
                #(#cooked)*
                |#args| #ret {
                    use ::serenity::futures::future::FutureExt;

                    async move { #body }.boxed()
                }
            })
            .into()
        },
    }
}
