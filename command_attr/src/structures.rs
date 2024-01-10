use std::str::FromStr;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    braced,
    Attribute,
    Block,
    Expr,
    ExprClosure,
    FnArg,
    Ident,
    Pat,
    Path,
    ReturnType,
    Stmt,
    Token,
    Type,
    Visibility,
};

use crate::consts::CHECK;
use crate::util::{self, Argument, AsOption, IdentExt2, Parenthesised};

#[derive(Debug, Default, Eq, PartialEq)]
pub enum OnlyIn {
    Dm,
    Guild,
    #[default]
    None,
}

impl OnlyIn {
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
            Self::Dm => stream.extend(quote!(#only_in_path::Dm)),
            Self::Guild => stream.extend(quote!(#only_in_path::Guild)),
            Self::None => stream.extend(quote!(#only_in_path::None)),
        }
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
                },
                Pat::Wild(wild) => {
                    let token = wild.underscore_token;

                    let name = Ident::new("_", token.spans[0]);

                    Ok(Argument {
                        mutable: None,
                        name,
                        kind: *kind,
                    })
                },
                _ => Err(Error::new(pat.span(), format_args!("unsupported pattern: {pat:?}"))),
            }
        },
        FnArg::Receiver(_) => {
            Err(Error::new(arg.span(), format_args!("`self` arguments are prohibited: {arg:?}")))
        },
    }
}

/// Test if the attribute is cooked.
fn is_cooked(attr: &Attribute) -> bool {
    const COOKED_ATTRIBUTE_NAMES: &[&str] =
        &["cfg", "cfg_attr", "derive", "inline", "allow", "warn", "deny", "forbid"];

    COOKED_ATTRIBUTE_NAMES.iter().any(|n| attr.path.is_ident(n))
}

pub fn is_rustfmt_or_clippy_attr(path: &Path) -> bool {
    path.segments.first().is_some_and(|s| s.ident == "rustfmt" || s.ident == "clippy")
}

/// Removes cooked attributes from a vector of attributes. Uncooked attributes are left in the
/// vector.
///
/// # Return
///
/// Returns a vector of cooked attributes that have been removed from the input vector.
fn remove_cooked(attrs: &mut Vec<Attribute>) -> Vec<Attribute> {
    let mut cooked = Vec::new();

    // FIXME: Replace with `Vec::drain_filter` once it is stable.
    let mut i = 0;
    while i < attrs.len() {
        if !is_cooked(&attrs[i]) && !is_rustfmt_or_clippy_attr(&attrs[i].path) {
            i += 1;
            continue;
        }

        cooked.push(attrs.remove(i));
    }

    cooked
}

#[derive(Debug)]
pub struct CommandFun {
    /// `#[...]`-style attributes.
    pub attributes: Vec<Attribute>,
    /// Populated cooked attributes. These are attributes outside of the realm of this crate's
    /// procedural macros and will appear in generated output.
    pub cooked: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub args: Vec<Argument>,
    pub ret: Type,
    pub body: Vec<Stmt>,
}

impl Parse for CommandFun {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut attributes = input.call(Attribute::parse_outer)?;

        // Rename documentation comment attributes (`#[doc = "..."]`) to `#[description = "..."]`.
        util::rename_attributes(&mut attributes, "doc", "description");

        let cooked = remove_cooked(&mut attributes);

        let visibility = input.parse::<Visibility>()?;

        input.parse::<Token![async]>()?;

        input.parse::<Token![fn]>()?;
        let name = input.parse()?;

        // (...)
        let Parenthesised(args) = input.parse::<Parenthesised<FnArg>>()?;

        let ret = match input.parse::<ReturnType>()? {
            ReturnType::Type(_, t) => (*t).clone(),
            ReturnType::Default => {
                return Err(input
                    .error("expected a result type of either `CommandResult` or `CheckResult`"))
            },
        };

        // { ... }
        let bcont;
        braced!(bcont in input);
        let body = bcont.call(Block::parse_within)?;

        let args = args.into_iter().map(parse_argument).collect::<Result<Vec<_>>>()?;

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
            attributes: _,
            cooked,
            visibility,
            name,
            args,
            ret,
            body,
        } = self;

        stream.extend(quote! {
            #(#cooked)*
            #visibility async fn #name (#(#args),*) -> #ret {
                #(#body)*
            }
        });
    }
}

