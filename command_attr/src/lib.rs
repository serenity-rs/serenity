// It has become too of a burden for the `quote!` macro.
#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    Ident, Lit, Path, ReturnType, Type,
};

pub(crate) mod attributes;
pub(crate) mod consts;
pub(crate) mod structures;
pub(crate) mod util;

use self::attributes::*;
use self::consts::*;
use self::structures::*;
use self::util::*;

// Stolen from <https://github.com/diesel-rs/diesel/blob/af1ff2476f997d6eb04fffd58260705d77ff6b6f/diesel_derives/src/util.rs#L81-L96>
pub(crate) fn crate_name() -> Path {
    let in_self = std::env::var("CARGO_PKG_NAME").unwrap() == "serenity";
    let in_doctest = std::env::args()
        .next()
        .unwrap_or_default()
        .contains("rustdoc");

    if in_self && !in_doctest {
        parse_quote!(crate)
    } else {
        parse_quote!(serenity)
    }
}

fn validate_declaration(fun: &mut CommandFun, is_help: bool) -> Result<()> {
    if is_help && fun.args.len() > 6 {
        return Err(Error::new(
            fun.args.last().unwrap().span(),
            "function's arity exceeds more than 6 arguments",
        ));
    } else if !is_help && fun.args.len() > 3 {
        return Err(Error::new(
            fun.args.last().unwrap().span(),
            "function's arity exceeds more than 3 arguments",
        ));
    }

    let context: Type = parse_quote!(&mut Context);
    let message: Type = parse_quote!(&Message);
    let args: Type = parse_quote!(Args);
    let options: Type = parse_quote!(&'static HelpOptions);
    let groups: Type = parse_quote!(&[&'static CommandGroup]);
    let owners: Type = parse_quote!(HashSet<UserId>);

    let cname = crate_name();
    let context_path: Type = parse_quote!(&mut #cname::prelude::Context);
    let message_path: Type = parse_quote!(&#cname::model::channel::Message);
    let args_path: Type = parse_quote!(#cname::framework::standard::Args);
    let options_path: Type = parse_quote!(&'static #cname::framework::standard::HelpOptions);
    let groups_path: Type = parse_quote!(&[&'static #cname::framework::standard::CommandGroup]);
    let owners_path: Type = parse_quote!(std::collections::HashSet<#cname::model::id::UserId>);

    let ctx_error = "first argument's type should be `&mut Context`";
    let msg_error = "second argument's type should be `&Message`";
    let args_error = "third argument's type should be `Args`";
    let options_error = "fourth argument's type should be `&'static HelpOptions`";
    let groups_error = "fifth argument's type should be `&[&'static CommandGroup]`";
    let owners_error = "sixth argument's type should be `HashSet<UserId>`";

    match fun.args.len() {
        0 => {
            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_ctx", Span::call_site()),
                kind: context_path,
            });
            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_msg", Span::call_site()),
                kind: message_path,
            });
            fun.args.push(Argument {
                mutable: Some(parse_quote!(mut)),
                name: Ident::new("_args", Span::call_site()),
                kind: args_path,
            });

            if is_help {
                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_options", Span::call_site()),
                    kind: options_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_groups", Span::call_site()),
                    kind: groups_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_owners", Span::call_site()),
                    kind: owners_path,
                });
            }
        }
        1 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_msg", Span::call_site()),
                kind: message_path,
            });
            fun.args.push(Argument {
                mutable: Some(parse_quote!(mut)),
                name: Ident::new("_args", Span::call_site()),
                kind: args_path,
            });

            if is_help {
                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_options", Span::call_site()),
                    kind: options_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_groups", Span::call_site()),
                    kind: groups_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_owners", Span::call_site()),
                    kind: owners_path,
                });
            }
        }
        2 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            if fun.args[1].kind != message {
                return Err(Error::new(fun.args[1].span(), msg_error));
            }

            fun.args.push(Argument {
                mutable: Some(parse_quote!(mut)),
                name: Ident::new("_args", Span::call_site()),
                kind: args_path,
            });

            if is_help {
                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_options", Span::call_site()),
                    kind: options_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_groups", Span::call_site()),
                    kind: groups_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_owners", Span::call_site()),
                    kind: owners_path,
                });
            }
        }
        3 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            if fun.args[1].kind != message {
                return Err(Error::new(fun.args[1].span(), msg_error));
            }

            if fun.args[2].kind != args {
                return Err(Error::new(fun.args[2].span(), args_error));
            }

            if is_help {
                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_options", Span::call_site()),
                    kind: options_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_groups", Span::call_site()),
                    kind: groups_path,
                });

                fun.args.push(Argument {
                    mutable: None,
                    name: Ident::new("_owners", Span::call_site()),
                    kind: owners_path,
                });
            }
        }
        4 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            if fun.args[1].kind != message {
                return Err(Error::new(fun.args[1].span(), msg_error));
            }

            if fun.args[2].kind != args {
                return Err(Error::new(fun.args[2].span(), args_error));
            }

            if fun.args[3].kind != options {
                return Err(Error::new(fun.args[3].span(), options_error));
            }

            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_groups", Span::call_site()),
                kind: groups_path,
            });

            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_owners", Span::call_site()),
                kind: owners_path,
            });
        }
        5 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            if fun.args[1].kind != message {
                return Err(Error::new(fun.args[1].span(), msg_error));
            }

            if fun.args[2].kind != args {
                return Err(Error::new(fun.args[2].span(), args_error));
            }

            if fun.args[3].kind != options {
                return Err(Error::new(fun.args[3].span(), options_error));
            }

            if fun.args[4].kind != groups {
                return Err(Error::new(fun.args[4].span(), groups_error));
            }

            fun.args.push(Argument {
                mutable: None,
                name: Ident::new("_owners", Span::call_site()),
                kind: owners_path,
            });
        }
        6 => {
            if fun.args[0].kind != context {
                return Err(Error::new(fun.args[0].span(), ctx_error));
            }

            if fun.args[1].kind != message {
                return Err(Error::new(fun.args[1].span(), msg_error));
            }

            if fun.args[2].kind != args {
                return Err(Error::new(fun.args[2].span(), args_error));
            }

            if fun.args[3].kind != options {
                return Err(Error::new(fun.args[3].span(), options_error));
            }

            if fun.args[4].kind != groups {
                return Err(Error::new(fun.args[4].span(), groups_error));
            }

            if fun.args[5].kind != owners {
                return Err(Error::new(fun.args[5].span(), owners_error));
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn validate_return_type(fun: &mut CommandFun) -> Result<()> {
    let span = fun.ret.span();
    let kind = match fun.ret {
        ReturnType::Type(_, ref kind) => kind,
        _ => unreachable!(),
    };

    let want: Type = parse_quote!(CommandResult);

    if &**kind != &want {
        return Err(Error::new(
            span,
            &format!(
                "expected a result as a return type, but got `{}`",
                quote!(#kind)
            ),
        ));
    }

    Ok(())
}

/// The heart of the attribute-based framework.
///
/// This is a function attribute macro, if you attempt to use this on other Rust constructs this will fail to work.
///
/// # Options
///
/// The point of this attribute is for easily configurable options,
/// altering the way the framework will interpret the command.
///
/// Options are passed also as attributes. Though each have their own way of accepting input.
///
/// Available options, are as follows:
///
/// - `#[checks(idents)]`
/// Preconditions that must be met. Executed before the command's execution.
/// `idents` is a list of identifiers, seperated by a comma, referencing functions of the declaration:
/// `Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> bool`
///
/// - `#[aliases(names)]`
/// A list of other names that can be used to execute this command.
/// In `CommandOptions`, these are put in the `names` field, right after the command's name.
///
/// - `#[description(desc)]`/`#[description = desc]`
/// A summary of the command.
///
/// - `#[usage(how_to)]`/`#[usage = how_to]
/// An example of the command's usage.
///
/// - `#[min_args(min)]`, `#[max_args(max)]`, `#[num_args(min_and_max)]`
/// The minimum and/or maximum amount of arguments that the command should/can receive.
///
/// `num_args` is a helper attribute, serving as a shorthand for calling
/// `min_args` and `max_args` with the same number of arguments.
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
/// - `#[owners_only]`/`#[owners_only(bool)]`
/// Whether this command is exclusive to owners.
///
/// - `#[owner_privilege]`/`#[owner_privilege]
/// Whether this command has a privilege for owners (i.e certain options are ignored for them).
///
/// - `#[sub(commands)]`
/// A list of command names, separated by a comma, stating the subcommands of this command.
/// These are executed in the form: `this-command sub-command`
///
/// # Notes
/// The name of the command is parsed from the applied function,
/// or can be passed inside the `#[command]` attribute, a lá `#[command(foobar)]`.
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

    let mut options = Options::default();

    options.help_available = true;
    options.owner_privilege = true;

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = match parse_values(attribute) {
            Ok(vals) => vals,
            Err(err) => return err.to_compile_error().into(),
        };

        let name = values.name.to_string();
        match &name[..] {
            "checks" => {
                let AChecks(idents) = AChecks::parse(values);

                options.checks = Checks(idents);
            }
            "aliases" => {
                let Aliases(args) = Aliases::parse(values);

                options.names = args;
            }
            "description" => {
                let Description(arg) = Description::parse(values);

                options.desc = Some(arg);
            }
            "usage" => {
                let Usage(arg) = Usage::parse(values);

                options.usage = Some(arg);
            }
            "min_args" => {
                let MinArgs(arg) = MinArgs::parse(values);

                options.min_args = Some(arg);
            }
            "max_args" => {
                let MaxArgs(arg) = MaxArgs::parse(values);

                options.max_args = Some(arg);
            }
            "num_args" => {
                let NumArgs(arg) = NumArgs::parse(values);

                options.min_args = Some(arg);
                options.max_args = Some(arg);
            }
            "allowed_roles" => {
                let AllowedRoles(args) = AllowedRoles::parse(values);

                options.allowed_roles = args;
            }
            "help_available" => {
                let HelpAvailable(b) = HelpAvailable::parse(values);

                options.help_available = b;
            }
            "only_in" => {
                let Only(o) = Only::parse(values);

                options.only_in = o;
            }
            "owners_only" => {
                let OwnersOnly(b) = OwnersOnly::parse(values);

                options.owners_only = b;
            }
            "owner_privilege" => {
                let OwnerPrivilege(b) = OwnerPrivilege::parse(values);

                options.owner_privilege = b;
            }
            "sub" => {
                let SubCommands(s) = SubCommands::parse(values);

                options.sub = s;
            }
            "required_permissions" => {
                let RequiredPermissions(p) = RequiredPermissions::parse(values);

                let mut permissions = Permissions::default();
                for perm in p {
                    let p = match Permissions::from_str(&perm.to_string()) {
                        Some(p) => p,
                        None => {
                            return Error::new(perm.span(), "invalid permission")
                                .to_compile_error()
                                .into();
                        }
                    };

                    // Add them together.
                    permissions.0 |= p.0;
                }

                options.required_permissions = permissions;
            }
            _ => {
                return Error::new(span, &format!("invalid attribute: {:?}", name))
                    .to_compile_error()
                    .into();
            }
        }
    }

    let Options {
        checks,
        names: aliases,
        desc,
        usage,
        min_args,
        max_args,
        allowed_roles,
        required_permissions,
        help_available,
        only_in,
        owners_only,
        owner_privilege,
        sub,
    } = options;

    let desc = AsOption(desc);
    let usage = AsOption(usage);
    let min_args = AsOption(min_args);
    let max_args = AsOption(max_args);

    if let Err(err) = validate_declaration(&mut fun, false) {
        return err.to_compile_error().into();
    }

    if let Err(err) = validate_return_type(&mut fun) {
        return err.to_compile_error().into();
    }

    let name = _name.clone();

    // If name starts with a number, prepend an underscore to make it a valid identifier.
    let _name = if _name.starts_with(|c: char| c.is_numeric()) {
        Ident::new(&format!("_{}", _name), Span::call_site())
    } else {
        Ident::new(&_name, Span::call_site())
    };

    let required_permissions = required_permissions.0;

    let options = _name.with_suffix(COMMAND_OPTIONS);
    let sub = sub
        .into_iter()
        .map(|i| i.with_suffix(COMMAND))
        .collect::<Vec<_>>();

    let n = _name.with_suffix(COMMAND);
    let nn = fun.name.clone();

    let cfgs = fun.cfgs.clone();
    let cfgs2 = cfgs.clone();

    let cname = crate_name();
    let options_path = quote!(#cname::framework::standard::CommandOptions);
    let command_path = quote!(#cname::framework::standard::Command);
    let permissions_path = quote!(#cname::model::permissions::Permissions);

    (quote! {
        #(#cfgs)*
        pub static #options: #options_path = #options_path {
            checks: #checks,
            names: &[#name, #(#aliases),*],
            desc: #desc,
            usage: #usage,
            min_args: #min_args,
            max_args: #max_args,
            allowed_roles: &[#(#allowed_roles),*],
            required_permissions: #permissions_path { bits: #required_permissions },
            help_available: #help_available,
            only_in: #only_in,
            owners_only: #owners_only,
            owner_privilege: #owner_privilege,
            sub: &[#(&#sub),*],
        };

        #(#cfgs2)*
        pub static #n: #command_path = #command_path {
            fun: #nn,
            options: &#options,
        };

        #fun
    })
    .into()
}

/// Just like [`command`], this is a function attribute macro for easily configurable options.
///
/// However, this is intended for defining the help command.
/// An interface for simple browsing of all the available commands the bot provides,
/// and reading through specific information regarding a command.
///
/// Therefore, the options here will pertain in the help command's **layout**,
/// rather than its functionality.
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
/// - `#[striked_commands_tip_in_dm]`/`#[striked_commands_tip_in_dm(s)]`/`#[striked_commands_tip_in_dm = s]`
/// Reasoning behind strikethrough-commands.
///
/// If there wasn't any text passed, default text will be used instead.
///
/// *Only used in dms.*
///
/// - `#[striked_commands_tip_in_guild]`/`#[striked_commands_tip_in_guild(s)]`/`#[striked_commands_tip_in_guild = s]`
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
/// [`command`]: fn.command.html
#[proc_macro_attribute]
pub fn help(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(input as CommandFun);

    let mut options = HelpOptions::default();

    for attribute in &fun.attributes {
        let span = attribute.span();
        let values = match parse_values(attribute) {
            Ok(vals) => vals,
            Err(err) => return err.to_compile_error().into(),
        };

        let name = values.name.to_string();
        match &name[..] {
            "suggestion_text" => {
                let SuggestionText(s) = SuggestionText::parse(values);

                options.suggestion_text = s;
            }
            "no_help_available_text" => {
                let NoHelpAvailableText(s) = NoHelpAvailableText::parse(values);

                options.no_help_available_text = s;
            }
            "usage_label" => {
                let UsageLabel(s) = UsageLabel::parse(values);

                options.usage_label = s;
            }
            "usage_sample_label" => {
                let UsageSampleLabel(s) = UsageSampleLabel::parse(values);

                options.usage_sample_label = s;
            }
            "ungrouped_label" => {
                let UngroupedLabel(s) = UngroupedLabel::parse(values);

                options.ungrouped_label = s;
            }
            "grouped_label" => {
                let GroupedLabel(s) = GroupedLabel::parse(values);

                options.grouped_label = s;
            }
            "aliases_label" => {
                let AliasesLabel(s) = AliasesLabel::parse(values);

                options.aliases_label = s;
            }
            "description_label" => {
                let DescriptionLabel(s) = DescriptionLabel::parse(values);

                options.description_label = s;
            }
            "guild_only_text" => {
                let GuildOnlyText(s) = GuildOnlyText::parse(values);

                options.guild_only_text = s;
            }
            "checks_label" => {
                let ChecksLabel(s) = ChecksLabel::parse(values);

                options.checks_label = s;
            }
            "dm_only_text" => {
                let DmOnlyText(s) = DmOnlyText::parse(values);

                options.dm_only_text = s;
            }
            "dm_and_guild_text" => {
                let DmAndGuildText(s) = DmAndGuildText::parse(values);

                options.dm_and_guild_text = s;
            }
            "available_text" => {
                let AvailableText(s) = AvailableText::parse(values);

                options.available_text = s;
            }
            "command_not_found_text" => {
                let CommandNotFoundText(s) = CommandNotFoundText::parse(values);

                options.command_not_found_text = s;
            }
            "individual_command_tip" => {
                let IndividualCommandTip(s) = IndividualCommandTip::parse(values);

                options.individual_command_tip = s;
            }
            "group_prefix" => {
                let GroupPrefix(s) = GroupPrefix::parse(values);

                options.group_prefix = s;
            }

            "strikethrough_commands_tip_in_dm" => {
                let StrikeThroughCommandsTipInDm(s) = StrikeThroughCommandsTipInDm::parse(values);

                options.strikethrough_commands_tip_in_dm = s;
            }
            "strikethrough_commands_tip_in_guild" => {
                let StrikeThroughCommandsTipInGuild(s) =
                    StrikeThroughCommandsTipInGuild::parse(values);

                options.strikethrough_commands_tip_in_guild = s;
            }
            "lacking_role" => {
                let LackingRole(s) = LackingRole::parse(values);

                options.lacking_role = match HelpBehaviour::from_str(&s) {
                    Some(h) => h,
                    None => {
                        return Error::new(span, &format!("invalid help behaviour: {:?}", s))
                            .to_compile_error()
                            .into();
                    }
                };
            }
            "lacking_permissions" => {
                let LackingPermissions(s) = LackingPermissions::parse(values);

                options.lacking_permissions = match HelpBehaviour::from_str(&s) {
                    Some(h) => h,
                    None => {
                        return Error::new(span, &format!("invalid help behaviour: {:?}", s))
                            .to_compile_error()
                            .into();
                    }
                };
            }
            "lacking_ownership" => {
                let LackingOwnership(s) = LackingOwnership::parse(values);

                options.lacking_ownership = match HelpBehaviour::from_str(&s) {
                    Some(h) => h,
                    None => {
                        return Error::new(span, &format!("invalid help behaviour: {:?}", s))
                            .to_compile_error()
                            .into();
                    }
                };
            }
            "wrong_channel" => {
                let WrongChannel(s) = WrongChannel::parse(values);

                options.wrong_channel = match HelpBehaviour::from_str(&s) {
                    Some(h) => h,
                    None => {
                        return Error::new(span, &format!("invalid help behaviour: {:?}", s))
                            .to_compile_error()
                            .into();
                    }
                };
            }
            "embed_error_colour" => {
                let EmbedErrorColour(s) = EmbedErrorColour::parse(values);

                options.embed_error_colour = s;
            }
            "embed_success_colour" => {
                let EmbedSuccessColour(s) = EmbedSuccessColour::parse(values);

                options.embed_success_colour = s;
            }
            "max_levenshtein_distance" => {
                let MaxLevenshteinDistance(s) = MaxLevenshteinDistance::parse(values);

                options.max_levenshtein_distance = s;
            }
            _ => {
                return Error::new(span, &format!("invalid attribute: {:?}", name))
                    .to_compile_error()
                    .into();
            }
        }
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
    } = options;

    let strikethrough_commands_tip_in_dm = AsOption(strikethrough_commands_tip_in_dm);
    let strikethrough_commands_tip_in_guild = AsOption(strikethrough_commands_tip_in_guild);

    if let Err(err) = validate_declaration(&mut fun, true) {
        return err.to_compile_error().into();
    }

    if let Err(err) = validate_return_type(&mut fun) {
        return err.to_compile_error().into();
    }

    let options = fun.name.with_suffix(HELP_OPTIONS);

    let n = fun.name.with_suffix(HELP);
    let nn = fun.name.clone();
    let cfgs = fun.cfgs.clone();
    let cfgs2 = cfgs.clone();

    let cname = crate_name();
    let options_path = quote!(#cname::framework::standard::HelpOptions);
    let command_path = quote!(#cname::framework::standard::HelpCommand);

    let colour_path = quote!(#cname::utils::Colour);

    (quote! {
        #(#cfgs)*
        pub static #options: #options_path = #options_path {
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
            embed_error_colour: #colour_path(#embed_error_colour),
            embed_success_colour: #colour_path(#embed_success_colour),
            max_levenshtein_distance: #max_levenshtein_distance,
        };

        #(#cfgs2)*
        pub static #n: #command_path = #command_path {
            fun: #nn,
            options: &#options,
        };

        #fun
    })
    .into()
}

#[proc_macro]
pub fn group(input: TokenStream) -> TokenStream {
    let group = parse_macro_input!(input as Group);

    group.into_token_stream().into()
}

/// Create an instance of `GroupOptions`.
/// Useful when making default options and then deriving them for command groups.
///
/// ```rust,ignore
/// use command_attr::group_options;
///
/// // First argument is the name of the options; second the actual options.
/// group_options!("foobar", {
///     description: "I'm an example group",
/// });
/// ```
#[proc_macro]
pub fn group_options(input: TokenStream) -> TokenStream {
    struct GroupOptionsName {
        name: Ident,
        options: GroupOptions,
    }

    impl Parse for GroupOptionsName {
        fn parse(input: ParseStream) -> Result<Self> {
            let name = input.parse::<Ident>()?;

            let options = input.parse::<GroupOptions>()?;

            Ok(GroupOptionsName { name, options })
        }
    }

    let GroupOptionsName { name, options } = parse_macro_input!(input as GroupOptionsName);

    let name = name.with_suffix(GROUP_OPTIONS);

    let cname = crate_name();
    let options_path = quote!(#cname::framework::standard::GroupOptions);

    (quote! {
        pub static #name: #options_path = #options;
    })
    .into()
}
