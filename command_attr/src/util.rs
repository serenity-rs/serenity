use crate::structures::CommandFun;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Error, Parse, ParseStream, Result as SynResult},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Mut},
    Ident, Lit, Type,
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
        Ident::new(&self.to_str(), self.span())
    }
}

pub trait IdentExt2: Sized {
    fn to_uppercase(&self) -> Self;
    fn with_suffix(&self, suf: &str) -> Ident;
}

impl IdentExt2 for Ident {
    #[inline]
    fn to_uppercase(&self) -> Self {
        format_ident!("{}", self.to_string().to_uppercase())
    }

    #[inline]
    fn with_suffix(&self, suffix: &str) -> Ident {
        format_ident!("{}_{}", self.to_string().to_uppercase(), suffix)
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
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let content;
        bracketed!(content in input);

        Ok(Bracketed(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct Braced<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Braced<T> {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let content;
        braced!(content in input);

        Ok(Braced(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct Parenthesised<T>(pub Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesised<T> {
    fn parse(input: ParseStream<'_>) -> SynResult<Self> {
        let content;
        parenthesized!(content in input);

        Ok(Parenthesised(content.parse_terminated(T::parse)?))
    }
}

#[derive(Debug)]
pub struct AsOption<T>(pub Option<T>);

impl<T> AsOption<T> {
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> AsOption<U> {
        AsOption(self.0.map(f))
    }
}

impl<T: ToTokens> ToTokens for AsOption<T> {
    fn to_tokens(&self, stream: &mut TokenStream2) {
        match &self.0 {
            Some(o) => stream.extend(quote!(Some(#o))),
            None => stream.extend(quote!(None)),
        }
    }
}

impl<T> Default for AsOption<T> {
    #[inline]
    fn default() -> Self {
        AsOption(None)
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

#[inline]
pub fn generate_type_validation(have: Type, expect: Type) -> syn::Stmt {
    parse_quote! {
        serenity::static_assertions::assert_type_eq_all!(#have, #expect);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarFor {
    Command,
    Help,
    Check,
}

pub fn create_declaration_validations(fun: &mut CommandFun, dec_for: DeclarFor) -> SynResult<()> {
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

    let context: Type = parse_quote!(&mut serenity::client::Context);
    let message: Type = parse_quote!(&serenity::model::channel::Message);
    let args: Type = parse_quote!(serenity::framework::standard::Args);
    let args2: Type = parse_quote!(&mut serenity::framework::standard::Args);
    let options: Type = parse_quote!(&serenity::framework::standard::CommandOptions);
    let hoptions: Type = parse_quote!(&'static serenity::framework::standard::HelpOptions);
    let groups: Type = parse_quote!(&[&'static serenity::framework::standard::CommandGroup]);
    let owners: Type = parse_quote!(std::collections::HashSet<serenity::model::id::UserId>);

    let mut index = 0;

    let mut spoof_or_check = |kind: Type, name: &str| {
        match fun.args.get(index) {
            Some(x) => fun.body.insert(0, generate_type_validation(x.kind.clone(), kind)),
            None => fun.args.push(Argument {
                mutable: None,
                name: Ident::new(name, Span::call_site()),
                kind,
            }),
        }

        index += 1;
    };

    spoof_or_check(context, "_ctx");
    spoof_or_check(message, "_msg");

    if dec_for == DeclarFor::Check {
        spoof_or_check(args2, "_args");
        spoof_or_check(options, "_options");

        return Ok(());
    }

    spoof_or_check(args, "_args");

    if dec_for == DeclarFor::Help {
        spoof_or_check(hoptions, "_hoptions");
        spoof_or_check(groups, "_groups");
        spoof_or_check(owners, "_owners");
    }

    Ok(())
}

#[inline]
pub fn create_return_type_validation(r#fn: &mut CommandFun, expect: Type) {
    let stmt = generate_type_validation(r#fn.ret.clone(), expect);
    r#fn.body.insert(0, stmt);
}