#[derive(Debug)]
pub struct FunctionHook {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub args: Vec<Argument>,
    pub ret: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct ClosureHook {
    pub attributes: Vec<Attribute>,
    pub args: Punctuated<Pat, Token![,]>,
    pub ret: ReturnType,
    pub body: Box<Expr>,
}

#[derive(Debug)]
pub enum Hook {
    Function(Box<FunctionHook>),
    Closure(ClosureHook),
}

impl Parse for Hook {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        if is_function(input) {
            parse_function_hook(input, attributes).map(|h| Self::Function(Box::new(h)))
        } else {
            parse_closure_hook(input, attributes).map(Self::Closure)
        }
    }
}

fn is_function(input: ParseStream<'_>) -> bool {
    input.peek(Token![pub]) || (input.peek(Token![async]) && input.peek2(Token![fn]))
}

fn parse_function_hook(input: ParseStream<'_>, attributes: Vec<Attribute>) -> Result<FunctionHook> {
    let visibility = input.parse::<Visibility>()?;

    input.parse::<Token![async]>()?;
    input.parse::<Token![fn]>()?;

    let name = input.parse()?;

    // (...)
    let Parenthesised(args) = input.parse::<Parenthesised<FnArg>>()?;

    let ret = match input.parse::<ReturnType>()? {
        ReturnType::Type(_, t) => (*t).clone(),
        ReturnType::Default => {
            Type::Verbatim(TokenStream2::from_str("()").expect("Invalid str to create `()`-type"))
        },
    };

    // { ... }
    let bcont;
    braced!(bcont in input);
    let body = bcont.call(Block::parse_within)?;

    let args = args.into_iter().map(parse_argument).collect::<Result<Vec<_>>>()?;

    Ok(FunctionHook {
        attributes,
        visibility,
        name,
        args,
        ret,
        body,
    })
}

fn parse_closure_hook(input: ParseStream<'_>, attributes: Vec<Attribute>) -> Result<ClosureHook> {
    input.parse::<Token![async]>()?;
    let closure = input.parse::<ExprClosure>()?;

    Ok(ClosureHook {
        attributes,
        args: closure.inputs,
        ret: closure.output,
        body: closure.body,
    })
}

#[derive(Debug, Default)]
pub struct Permissions(pub u64);

impl Permissions {
    pub fn from_str(s: &str) -> Option<Self> {
        Some(Permissions(match s.to_uppercase().as_str() {
            "PRESET_GENERAL" => 0b0000_0110_0011_0111_1101_1100_0100_0001,
            "PRESET_TEXT" => 0b0000_0000_0000_0111_1111_1100_0100_0000,
            "PRESET_VOICE" => 0b0000_0011_1111_0000_0000_0000_0000_0000,
            "CREATE_INVITE" | "CREATE_INSTANT_INVITE" => 1 << 0,
            "KICK_MEMBERS" => 1 << 1,
            "BAN_MEMBERS" => 1 << 2,
            "ADMINISTRATOR" => 1 << 3,
            "MANAGE_CHANNELS" => 1 << 4,
            "MANAGE_GUILD" => 1 << 5,
            "ADD_REACTIONS" => 1 << 6,
            "VIEW_AUDIT_LOG" => 1 << 7,
            "PRIORITY_SPEAKER" => 1 << 8,
            "STREAM" => 1 << 9,
            "VIEW_CHANNEL" => 1 << 10,
            "SEND_MESSAGES" => 1 << 11,
            "SEND_TTS_MESSAGES" => 1 << 12,
            "MANAGE_MESSAGES" => 1 << 13,
            "EMBED_LINKS" => 1 << 14,
            "ATTACH_FILES" => 1 << 15,
            "READ_MESSAGE_HISTORY" => 1 << 16,
            "MENTION_EVERYONE" => 1 << 17,
            "USE_EXTERNAL_EMOJIS" => 1 << 18,
            "VIEW_GUILD_INSIGHTS" => 1 << 19,
            "CONNECT" => 1 << 20,
            "SPEAK" => 1 << 21,
            "MUTE_MEMBERS" => 1 << 22,
            "DEAFEN_MEMBERS" => 1 << 23,
            "MOVE_MEMBERS" => 1 << 24,
            "USE_VAD" => 1 << 25,
            "CHANGE_NICKNAME" => 1 << 26,
            "MANAGE_NICKNAMES" => 1 << 27,
            "MANAGE_ROLES" => 1 << 28,
            "MANAGE_WEBHOOKS" => 1 << 29,
            "MANAGE_EMOJIS_AND_STICKERS" | "MANAGE_GUILD_EXPRESSIONS" => 1 << 30,
            "USE_SLASH_COMMANDS" | "USE_APPLICATION_COMMANDS" => 1 << 31,
            "REQUEST_TO_SPEAK" => 1 << 32,
            "MANAGE_EVENTS" => 1 << 33,
            "MANAGE_THREADS" => 1 << 34,
            "CREATE_PUBLIC_THREADS" => 1 << 35,
            "CREATE_PRIVATE_THREADS" => 1 << 36,
            "USE_EXTERNAL_STICKERS" => 1 << 37,
            "SEND_MESSAGES_IN_THREADS" => 1 << 38,
            "USE_EMBEDDED_ACTIVITIES" => 1 << 39,
            "MODERATE_MEMBERS" => 1 << 40,
            "VIEW_CREATOR_MONETIZATION_ANALYTICS" => 1 << 41,
            "USE_SOUNDBOARD" => 1 << 42,
            "CREATE_GUILD_EXPRESSIONS" => 1 << 43,
            "CREATE_EVENTS" => 1 << 44,
            "USE_EXTERNAL_SOUNDS" => 1 << 45,
            "SEND_VOICE_MESSAGES" => 1 << 46,
            "SET_VOICE_CHANNEL_STATUS" => 1 << 48,
            _ => return None,
        }))
    }
}

