use crate::crate_name;
use crate::structures::CommandFun;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed,
    parse::{Error, Parse, ParseBuffer, ParseStream, Result},
    parse_quote,
    spanned::Spanned,
    token::{Brace, Bracket, Mut},
    Ident, Lit, ReturnType, Token, Type,
};

pub trait LitExt {
    fn to_str(&self) -> String;
    fn to_bool(&self) -> bool;
    fn to_ident(&self) -> Ident;
}

impl LitExt for Lit {
    fn to_str(&self) -> String {
        match self {
            Lit::Str(s) => s.value(),
            Lit::ByteStr(s) => unsafe { String::from_utf8_unchecked(s.value()) },
            Lit::Char(c) => c.value().to_string(),
            Lit::Byte(b) => (b.value() as char).to_string(),
            _ => panic!("values must be a (byte)string or a char"),
        }
    }

    fn to_bool(&self) -> bool {
        if let Lit::Bool(b) = self {
            b.value
        } else {
            self.to_str()
                .parse()
                .unwrap_or_else(|_| panic!("expected bool from {:?}", self))
        }
    }

    #[inline]
    fn to_ident(&self) -> Ident {
        Ident::new(&self.to_str(), Span::call_site())
    }
}

pub trait IdentExt2: Sized {
    fn to_uppercase(&self) -> Self;
    fn with_suffix(&self, suf: &str) -> Ident;
}

impl IdentExt2 for Ident {
    fn to_uppercase(&self) -> Self {
        let ident = self.to_string();
        let ident = ident.to_uppercase();

        Ident::new(&ident, Span::call_site())
    }

    fn with_suffix(&self, suf: &str) -> Ident {
        Ident::new(
            &format!("{}_{}", self.to_uppercase(), suf),
            Span::call_site(),
        )
    }
}

pub trait ParseStreamExt {
    /// Do not advance unless the parse was successful.
    fn try_parse<T: Parse>(&self) -> Result<T>;
}

impl<'a> ParseStreamExt for ParseBuffer<'a> {
    fn try_parse<T: Parse>(&self) -> Result<T> {
        let stream = self.fork();
        let res = T::parse(&stream);
        if res.is_ok() {
            T::parse(self)?;
        }

        res
    }
}

pub struct AsOption<T>(pub Option<T>);

impl<T: ToTokens> ToTokens for AsOption<T> {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        match &self.0 {
            Some(o) => stream.extend(quote!(Some(#o))),
            None => stream.extend(quote!(None)),
        }
    }
}

#[derive(Debug)]
pub struct Argument {
    pub mutable: Option<Mut>,
    pub name: Ident,
    pub kind: Type,
}

impl ToTokens for Argument {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        let Argument {
            mutable,
            name,
            kind,
        } = self;

        stream.extend(quote! {
            #mutable #name: #kind
        });
    }
}

#[derive(Debug)]
pub struct Field<T> {
    pub name: Ident,
    pub value: T,
}

impl<T: Parse> Parse for Field<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;

        let value = input.parse()?;

        Ok(Field { name, value })
    }
}

#[derive(Debug)]
pub enum RefOrInstance<T> {
    Ref(Ident),
    Instance(T),
}

impl<T: Parse> Parse for RefOrInstance<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) {
            Ok(RefOrInstance::Ref(input.parse()?))
        } else {
            Ok(RefOrInstance::Instance(input.parse()?))
        }
    }
}

impl<T: Default> Default for RefOrInstance<T> {
    #[inline]
    fn default() -> Self {
        RefOrInstance::Instance(T::default())
    }
}

#[derive(Debug)]
pub struct IdentAccess(pub Ident, pub Option<Ident>);

impl Parse for IdentAccess {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        let name2 = if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;

            Some(input.parse()?)
        } else {
            None
        };

        Ok(IdentAccess(name, name2))
    }
}

#[derive(Debug)]
pub enum Expr {
    Lit(Lit),
    Access(IdentAccess),
    Object(Object),
    Array(Array),
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Brace) {
            Ok(Expr::Object(input.parse()?))
        } else if input.peek(Bracket) {
            Ok(Expr::Array(input.parse()?))
        } else if let Ok(access) = input.try_parse::<IdentAccess>() {
            Ok(Expr::Access(access))
        } else {
            Ok(Expr::Lit(input.parse()?))
        }
    }
}

#[derive(Debug)]
pub struct Object(pub Vec<Field<Expr>>);

impl Parse for Object {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let input = content;

        let fields = input.parse_terminated::<_, Token![,]>(Field::<Expr>::parse)?;

        Ok(Object(fields.into_iter().collect()))
    }
}

#[derive(Debug)]
pub struct Array(pub Vec<Expr>);

impl Parse for Array {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        let input = content;

        let fields = input.parse_terminated::<_, Token![,]>(Expr::parse)?;

        Ok(Array(fields.into_iter().collect()))
    }
}

pub fn validate_declaration(fun: &mut CommandFun, is_help: bool) -> Result<()> {
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
    let owners: Type = parse_quote!(HashSet<UserId, impl BuildHasher>);

    let cname = crate_name();
    let context_path: Type = parse_quote!(&mut #cname::prelude::Context);
    let message_path: Type = parse_quote!(&#cname::model::channel::Message);
    let args_path: Type = parse_quote!(#cname::framework::standard::Args);
    let options_path: Type = parse_quote!(&'static #cname::framework::standard::HelpOptions);
    let groups_path: Type = parse_quote!(&[&'static #cname::framework::standard::CommandGroup]);
    let owners_path: Type = parse_quote!(std::collections::HashSet<#cname::model::id::UserId, std::hash::BuildHasher>);

    let ctx_error = "first argument's type should be `&mut Context`";
    let msg_error = "second argument's type should be `&Message`";
    let args_error = "third argument's type should be `Args`";
    let options_error = "fourth argument's type should be `&'static HelpOptions`";
    let groups_error = "fifth argument's type should be `&[&'static CommandGroup]`";
    let owners_error = "sixth argument's type should be `HashSet<UserId, impl BuildHasher>`";

    #[allow(unused_assignments)]
    macro_rules! spoof_or_check {
        ($(($($help:tt)?) [$($mut:tt)?] $type:ident, $name:literal, $error:ident, $path:ident);*) => {{
            macro_rules! arg {
                () => {
                    None
                };
                (mut) => {
                    Some(parse_quote!(mut))
                }
            }

            macro_rules! help {
                ($b:block) => {
                    $b
                };
                (help $b:block) => {
                    if is_help {
                        $b
                    }
                }
            }

            let mut index = 0;
            $(
                help!($($help)? {
                    match fun.args.get(index) {
                        Some(x) => {
                            if x.kind != $type {
                                return Err(Error::new(fun.args[index].span(), $error));
                            }
                        },
                        None => fun.args.push(Argument {
                            mutable: arg!($($mut)?),
                            name: Ident::new($name, Span::call_site()),
                            kind: $path,
                        }),
                    }
                });

                index += 1;
            )*
        }};
    }

    spoof_or_check! {
        ()     []    context, "_ctx", ctx_error, context_path;
        ()     []    message, "_msg", msg_error, message_path;
        ()     [mut] args, "_args", args_error, args_path;
        (help) []    options, "_options", options_error, options_path;
        (help) []    groups, "_groups", groups_error, groups_path;
        (help) []    owners, "_owners", owners_error, owners_path
    }

    Ok(())
}

pub fn validate_return_type(fun: &mut CommandFun) -> Result<()> {
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
