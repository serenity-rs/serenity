use crate::consts::{CHECK, COMMAND, GROUP, GROUP_OPTIONS};
use crate::util::{
    Argument, Array, AsOption, Braced, Bracketed, BracketedIdents, Parenthesised, Expr, Field, IdentAccess,
    IdentExt2, LitExt, Object, RefOrInstance,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Pub,
    ArgCaptured, Attribute, Block, FnArg, Ident, Lit, Pat, Type, ReturnType, Stmt, Token,
};

#[derive(Debug, PartialEq)]
pub enum OnlyIn {
    Dm,
    Guild,
    None,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ToTokens for OnlyIn {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let only_in_path = quote!(serenity::framework::standard::OnlyIn);
        match self {
            OnlyIn::Dm => stream.extend(quote!(#only_in_path::Dm)),
            OnlyIn::Guild => stream.extend(quote!(#only_in_path::Guild)),
            OnlyIn::None => stream.extend(quote!(#only_in_path::None)),
            OnlyIn::__Nonexhaustive => unreachable!(),
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
    pub docs: Vec<Attribute>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub args: Vec<Argument>,
    pub ret: Type,
    pub body: Vec<Stmt>,
}

impl Parse for CommandFun {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let (cfgs, attributes): (Vec<_>, Vec<_>) = attributes
            .into_iter()
            // Split the cfgs from our attributes.
            .partition(|a| a.path.is_ident("cfg"));

        // Filter out doc-comments as well.
        let (docs, attributes) = attributes.into_iter().partition(|a| a.path.is_ident("doc"));

        let _pub = if input.peek(Token![pub]) {
            Some(input.parse::<Token![pub]>()?)
        } else {
            None
        };

        input.parse::<Token![fn]>()?;
        let name = input.parse()?;

        // (...)
        let Parenthesised(args) = input.parse::<Parenthesised<FnArg>>()?;

        let ret = match input.parse::<ReturnType>()? {
            ReturnType::Type(_, t) => (*t).clone(),
            ReturnType::Default => {
                return Err(input
                    .error("expected a result type of either `CommandResult` or `CheckResult`"))
            }
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
            docs,
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
            docs,
            attributes: _,
            name,
            args,
            ret,
            body,
        } = self;

        stream.extend(quote! {
            #(#cfgs)*
            #(#docs)*
            #_pub fn #name (#(#args),*) -> #ret {
                #(#body)*
            }
        });
    }
}

#[derive(Debug, Default)]
pub struct Permissions(pub u64);

impl Permissions {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(Permissions(match s.to_uppercase().as_str() {
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Colour(pub u32);

impl Colour {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(Colour(match s.to_uppercase().as_str() {
            "BLITZ_BLUE" => 0x6FC6E2,
            "BLUE" => 0x3498DB,
            "BLURPLE" => 0x7289DA,
            "DARK_BLUE" => 0x206694,
            "DARK_GOLD" => 0xC27C0E,
            "DARK_GREEN" => 0x1F8B4C,
            "DARK_GREY" => 0x607D8B,
            "DARK_MAGENTA" => 0xAD14757,
            "DARK_ORANGE" => 0xA84300,
            "DARK_PURPLE" => 0x71368A,
            "DARK_RED" => 0x992D22,
            "DARK_TEAL" => 0x11806A,
            "DARKER_GREY" => 0x546E7A,
            "FABLED_PINK" => 0xFAB81ED,
            "FADED_PURPLE" => 0x8882C4,
            "FOOYOO" => 0x11CA80,
            "GOLD" => 0xF1C40F,
            "KERBAL" => 0xBADA55,
            "LIGHT_GREY" => 0x979C9F,
            "LIGHTER_GREY" => 0x95A5A6,
            "MAGENTA" => 0xE91E63,
            "MEIBE_PINK" => 0xE68397,
            "ORANGE" => 0xE67E22,
            "PURPLE" => 0x9B59B6,
            "RED" => 0xE74C3C,
            "ROHRKATZE_BLUE" => 0x7596FF,
            "ROSEWATER" => 0xF6DBD8,
            "TEAL" => 0x1ABC9C,
            _ => return None,
        }))
    }
}

#[derive(Debug, Default)]
pub struct Checks(pub Vec<Ident>);

impl ToTokens for Checks {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let v = self.0.iter().map(|i| i.with_suffix(CHECK));

        stream.extend(quote!(&[#(&#v),*]));
    }
}

#[derive(Debug)]
pub struct Options {
    pub checks: Checks,
    pub bucket: Option<String>,
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub usage: Option<String>,
    pub example: Option<String>,
    pub min_args: Option<u16>,
    pub max_args: Option<u16>,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub sub_commands: Vec<Ident>,
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        Options {
            checks: Checks::default(),
            bucket: None,
            aliases: Vec::new(),
            description: None,
            usage: None,
            example: None,
            min_args: None,
            max_args: None,
            allowed_roles: Vec::new(),
            required_permissions: Permissions::default(),
            help_available: true,
            only_in: OnlyIn::default(),
            owners_only: false,
            owner_privilege: true,
            sub_commands: Vec::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum HelpBehaviour {
    Strike,
    Hide,
    Nothing,
    #[doc(hidden)]
    __Nonexhaustive,
}

impl HelpBehaviour {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s.to_lowercase().as_str() {
            "strike" => HelpBehaviour::Strike,
            "hide" => HelpBehaviour::Hide,
            "nothing" => HelpBehaviour::Nothing,
            _ => return None,
        })
    }
}

impl ToTokens for HelpBehaviour {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let help_behaviour_path = quote!(serenity::framework::standard::HelpBehaviour);
        match self {
            HelpBehaviour::Strike => stream.extend(quote!(#help_behaviour_path::Strike)),
            HelpBehaviour::Hide => stream.extend(quote!(#help_behaviour_path::Hide)),
            HelpBehaviour::Nothing => stream.extend(quote!(#help_behaviour_path::Nothing)),
            HelpBehaviour::__Nonexhaustive => unreachable!(),
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
    pub embed_error_colour: Colour,
    pub embed_success_colour: Colour,
    pub max_levenshtein_distance: usize,
    pub indention_prefix: String,
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
            embed_error_colour: Colour::from_str("DARK_RED").unwrap(),
            embed_success_colour: Colour::from_str("ROSEWATER").unwrap(),
            max_levenshtein_distance: 0,
            indention_prefix: "-".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct GroupOptions {
    pub prefixes: Vec<String>,
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub help_available: bool,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub checks: Checks,
    pub default_command: Option<Ident>,
    pub description: Option<String>,
    pub inherit: Option<IdentAccess>,
}

impl Default for GroupOptions {
    #[inline]
    fn default() -> Self {
        GroupOptions {
            prefixes: Vec::new(),
            only_in: OnlyIn::default(),
            owners_only: false,
            owner_privilege: true,
            help_available: true,
            allowed_roles: Vec::new(),
            required_permissions: Permissions::default(),
            checks: Checks::default(),
            default_command: None,
            description: None,
            inherit: None,
        }
    }
}

impl Parse for GroupOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let Object(fields) = input.parse::<Object>()?;

        let mut options = GroupOptions::default();

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
                        .collect::<Option<Vec<String>>>();

                    let values = match values {
                        Some(values) => values,
                        None => return Err(Error::new(span, "expected a list of strings")),
                    };

                    if name == "prefixes" {
                        for v in &values {
                            if v.is_empty() {
                                return Err(Error::new(span, "prefixes cannot be empty"));
                            }
                        }

                        options.prefixes = values;
                    } else {
                        options.allowed_roles = values;
                    }
                }
                ("only_in", Expr::Lit(value)) => {
                    let span = value.span();
                    let value = value.to_str();

                    let only = match &value[..] {
                        "dms" => OnlyIn::Dm,
                        "guilds" => OnlyIn::Guild,
                        _ => return Err(Error::new(span, "invalid only option")),
                    };

                    options.only_in = only;
                }
                ("owners_only", Expr::Lit(value))
                | ("owner_privilege", Expr::Lit(value))
                | ("help_available", Expr::Lit(value)) => {
                    let b = value.to_bool();

                    if name == "owners_only" {
                        options.owners_only = b;
                    } else if name == "owner_privilege" {
                        options.owner_privilege = b;
                    } else {
                        options.help_available = b;
                    }
                }
                ("checks", Expr::Array(Array(arr)))
                | ("required_permissions", Expr::Array(Array(arr))) => {
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
                        if s.is_empty() {
                            return Err(Error::new(s.span(), "prefixes cannot be empty"));
                        }

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
            owners_only,
            help_available,
            only_in,
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

        let options_path = quote!(serenity::framework::standard::GroupOptions);
        let permissions_path = quote!(serenity::model::permissions::Permissions);
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

            let owners_only = if *owners_only {
                quote! { owners_only: #owners_only, }
            } else {
                quote!()
            };

            let help_available = if !help_available {
                quote! { help_available: #help_available, }
            } else {
                quote!()
            };

            let only_in = if *only_in != OnlyIn::None {
                quote! { only_in: #only_in, }
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
                    #owners_only
                    #help_available
                    #only_in
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
                    owners_only: #owners_only,
                    help_available: #help_available,
                    only_in: #only_in,
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
    pub help_name: String,
    pub name: Ident,
    pub options: RefOrInstance<GroupOptions>,
    pub commands: Punctuated<Ident, Token![,]>,
    pub sub_groups: Vec<RefOrInstance<Group>>,
}

impl Parse for Group {
    fn parse(input: ParseStream) -> Result<Self> {
        enum GroupField {
            HelpName(String),
            Name(Ident),
            Options(RefOrInstance<GroupOptions>),
            Commands(BracketedIdents),
            SubGroups(Bracketed<RefOrInstance<Group>>),
        }

        impl Parse for GroupField {
            fn parse(input: ParseStream) -> Result<Self> {
                let name = input.parse::<Ident>()?;

                input.parse::<Token![:]>()?;

                match name.to_string().as_str() {
                    "help_name" => Ok(GroupField::HelpName(input.parse::<Lit>()?.to_str())),
                    "name" => Ok(GroupField::Name(input.parse::<Lit>()?.to_ident())),
                    "options" => Ok(GroupField::Options(input.parse()?)),
                    "commands" => Ok(GroupField::Commands(input.parse()?)),
                    "sub_groups" => Ok(GroupField::SubGroups(input.parse()?)),
                    n => Err(input.error(format_args!(
                    "`{}` is not an acceptable field of the `group!` macro.
                    Perhaps you meant one these fields? `name` / `options` / `commands` / `sub_groups`", n))),
                }
            }
        }

        let Braced(fields) = input.parse::<Braced<GroupField>>()?;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        let mut name = None;
        let mut help_name = None;
        let mut options = None;
        let mut commands = None;
        let mut sub_groups = None;

        for field in fields {
            match field {
                GroupField::Name(n) => name = Some(n),
                GroupField::HelpName(n) => help_name = Some(n),
                GroupField::Options(o) => options = Some(o),
                GroupField::Commands(BracketedIdents(p)) => commands = Some(p),
                GroupField::SubGroups(Bracketed(p)) => sub_groups = Some(p.into_iter().collect()),
            }
        }

        let name = match name {
            Some(n) => n,
            None => return Err(input.error("every group must have a `name`")),
        };

        let commands = match commands {
            Some(c) => c,
            None => return Err(input.error("every group must have some runnable `commands`")),
        };

        Ok(Group {
            help_name: help_name.unwrap_or_else(|| name.to_string()),
            name,
            commands,
            options: options.unwrap_or_default(),
            sub_groups: sub_groups.unwrap_or_else(Vec::new),
        })
    }
}

impl ToTokens for Group {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Group {
            help_name,
            name,
            options: opts,
            commands,
            sub_groups,
        } = self;

        let commands = commands.into_iter().map(|cmd| {
            let cmd = cmd.with_suffix(COMMAND);
            quote! {
                &#cmd
            }
        });

        let sub_groups = sub_groups
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

        let options_path = quote!(serenity::framework::standard::GroupOptions);
        let group_path = quote!(serenity::framework::standard::CommandGroup);

        if options.is_some() {
            stream.extend(quote! {
                pub static #group_ops: #options_path = #options;
            });
        }

        stream.extend(quote! {
            pub static #name: #group_path = #group_path {
                help_name: #help_name,
                name: #n,
                options: &#group_ops,
                commands: &[#(#commands),*],
                sub_groups: &[#(&#sub_groups),*]
            };
        });
    }
}
