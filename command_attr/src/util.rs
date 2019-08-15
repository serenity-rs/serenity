use crate::structures::CommandFun;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Error, Parse, ParseStream, Result},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Bracket, Comma, Mut},
    Ident, Lit, Token, Type,
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

#[inline]
pub fn into_stream(e: Error) -> TokenStream {
    e.to_compile_error().into()
}

macro_rules! propagate_err {
    ($res:expr) => {{
        match $res {
            Ok(v) => v,
            Err(e) => return $crate::util::into_stream(e),
        }
    }};
}

#[derive(Debug)]
pub struct Bracketed<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Bracketed<T> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        bracketed!(content in input);

        Ok(Bracketed(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct Braced<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Braced<T> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        braced!(content in input);

        Ok(Braced(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct Parenthesised<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesised<T> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        Ok(Parenthesised(content.parse_terminated(T::parse)?))
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
    fn parse(input: ParseStream<'_>) -> Result<Self> {
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
    fn parse(input: ParseStream<'_>) -> Result<Self> {
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

#[derive(Debug)]
pub enum Expr {
    Lit(Lit),
    Access(IdentAccess),
    Object(Object),
    Array(Array),
}

impl Parse for Expr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Brace) {
            Ok(Expr::Object(input.parse()?))
        } else if input.peek(Bracket) {
            Ok(Expr::Array(input.parse()?))
        } else {
            // Because boolean literals are converted into `Ident` tokens,
            // we must be more vigilant with how we parse the `IdentAccess` structure and `Lit`s.
            let access = input.step(|cursor| match cursor.ident() {
                Some((ref i, mut cursor)) if i != "true" && i != "false" => {
                    let i2 = match cursor.punct() {
                        Some((ref p, c)) if p.as_char() == '.' => c.ident().map(|(i2, c)| {
                            cursor = c;

                            i2
                        }),
                        _ => None,
                    };

                    return Ok((IdentAccess(i.clone(), i2), cursor));
                }
                _ => Err(cursor.error("...")),
            });

            match access {
                Ok(access) => Ok(Expr::Access(access)),
                Err(_) => Ok(Expr::Lit(input.parse()?)),
            }
        }
    }
}

#[derive(Debug)]
pub struct Object(pub Vec<Field<Expr>>);

impl Parse for Object {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let Braced(fields) = input.parse::<Braced<Field<Expr>>>()?;

        Ok(Object(fields.into_iter().collect()))
    }
}

#[derive(Debug)]
pub struct Array(pub Vec<Expr>);

impl Parse for Array {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let Bracketed(fields) = input.parse::<Bracketed<Expr>>()?;

        Ok(Array(fields.into_iter().collect()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarFor {
    Command,
    Help,
    Check,
}

pub fn validate_declaration(fun: &mut CommandFun, dec_for: DeclarFor) -> Result<()> {
    let len = match dec_for {
        DeclarFor::Command => 3,
        DeclarFor::Help => 6,
        DeclarFor::Check => 4,
    };

    if fun.args.len() > len {
        return Err(Error::new(
            fun.args.last().unwrap().span(),
            format_args!("function's arity exceeds more than {} arguments", len),
        ));
    }

    let context: Type = parse_quote!(&mut Context);
    let message: Type = parse_quote!(&Message);
    let args: Type = parse_quote!(Args);
    let args2: Type = parse_quote!(&mut Args);
    let options: Type = parse_quote!(&CommandOptions);
    let hoptions: Type = parse_quote!(&'static HelpOptions);
    let groups: Type = parse_quote!(&[&'static CommandGroup]);
    let owners: Type = parse_quote!(HashSet<UserId>);

    let context_path: Type = parse_quote!(&mut serenity::prelude::Context);
    let message_path: Type = parse_quote!(&serenity::model::channel::Message);
    let args_path: Type = parse_quote!(serenity::framework::standard::Args);
    let args2_path: Type = parse_quote!(&mut serenity::framework::standard::Args);
    let options_path: Type = parse_quote!(&'static serenity::framework::standard::CommandOptions);
    let hoptions_path: Type = parse_quote!(&'static serenity::framework::standard::HelpOptions);
    let groups_path: Type = parse_quote!(&[&'static serenity::framework::standard::CommandGroup]);
    let owners_path: Type = parse_quote!(std::collections::HashSet<serenity::model::id::UserId, std::hash::BuildHasher>);

    let ctx_error = "first argument's type should be `&mut Context`";
    let msg_error = "second argument's type should be `&Message`";
    let args_error = "third argument's type should be `Args`";
    let args2_error = "third argument's type should be `&mut Args`";
    let options_error = "fourth argument's type should be `&'static CommandOptions`";
    let hoptions_error = "fourth argument's type should be `&'static HelpOptions`";
    let groups_error = "fifth argument's type should be `&[&'static CommandGroup]`";
    let owners_error = "sixth argument's type should be `HashSet<UserId>`";

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
                    if dec_for == DeclarFor::Help {
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

            let _ = index;
        }};
    }

    if dec_for != DeclarFor::Check {
        spoof_or_check! {
            ()     []    context,  "_ctx",     ctx_error,      context_path;
            ()     []    message,  "_msg",     msg_error,      message_path;
            ()     [mut] args,     "_args",    args_error,     args_path;
            (help) []    hoptions, "_hoptions", hoptions_error, hoptions_path;
            (help) []    groups,   "_groups",  groups_error,   groups_path;
            (help) []    owners,   "_owners",  owners_error,   owners_path
        }
    } else {
        spoof_or_check! {
            ()     []    context, "_ctx",     ctx_error,     context_path;
            ()     []    message, "_msg",     msg_error,     message_path;
            ()     []    args2,   "_args",    args2_error,   args2_path;
            ()     []    options, "_options", options_error, options_path
        }
    }

    Ok(())
}

pub fn validate_return_type(fun: &mut CommandFun, [relative, absolute]: [Type; 2]) -> Result<()> {
    let ret = &fun.ret;

    if *ret == relative || *ret == absolute {
        return Ok(());
    }

    Err(Error::new(
        ret.span(),
        format_args!(
            "expected either `{}` or `{}` as the return type, but got `{}`",
            quote!(#relative),
            quote!(#absolute),
            quote!(#ret),
        ),
    ))
}
