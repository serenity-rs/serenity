use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream, Result},
    token::{Brace, Bracket, Mut},
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

pub trait ParseStreamExt {
    /// Do not advance unless the parse was successful.
    fn try_parse<T: Parse>(&self) -> Result<T>;
}

impl<'a> ParseStreamExt for ParseStream<'a> {
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
