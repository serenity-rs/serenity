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

// FIXME: God give me the strength I need to shorten this.
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
