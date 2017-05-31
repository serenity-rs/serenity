use quote::{Tokens, ToTokens};
use super::*;

macro_rules! generate_simple_function_impls {
    ($start:ident, $settings:ident: {$($name:ident;)*}) => {
        $(
            if let Some(ref $name) = $settings.$name {
                $start.append(&format!("fn {}(&self) -> String {{ \"{}\".to_string() }}", stringify!($name), $name));
            }
        )*
    }
}

#[derive(Debug, Clone)]
pub struct FnArgs {
    kind: FnArgsType,
    block: syn::Block,
}

#[derive(Debug, Clone)]
pub enum FnArgsType {
    Context(String),
    ContextMessage(String, String),
    ContextMessageArgs(String, String, String),
}

impl ToTokens for FnArgs {
    fn to_tokens(&self, tokens: &mut Tokens) {
        use self::FnArgsType::*;
        match self.kind {
            ContextMessageArgs(ref c, ref m, ref a) => tokens.append(format!("fn exec(&self, {}: &mut _serenity::client::Context, {}: &_serenity::model::Message, {}: Vec<String>) -> Result<(), String> {{ {} Ok(()) }}", c, m, a, quote!(#self.block).as_ref())),
            ContextMessage(ref c, ref m) => tokens.append(format!("fn exec(&self, {}: &mut _serenity::client::Context, {}: &_serenity::model::Message, _: Vec<String>) -> Result<(), String> {{ {} Ok(()) }}", c, m, quote!(#self.block).as_ref())),
            Context(ref c) => tokens.append(format!("fn exec(&self, {}: &mut _serenity::client::Context, _: &_serenity::model::Message, _: Vec<String>) -> Result<(), String> {{ {} Ok(()) }}", c, quote!(#self.block).as_ref())),
        }
    }
}

impl FnArgs {
    pub fn new(kind: FnArgsType, block: syn::Block) -> Self {
        FnArgs {
            kind,
            block,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Only {
    Guild,
    Dm,
    Owner,
}

#[derive(Debug, Clone)]
pub enum Bucket {
    Simple(i64),
    Complex(i64, i64, i32),
}

impl ToTokens for Bucket {
    fn to_tokens(&self, tokens: &mut Tokens) {
        use self::Bucket::*;
        match *self {
            Simple(delay) => tokens.append(&format!("Bucket {{ ratelimit: Ratelimit {{ delay: {}, limit: None, }}, users: HashMap::new(), }}", delay)),
            Complex(delay, limit, timespan) => tokens.append(&format!("Bucket {{ ratelimit: Ratelimit {{ delay: {}, limit: Some(({}, {})), }}, users: HashMap::new(), }}", delay, timespan, limit)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Checks(pub Vec<String>);

impl ToTokens for Checks {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append("vec![");
        let len = self.0.len();
        for (index, checkname) in self.0.iter().enumerate() {
            if index == len {
                tokens.append(&format!("Box::new({})", checkname));
                break;
            }
            tokens.append(&format!("Box::new({}),", checkname));
        }
        tokens.append("]");
    }
}

#[derive(Debug, Clone)]
pub struct Aliases(pub Vec<String>);

impl ToTokens for Aliases {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append("vec![");
        let len = self.0.len();
        for (index, checkname) in self.0.iter().enumerate() {
            if index == len {
                tokens.append(&format!(r#""{}""#, checkname));
                break;
            }
            tokens.append(&format!(r#""{}","#, checkname));
        }
        tokens.append("]");
    }
}

#[derive(Debug, Clone)]
pub enum Args {
    Max(i32),
    Min(i32),
    MaxAndMin(i32, i32),
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub checks: Option<Checks>,
    pub aliases: Option<Aliases>,
    pub bucket: Option<Bucket>,
    pub only: Option<Only>,
    pub group: Option<(String, String)>,
    pub description: Option<String>,
    pub example: Option<String>,
    pub usage: Option<String>,
    pub permissions: u64,
    pub help_available: bool,
    pub args: Option<Args>,
}

impl Default for Settings {
    #[allow(unconditional_recursion)]
    fn default() -> Self {
        Settings {
            help_available: true,
            ..Default::default()
        }
    }
}

impl ToTokens for Settings {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if let Some(ref checks) = self.checks {
            tokens.append(quote!(fn checks(&self) -> Vec<Box<Check>> { #checks }));
        }
        if let Some(ref aliases) = self.aliases {
            tokens.append(quote!(fn aliases(&self) -> Vec<String> { #aliases }));
        }
        if let Some(ref bucket) = self.bucket {
            tokens.append(quote!(fn bucket(&self) -> Option<Bucket> { Some(#bucket) }));
        }
        if let Some(ref only) = self.only {
            tokens.append(match *only {
                Only::Dm => "fn dm_only(&self) -> bool { true }",
                Only::Guild => "fn guild_only(&self) -> bool { true }",
                Only::Owner => "fn owners_only(&self) -> bool { true }",
            });
        }
        generate_simple_function_impls! {
            tokens, self: {
                description;
                example;
                usage;
            }
        }
        if self.permissions != 0 {
            tokens.append(quote!(fn required_permissions(&self) -> _serenity::model::Permissions { #self.permissions }));
        }
        if !self.help_available {
             tokens.append(quote!(fn help_available(&self) -> bool { #self.help_available }));
        }
        if let Some(ref args) = self.args {
            match *args {
                Args::Min(min) => tokens.append(format!("fn min_args(&self) -> i32 {{ {} }}", min)),
                Args::Max(max) => tokens.append(format!("fn max_args(&self) -> i32 {{ {} }}", max)),
                Args::MaxAndMin(max, min) => tokens.append(format!("fn min_args(&self) -> i32 {{ {} }} fn max_args(&self) -> i32 {{ {} }}", min, max)),
            }
        }
        if let Some((ref group, ref prefix)) = self.group {
            tokens.append(quote! {
                fn group(&self) -> String { format!("{}", #group) }
                fn group_prefix(&self) -> String { format!("{}", #prefix) }
            });
        }
    }
}

impl Settings {
    pub fn parse_attr(&mut self, attr: &syn::Attribute) -> Result<(), String> {
        match attr.name() {
            "group" => self.group = Some(parse_group(attr)),
            "checks" => self.checks = Some(Checks(parse_checks(attr))),
            "bucket" => self.bucket = Some(parse_bucket(attr)),
            "description" => self.description = Some(parse_description(attr)),
            "example" => self.example = Some(parse_example(attr)),
            "usage" => self.usage = Some(parse_usage(attr)),
            "args" => self.args = Some(parse_args(attr)),
            "permissions" => self.permissions = parse_permissions(attr),
            "help_available" => self.help_available = parse_help_available(attr),
            "only" => self.only = Some(parse_only(attr)),
            "aliases" => self.aliases = Some(Aliases(parse_aliases(attr))),
            _ => return Err("Invalid attribute".to_string()),
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct StringResponse(pub Option<String>);

impl ToTokens for StringResponse {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.0.as_ref().map(|ref sr| {
            tokens.append(quote! {
                fn exec(&self, _: &mut _serenity::client::Context, m: &_serenity::model::Message, _: Vec<String>) -> Result<(), String> { 
                    m.channel_id.say(&format!("{}", #sr)); 
                    Ok(()) 
                }
            });
        });
        
    }
}

pub fn to_pascal_case(st: &str) -> String {
    fn first_c_to_uppercase(ch: &str) -> String {
        let mut v = ch.chars().collect::<Vec<char>>();
        v[0] = v[0].to_uppercase().nth(0).unwrap();
        v.into_iter().collect()
    }
    first_c_to_uppercase(&st
        .replace('_', " ")
        .split_whitespace()
        .map(|s| first_c_to_uppercase(s))
        .collect::<String>())
}