impl ToTokens for Permissions {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let bits = self.0;

        let path = quote!(serenity::model::permissions::Permissions::from_bits_truncate);

        stream.extend(quote! {
            #path(#bits)
        });
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Colour(pub u32);

impl Colour {
    pub fn from_str(s: &str) -> Option<Self> {
        let hex = match s.to_uppercase().as_str() {
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
            _ => {
                let s = s.strip_prefix('#')?;

                if s.len() != 6 {
                    return None;
                }

                u32::from_str_radix(s, 16).ok()?
            },
        };

        Some(Colour(hex))
    }
}

impl ToTokens for Colour {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let value = self.0;
        let path = quote!(serenity::model::Colour);

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
    pub fn new() -> Self {
        Self {
            help_available: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
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
            Self::Strike => stream.extend(quote!(#help_behaviour_path::Strike)),
            Self::Hide => stream.extend(quote!(#help_behaviour_path::Hide)),
            Self::Nothing => stream.extend(quote!(#help_behaviour_path::Nothing)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct HelpOptions {
    pub suggestion_text: String,
    pub no_help_available_text: String,
    pub usage_label: String,
    pub usage_sample_label: String,
    pub ungrouped_label: String,
    pub description_label: String,
    pub grouped_label: String,
    pub aliases_label: String,
    pub sub_commands_label: String,
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
    pub lacking_conditions: HelpBehaviour,
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
            guild_only_text: "Only in servers".to_string(),
            checks_label: "Checks".to_string(),
            sub_commands_label: "Sub Commands".to_string(),
            dm_only_text: "Only in DM".to_string(),
            dm_and_guild_text: "In DM and servers".to_string(),
            available_text: "Available".to_string(),
            command_not_found_text: "**Error**: Command `{}` not found.".to_string(),
            individual_command_tip: "To get help with an individual command, pass its \
                                     name as an argument to this command."
                .to_string(),
            group_prefix: "Prefix".to_string(),
            strikethrough_commands_tip_in_dm: None,
            strikethrough_commands_tip_in_guild: None,
            lacking_role: HelpBehaviour::Strike,
            lacking_permissions: HelpBehaviour::Strike,
            lacking_ownership: HelpBehaviour::Hide,
            lacking_conditions: HelpBehaviour::Strike,
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
        let mut attributes = input.call(Attribute::parse_outer)?;

        util::rename_attributes(&mut attributes, "doc", "description");

        let cooked = remove_cooked(&mut attributes);

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
    pub summary: AsOption<String>,
    pub commands: Vec<Ident>,
    pub sub_groups: Vec<Ident>,
}

impl GroupOptions {
    pub fn new() -> Self {
        Self {
            help_available: true,
            ..Default::default()
        }
    }
}
