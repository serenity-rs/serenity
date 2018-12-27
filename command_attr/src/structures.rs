use crate::consts::{COMMAND, GROUP, GROUP_OPTIONS};
use crate::util::{
    Argument, Array, AsOption, Expr, Field, IdentAccess, IdentExt2, LitExt, Object, RefOrInstance,
};
use crate::crate_name;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Pub,
    ArgCaptured, Attribute, Block, FnArg, Ident, Lit, Pat, ReturnType, Stmt, Token,
};

#[derive(Debug, PartialEq)]
pub enum OnlyIn {
    Dm,
    Guild,
    None,
}

impl ToTokens for OnlyIn {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let cname = crate_name();
        let only_in_path = quote!(#cname::framework::standard::OnlyIn);
        match self {
            OnlyIn::Dm => stream.extend(quote!(#only_in_path::Dm)),
            OnlyIn::Guild => stream.extend(quote!(#only_in_path::Guild)),
            OnlyIn::None => stream.extend(quote!(#only_in_path::None)),
        }
    }
}

impl Default for OnlyIn {
    #[inline]
    fn default() -> Self {
        OnlyIn::None
    }
}

#[derive(Debug)]
pub struct CommandFun {
    pub _pub: Option<Pub>,
    pub cfgs: Vec<Attribute>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub args: Vec<Argument>,
    pub ret: ReturnType,
    pub body: Vec<Stmt>,
}

impl Parse for CommandFun {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let (cfgs, attributes): (Vec<_>, Vec<_>) = attributes
            .into_iter()
            // Omit doc comments.
            .filter(|a| !a.path.is_ident("doc"))
            // Split the cfgs from our attributes.
            .partition(|a| a.path.is_ident("cfg"));

        let _pub = if input.peek(Token![pub]) {
            Some(input.parse::<Token![pub]>()?)
        } else {
            None
        };

        input.parse::<Token![fn]>()?;
        let name = input.parse()?;

        // (....)
        let pcont;
        parenthesized!(pcont in input);
        let args: Punctuated<FnArg, Token![,]> = pcont.parse_terminated(FnArg::parse)?;

        let ret = if input.peek(Token![->]) {
            input.parse()?
        } else {
            return Err(Error::new(input.cursor().span(), "expected a return type"));
        };

        // { ... }
        let bcont;
        braced!(bcont in input);
        let body = bcont.call(Block::parse_within)?;

        let args: ::std::result::Result<Vec<Argument>, _> = args
            .into_iter()
            .map(|arg| {
                let span = arg.span();
                match arg {
                    FnArg::Captured(ArgCaptured {
                        pat,
                        colon_token: _,
                        ty: kind,
                    }) => {
                        let span = pat.span();
                        match pat {
                            Pat::Ident(id) => {
                                let name = id.ident;
                                let mutable = id.mutability;

                                Ok(Argument {
                                    mutable,
                                    name,
                                    kind,
                                })
                            }
                            Pat::Wild(wild) => {
                                let token = wild.underscore_token;

                                let name = Ident::new("_", token.spans[0]);

                                Ok(Argument {
                                    mutable: None,
                                    name,
                                    kind,
                                })
                            }
                            _ => Err(Error::new(span, &format!("unsupported pattern: {:?}", pat))),
                        }
                    }
                    _ => Err(Error::new(
                        span,
                        &format!("use of a prohibited argument type: {:?}", arg),
                    )),
                }
            })
            .collect();

        let args = args?;

        Ok(CommandFun {
            _pub,
            cfgs,
            attributes,
            name,
            args,
            ret,
            body,
        })
    }
}

impl ToTokens for CommandFun {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let CommandFun {
            _pub,
            cfgs,
            attributes: _,
            name,
            args,
            ret,
            body,
        } = self;

        stream.extend(quote! {
            #(#cfgs)*
            #_pub fn #name (#(#args),*) #ret {
                #(#body)*
            }
        });
    }
}

#[derive(Debug, Default)]
pub struct Permissions(pub u64);

impl Permissions {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(Permissions(match s {
            "PRESET_GENERAL" => 0b0000_0110_0011_0111_1101_1100_0100_0001,
            "PRESET_TEXT" => 0b0000_0000_0000_0111_1111_1100_0100_0000,
            "PRESET_VOICE" => 0b0000_0011_1111_0000_0000_0000_0000_0000,
            "CREATE_INVITE" => 0b0000_0000_0000_0000_0000_0000_0000_0001,
            "KICK_MEMBERS" => 0b0000_0000_0000_0000_0000_0000_0000_0010,
            "BAN_MEMBERS" => 0b0000_0000_0000_0000_0000_0000_0000_0100,
            "ADMINISTRATOR" => 0b0000_0000_0000_0000_0000_0000_0000_1000,
            "MANAGE_CHANNELS" => 0b0000_0000_0000_0000_0000_0000_0001_0000,
            "MANAGE_GUILD" => 0b0000_0000_0000_0000_0000_0000_0010_0000,
            "ADD_REACTIONS" => 0b0000_0000_0000_0000_0000_0000_0100_0000,
            "VIEW_AUDIT_LOG" => 0b0000_0000_0000_0000_0000_0000_1000_0000,
            "PRIORITY_SPEAKER" => 0b0000_0000_0000_0000_0000_0001_0000_0000,
            "READ_MESSAGES" => 0b0000_0000_0000_0000_0000_0100_0000_0000,
            "SEND_MESSAGES" => 0b0000_0000_0000_0000_0000_1000_0000_0000,
            "SEND_TTS_MESSAGES" => 0b0000_0000_0000_0000_0001_0000_0000_0000,
            "MANAGE_MESSAGES" => 0b0000_0000_0000_0000_0010_0000_0000_0000,
            "EMBED_LINKS" => 0b0000_0000_0000_0000_0100_0000_0000_0000,
            "ATTACH_FILES" => 0b0000_0000_0000_0000_1000_0000_0000_0000,
            "READ_MESSAGE_HISTORY" => 0b0000_0000_0000_0001_0000_0000_0000_0000,
            "MENTION_EVERYONE" => 0b0000_0000_0000_0010_0000_0000_0000_0000,
            "USE_EXTERNAL_EMOJIS" => 0b0000_0000_0000_0100_0000_0000_0000_0000,
            "CONNECT" => 0b0000_0000_0001_0000_0000_0000_0000_0000,
            "SPEAK" => 0b0000_0000_0010_0000_0000_0000_0000_0000,
            "MUTE_MEMBERS" => 0b0000_0000_0100_0000_0000_0000_0000_0000,
            "DEAFEN_MEMBERS" => 0b0000_0000_1000_0000_0000_0000_0000_0000,
            "MOVE_MEMBERS" => 0b0000_0001_0000_0000_0000_0000_0000_0000,
            "USE_VAD" => 0b0000_0010_0000_0000_0000_0000_0000_0000,
            "CHANGE_NICKNAME" => 0b0000_0100_0000_0000_0000_0000_0000_0000,
            "MANAGE_NICKNAMES" => 0b0000_1000_0000_0000_0000_0000_0000_0000,
            "MANAGE_ROLES" => 0b0001_0000_0000_0000_0000_0000_0000_0000,
            "MANAGE_WEBHOOKS" => 0b0010_0000_0000_0000_0000_0000_0000_0000,
            "MANAGE_EMOJIS" => 0b0100_0000_0000_0000_0000_0000_0000_0000,
            _ => return None,
        }))
    }
}

#[derive(Debug, Default)]
pub struct Checks(pub Vec<Ident>);

impl ToTokens for Checks {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let cname = crate_name();
        let checks_path = quote!(#cname::framework::standard::Check);

        let v = self.0.iter().map(|i| quote!(#checks_path { name: "", function: #i, check_in_help: true, display_in_help: true }));

        stream.extend(quote!(&[#(#v),*]));
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub checks: Checks,
    pub names: Vec<String>,
    pub desc: Option<String>,
    pub usage: Option<String>,
    pub min_args: Option<u8>,
    pub max_args: Option<u8>,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub sub: Vec<Ident>,
}

#[derive(PartialEq, Debug)]
pub enum HelpBehaviour {
    Strike,
    Hide,
    Nothing,
}

impl HelpBehaviour {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "strike" => HelpBehaviour::Strike,
            "hide" => HelpBehaviour::Hide,
            "nothing" => HelpBehaviour::Nothing,
            _ => return None,
        })
    }
}

impl ToTokens for HelpBehaviour {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let cname = crate_name();
        let help_behaviour_path = quote!(#cname::framework::standard::HelpBehaviour);
        match self {
            HelpBehaviour::Strike => stream.extend(quote!(#help_behaviour_path::Strike)),
            HelpBehaviour::Hide => stream.extend(quote!(#help_behaviour_path::Hide)),
            HelpBehaviour::Nothing => stream.extend(quote!(#help_behaviour_path::Nothing)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HelpOptions {
    pub suggestion_text: String,
    pub no_help_available_text: String,
    pub usage_label: String,
    pub usage_sample_label: String,
    pub ungrouped_label: String,
    pub description_label: String,
    pub grouped_label: String,
    pub aliases_label: String,
    pub guild_only_text: String,
    pub checks_label: String,
    pub dm_only_text: String,
    pub dm_and_guild_text: String,
    pub available_text: String,
    pub command_not_found_text: String,
    pub individual_command_tip: String,
    pub strikethrough_commands_tip_in_dm: Option<String>,
    pub strikethrough_commands_tip_in_guild: Option<String>,
    pub group_prefix: String,
    pub lacking_role: HelpBehaviour,
    pub lacking_permissions: HelpBehaviour,
    pub lacking_ownership: HelpBehaviour,
    pub wrong_channel: HelpBehaviour,
    pub embed_error_colour: u32,
    pub embed_success_colour: u32,
    pub max_levenshtein_distance: usize,
}

impl Default for HelpOptions {
    fn default() -> HelpOptions {
        HelpOptions {
            suggestion_text: "Did you mean `{}`?".to_string(),
            no_help_available_text: "**Error**: No help available.".to_string(),
            usage_label: "Usage".to_string(),
            usage_sample_label: "Sample usage".to_string(),
            ungrouped_label: "Ungrouped".to_string(),
            grouped_label: "Group".to_string(),
            aliases_label: "Aliases".to_string(),
            description_label: "Description".to_string(),
            guild_only_text: "Only in guilds".to_string(),
            checks_label: "Checks".to_string(),
            dm_only_text: "Only in DM".to_string(),
            dm_and_guild_text: "In DM and guilds".to_string(),
            available_text: "Available".to_string(),
            command_not_found_text: "**Error**: Command `{}` not found.".to_string(),
            individual_command_tip: "To get help with an individual command, pass its \
                                     name as an argument to this command."
                .to_string(),
            group_prefix: "Prefix".to_string(),
            strikethrough_commands_tip_in_dm: Some(String::new()),
            strikethrough_commands_tip_in_guild: Some(String::new()),
            lacking_role: HelpBehaviour::Strike,
            lacking_permissions: HelpBehaviour::Strike,
            lacking_ownership: HelpBehaviour::Hide,
            wrong_channel: HelpBehaviour::Strike,
            embed_error_colour: 0x992D22,   // DARK_RED
            embed_success_colour: 0xF6DBD8, // ROSEWATER
            max_levenshtein_distance: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct GroupOptions {
    pub prefixes: Vec<String>,
    pub only: OnlyIn,
    pub owner_only: bool,
    pub owner_privilege: bool,
    pub help_available: bool,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub checks: Checks,
    pub default_command: Option<Ident>,
    pub description: Option<String>,
    pub inherit: Option<IdentAccess>,
}

impl Parse for GroupOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let Object(fields) = input.parse::<Object>()?;

        let mut options = GroupOptions::default();

        options.help_available = true;
        options.owner_privilege = true;

        for Field { name, value } in fields {
            let span = name.span();
            let name = name.to_string();
            match (&name[..], value) {
                ("prefixes", Expr::Array(Array(values)))
                | ("allowed_roles", Expr::Array(Array(values))) => {
                    let values = values
                        .into_iter()
                        .map(|l| match l {
                            Expr::Lit(l) => Some(l.to_str()),
                            _ => None,
                        })
                        .collect::<Option<_>>();

                    let values = match values {
                        Some(values) => values,
                        None => return Err(Error::new(span, "expected a list of strings")),
                    };

                    if name == "prefixes" {
                        options.prefixes = values;
                    } else {
                        options.allowed_roles = values;
                    }
                }
                ("only", Expr::Lit(value)) => {
                    let span = value.span();
                    let value = value.to_str();

                    let only = match &value[..] {
                        "dms" => OnlyIn::Dm,
                        "guilds" => OnlyIn::Guild,
                        _ => return Err(Error::new(span, "invalid only option")),
                    };

                    options.only = only;
                }
                ("owner_only", Expr::Lit(value))
                | ("owner_privilege", Expr::Lit(value))
                | ("help_available", Expr::Lit(value)) => {
                    let b = value.to_bool();

                    if name == "owner_only" {
                        options.owner_only = b;
                    } else if name == "owner_privilege" {
                        options.owner_privilege = b;
                    } else {
                        options.help_available = b;
                    }
                }
                ("checks", Expr::Array(Array(arr))) | ("required_permissions", Expr::Array(Array(arr))) => {
                    let idents = arr
                        .into_iter()
                        .map(|l| match l {
                            Expr::Access(IdentAccess(l, None)) => Some(l),
                            _ => None,
                        })
                        .collect::<Option<_>>();

                    let idents = match idents {
                        Some(idents) => idents,
                        None => return Err(Error::new(span, "invalid value, expected ident")),
                    };

                    if name == "checks" {
                        options.checks = Checks(idents);
                    } else {
                        let mut permissions = Permissions::default();
                        for perm in idents {
                            let p = match Permissions::from_str(&perm.to_string()) {
                                Some(p) => p,
                                None => return Err(Error::new(perm.span(), "invalid permission")),
                            };

                            // Add them together.
                            permissions.0 |= p.0;
                        }

                        options.required_permissions = permissions;
                    }
                }
                ("default_command", Expr::Access(IdentAccess(re, _))) => {
                    options.default_command = Some(re);
                }
                ("prefix", Expr::Lit(s)) | ("description", Expr::Lit(s)) => {
                    let s = s.to_str();

                    if name == "prefix" {
                        options.prefixes = vec![s];
                    } else {
                        options.description = Some(s);
                    }
                }
                ("inherit", Expr::Access(access)) => {
                    options.inherit = Some(access);
                }
                (name, _) => {
                    return Err(Error::new(
                        span,
                        &format!("`{}` is not a valid group option", name),
                    ));
                }
            }
        }
        Ok(options)
    }
}

impl ToTokens for GroupOptions {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let GroupOptions {
            prefixes,
            allowed_roles,
            required_permissions,
            owner_privilege,
            owner_only,
            help_available,
            only,
            description,
            checks,
            default_command,
            inherit,
        } = self;

        let description = AsOption(description.clone());
        let mut dc = quote! { None };

        if let Some(cmd) = default_command {
            let cmd = cmd.with_suffix(COMMAND);
            dc = quote! {
                Some(&#cmd)
            };
        }

        let cname = crate_name();
        let options_path = quote!(#cname::framework::standard::GroupOptions);
        let permissions_path = quote!(#cname::model::permissions::Permissions);
        let required_permissions = required_permissions.0;

        if let Some(IdentAccess(from, its)) = inherit {
            let inherit = match its {
                Some(its) => {
                    if its != "options" {
                        *stream = Error::new(its.span(), "field being accessed is not `options`")
                            .to_compile_error();
                        return;
                    }
                    let from = from.with_suffix(GROUP);
                    quote! { *(#from).#its }
                }
                None => {
                    let from = from.with_suffix(GROUP_OPTIONS);
                    quote! { #from }
                }
            };

            let description = if description.0.is_some() {
                quote! { description: #description, }
            } else {
                quote!()
            };

            let prefixes = if !prefixes.is_empty() {
                quote! { prefixes: &[#(#prefixes),*], }
            } else {
                quote!()
            };

            let allowed_roles = if !allowed_roles.is_empty() {
                quote! { allowed_roles: &[#(#allowed_roles),*], }
            } else {
                quote!()
            };

            let required_permissions = if required_permissions != 0 {
                quote! { required_permissions: #permissions_path { bits: #required_permissions }, }
            } else {
                quote!()
            };

            let owner_privilege = if !owner_privilege {
                quote! { owner_privilege: #owner_privilege, }
            } else {
                quote!()
            };

            let owner_only = if *owner_only {
                quote! { owner_only: #owner_only, }
            } else {
                quote!()
            };

            let help_available = if !help_available {
                quote! { help_available: #help_available, }
            } else {
                quote!()
            };

            let only = if *only != OnlyIn::None {
                quote! { only: #only, }
            } else {
                quote!()
            };

            let checks = if !checks.0.is_empty() {
                quote! { checks: #checks, }
            } else {
                quote!()
            };

            let default_command = if default_command.is_some() {
                quote! { default_command: #dc, }
            } else {
                quote!()
            };

            stream.extend(quote! {
                #options_path {
                    #prefixes
                    #allowed_roles
                    #required_permissions
                    #owner_privilege
                    #owner_only
                    #help_available
                    #only
                    #description
                    #checks
                    #default_command
                    ..#inherit
                }
            });
        } else {
            stream.extend(quote! {
                #options_path {
                    prefixes: &[#(#prefixes),*],
                    allowed_roles: &[#(#allowed_roles),*],
                    required_permissions: #permissions_path { bits: #required_permissions },
                    owner_privilege: #owner_privilege,
                    owners_only: #owner_only,
                    help_available: #help_available,
                    only: #only,
                    description: #description,
                    checks: #checks,
                    default_command: #dc,
                }
            });
        }
    }
}

#[derive(Debug)]
pub struct Group {
    pub name: Ident,
    pub options: RefOrInstance<GroupOptions>,
    pub commands: Punctuated<Ident, Token![,]>,
    pub sub: Vec<RefOrInstance<Group>>,
}

impl Parse for Group {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let input = content;

        let Field { name, value } = input.parse::<Field<Lit>>()?;
        if name != "name" {
            return Err(Error::new(name.span(), "first key needs to be `name`"));
        }

        let name = value.to_ident();

        input.parse::<Token![,]>()?;

        let Field {
            name: n,
            value: options,
        } = input.parse::<Field<RefOrInstance<GroupOptions>>>()?;
        if n != "options" {
            return Err(Error::new(n.span(), "second key needs to be `options`"));
        }

        input.parse::<Token![,]>()?;

        let commands = input.parse::<Ident>()?;
        if commands != "commands" {
            return Err(Error::new(
                commands.span(),
                "third key needs to be `commands`",
            ));
        }

        input.parse::<Token![:]>()?;

        let content;
        bracketed!(content in input);

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let mut sub = Vec::new();

        if let Ok(s) = input.parse::<Ident>() {
            if s != "sub" {
                return Err(Error::new(s.span(), "fourth key needs to be `sub`"));
            }

            input.parse::<Token![:]>()?;

            let content;
            bracketed!(content in input);

            let refs: Punctuated<_, Token![,]> = content.parse_terminated(RefOrInstance::parse)?;

            sub.extend(refs.into_iter());
        }

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Group {
            name,
            options,
            commands: content.parse_terminated(Ident::parse_any)?,
            sub,
        })
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Group {
            name,
            options: opts,
            commands,
            sub,
        } = self;

        let commands = commands.into_iter().map(|cmd| {
            let cmd = cmd.with_suffix(COMMAND);
            quote! {
                &#cmd
            }
        });

        let sub = sub
            .into_iter()
            .map(|group| match group {
                RefOrInstance::Instance(group) => {
                    let name = group.name.with_suffix(GROUP);
                    stream.extend(group.into_token_stream());

                    name
                }
                RefOrInstance::Ref(name) => name.with_suffix(GROUP),
            })
            .collect::<Vec<_>>();

        let mut group_ops = name.with_suffix(GROUP_OPTIONS);
        let n = name.to_string();
        let name = name.with_suffix(GROUP);

        let mut options = None;
        match opts {
            RefOrInstance::Ref(name) => group_ops = name.with_suffix(GROUP_OPTIONS),
            RefOrInstance::Instance(opt) => options = Some(opt),
        }

        let cname = crate_name();
        let options_path = quote!(#cname::framework::standard::GroupOptions);
        let group_path = quote!(#cname::framework::standard::CommandGroup);

        if options.is_some() {
            stream.extend(quote! {
                pub static #group_ops: #options_path = #options;
            });
        }

        stream.extend(quote! {
            pub static #name: #group_path = #group_path {
                name: #n,
                options: &#group_ops,
                commands: &[#(#commands),*],
                sub: &[#(&#sub),*]
            };
        });
    }
}
