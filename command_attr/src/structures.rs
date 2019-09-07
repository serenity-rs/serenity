use crate::consts::CHECK;
use crate::util::{Argument, AsOption, IdentExt2, Parenthesised};
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Error, Parse, ParseStream, Result},
    spanned::Spanned,
    Attribute, Block, FnArg, Ident, Pat, ReturnType, Stmt, Token, Type, Visibility,
};

#[derive(Debug, PartialEq)]
pub enum OnlyIn {
    Dm,
    Guild,
    None,
}

impl OnlyIn {
    #[inline]
    pub fn from_str(s: &str, span: Span) -> Result<Self> {
        match s {
            "guilds" | "guild" => Ok(OnlyIn::Guild),
            "dms" | "dm" => Ok(OnlyIn::Dm),
            _ => Err(Error::new(span, "invalid restriction type")),
        }
    }
}

impl ToTokens for OnlyIn {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let only_in_path = quote!(serenity::framework::standard::OnlyIn);
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

fn parse_argument(arg: FnArg) -> Result<Argument> {
    match arg {
        FnArg::Typed(typed) => {
            let pat = typed.pat;
            let kind = typed.ty;

            match *pat {
                Pat::Ident(id) => {
                    let name = id.ident;
                    let mutable = id.mutability;

                    Ok(Argument {
                        mutable,
                        name,
                        kind: *kind,
                    })
                }
                Pat::Wild(wild) => {
                    let token = wild.underscore_token;

                    let name = Ident::new("_", token.spans[0]);

                    Ok(Argument {
                        mutable: None,
                        name,
                        kind: *kind,
                    })
                }
                _ => Err(Error::new(
                    pat.span(),
                    format_args!("unsupported pattern: {:?}", pat),
                )),
            }
        }
        FnArg::Receiver(_) => Err(Error::new(
            arg.span(),
            format_args!("`self` arguments are prohibited: {:?}", arg),
        )),
    }
}

#[derive(Debug)]
pub struct CommandFun {
    /// `#[...]`-style attributes.
    pub attributes: Vec<Attribute>,
    /// Populated by either `#[cfg(...)]` or `#[doc = "..."]` (the desugared form of doc-comments) type of attributes.
    pub cooked: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub args: Vec<Argument>,
    pub ret: Type,
    pub body: Vec<Stmt>,
}

impl Parse for CommandFun {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let (cooked, attributes): (Vec<_>, Vec<_>) = attributes
            .into_iter()
            .partition(|a| a.path.is_ident("cfg") || a.path.is_ident("doc"));

        let visibility = input.parse::<Visibility>()?;

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

        let args = args
            .into_iter()
            .map(parse_argument)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            attributes,
            cooked,
            visibility,
            name,
            args,
            ret,
            body,
        })
    }
}

impl ToTokens for CommandFun {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Self {
            cooked,
            attributes: _,
            visibility,
            name,
            args,
            ret,
            body,
        } = self;

        stream.extend(quote! {
            #(#cooked)*
            #visibility fn #name (#(#args),*) -> #ret {
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

impl ToTokens for Permissions {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let bits = self.0;

        let path = quote!(serenity::model::permissions::Permissions);

        stream.extend(quote! {
            #path { bits: #bits }
        });
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

impl ToTokens for Colour {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let value = self.0;
        let path = quote!(serenity::utils::Colour);

        stream.extend(quote! {
            #path(#value)
        });
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

#[derive(Debug, Default)]
pub struct Options {
    pub checks: Checks,
    pub bucket: AsOption<String>,
    pub aliases: Vec<String>,
    pub description: AsOption<String>,
    pub delimiters: Vec<String>,
    pub usage: AsOption<String>,
    pub examples: Vec<String>,
    pub min_args: AsOption<u16>,
    pub max_args: AsOption<u16>,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub help_available: bool,
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub sub_commands: Vec<Ident>,
}

impl Options {
    #[inline]
    pub fn new() -> Self {
        let mut options = Self::default();

        options.help_available = true;

        options
    }
}

#[derive(PartialEq, Debug)]
pub enum HelpBehaviour {
    Strike,
    Hide,
    Nothing,
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
pub struct GroupStruct {
    pub visibility: Visibility,
    pub cooked: Vec<Attribute>,
    pub attributes: Vec<Attribute>,
    pub name: Ident,
}

impl Parse for GroupStruct {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let (cooked, attributes): (Vec<_>, Vec<_>) = attributes
            .into_iter()
            .partition(|a| a.path.is_ident("cfg") || a.path.is_ident("doc"));

        let visibility = input.parse()?;

        input.parse::<Token![struct]>()?;

        let name = input.parse()?;

        input.parse::<Token![;]>()?;

        Ok(Self {
            visibility,
            cooked,
            attributes,
            name,
        })
    }
}

impl ToTokens for GroupStruct {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Self {
            visibility,
            cooked,
            attributes: _,
            name,
        } = self;

        stream.extend(quote! {
            #(#cooked)*
            #visibility struct #name;
        });
    }
}

#[derive(Debug, Default)]
pub struct GroupOptions {
    pub prefixes: Vec<String>,
    pub only_in: OnlyIn,
    pub owners_only: bool,
    pub owner_privilege: bool,
    pub help_available: bool,
    pub allowed_roles: Vec<String>,
    pub required_permissions: Permissions,
    pub checks: Checks,
    pub default_command: AsOption<Ident>,
    pub description: AsOption<String>,
    pub commands: Vec<Ident>,
    pub sub_groups: Vec<Ident>,
}

impl GroupOptions {
    #[inline]
    pub fn new() -> Self {
        let mut options = Self::default();

        options.help_available = true;

        options
    }
}
