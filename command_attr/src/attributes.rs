use proc_macro2::Span;
use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Lit, LitStr, Meta, NestedMeta, Path};

use crate::structures::{Checks, Colour, HelpBehaviour, OnlyIn, Permissions};
use crate::util::{AsOption, LitExt};

use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueKind {
    // #[<name>]
    Name,

    // #[<name> = <value>]
    Equals,

    // #[<name>([<value>, <value>, <value>, ...])]
    List,

    // #[<name>(<value>)]
    SingleList,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Name => f.pad("`#[<name>]`"),
            ValueKind::Equals => f.pad("`#[<name> = <value>]`"),
            ValueKind::List => f.pad("`#[<name>([<value>, <value>, <value>, ...])]`"),
            ValueKind::SingleList => f.pad("`#[<name>(<value>)]`"),
        }
    }
}

fn to_ident(p: Path) -> Result<Ident> {
    if p.segments.is_empty() {
        return Err(Error::new(
            p.span(),
            "cannot convert an empty path to an identifier",
        ));
    }

    if p.segments.len() > 1 {
        return Err(Error::new(
            p.span(),
            "the path must not have more than one segment",
        ));
    }

    if !p.segments[0].arguments.is_empty() {
        return Err(Error::new(
            p.span(),
            "the singular path segment must not have any arguments",
        ));
    }

    Ok(p.segments[0].ident.clone())
}

#[derive(Debug)]
pub struct Values {
    pub name: Ident,
    pub literals: Vec<Lit>,
    pub kind: ValueKind,
    pub span: Span,
}

impl Values {
    #[inline]
    pub fn new(name: Ident, kind: ValueKind, literals: Vec<Lit>, span: Span) -> Self {
        Values {
            name,
            literals,
            kind,
            span,
        }
    }
}

pub fn parse_values(attr: &Attribute) -> Result<Values> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Path(path) => {
            let name = to_ident(path)?;

            Ok(Values::new(name, ValueKind::Name, Vec::new(), attr.span()))
        }
        Meta::List(meta) => {
            let name = to_ident(meta.path)?;
            let nested = meta.nested;

            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

            let mut lits = Vec::with_capacity(nested.len());

            for meta in nested {
                match meta {
                    NestedMeta::Lit(l) => lits.push(l),
                    NestedMeta::Meta(m) => match m {
                        Meta::Path(path) => {
                            let i = to_ident(path)?;
                            lits.push(Lit::Str(LitStr::new(&i.to_string(), i.span())))
                        }
                        Meta::List(_) | Meta::NameValue(_) => {
                            return Err(Error::new(attr.span(), "cannot nest a list; only accept literals and identifiers at this level"))
                        }
                    },
                }
            }

            let kind = if lits.len() == 1 {
                ValueKind::SingleList
            } else {
                ValueKind::List
            };

            Ok(Values::new(name, kind, lits, attr.span()))
        }
        Meta::NameValue(meta) => {
            let name = to_ident(meta.path)?;
            let lit = meta.lit;

            Ok(Values::new(name, ValueKind::Equals, vec![lit], attr.span()))
        }
    }
}

#[derive(Debug, Clone)]
struct DisplaySlice<'a, T>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter().enumerate();

        match iter.next() {
            None => f.write_str("nothing")?,
            Some((idx, elem)) => {
                write!(f, "{}: {}", idx, elem)?;

                for (idx, elem) in iter {
                    f.write_char('\n')?;
                    write!(f, "{}: {}", idx, elem)?;
                }
            }
        }

        Ok(())
    }
}

#[inline]
fn is_form_acceptable(expect: &[ValueKind], kind: ValueKind) -> bool {
    if expect.contains(&ValueKind::List) && kind == ValueKind::SingleList {
        true
    } else {
        expect.contains(&kind)
    }
}

#[inline]
fn validate(values: &Values, forms: &[ValueKind]) -> Result<()> {
    if !is_form_acceptable(forms, values.kind) {
        return Err(Error::new(
            values.span,
            // Using the `_args` version here to avoid an allocation.
            format_args!(
                "the attribute must be in of these forms:\n{}",
                DisplaySlice(forms)
            ),
        ));
    }

    Ok(())
}

#[inline]
pub fn parse<T: AttributeOption>(values: Values) -> Result<T> {
    T::parse(values)
}

pub trait AttributeOption: Sized {
    fn parse(values: Values) -> Result<Self>;
}

impl AttributeOption for Vec<String> {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::List])?;

        Ok(values
            .literals
            .into_iter()
            .map(|lit| lit.to_str())
            .collect())
    }
}

impl AttributeOption for String {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Equals, ValueKind::SingleList])?;

        Ok(values.literals[0].to_str())
    }
}

impl AttributeOption for bool {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Name, ValueKind::SingleList])?;

        Ok(values.literals.get(0).map_or(true, |l| l.to_bool()))
    }
}

impl AttributeOption for Ident {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::SingleList])?;

        Ok(values.literals[0].to_ident())
    }
}

impl AttributeOption for Vec<Ident> {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::List])?;

        Ok(values.literals.into_iter().map(|l| l.to_ident()).collect())
    }
}

impl AttributeOption for Option<String> {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Name, ValueKind::SingleList])?;

        Ok(if values.literals.is_empty() {
            Some(String::new())
        } else if let Lit::Bool(b) = &values.literals[0] {
            if b.value {
                Some(String::new())
            } else {
                None
            }
        } else {
            let s = values.literals[0].to_str();
            match &s[..] {
                "true" => Some(String::new()),
                "false" => None,
                _ => Some(s),
            }
        })
    }
}

impl AttributeOption for OnlyIn {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::SingleList])?;

        let lit = &values.literals[0];

        OnlyIn::from_str(&lit.to_str()[..], lit.span())
    }
}

impl AttributeOption for Colour {
    fn parse(values: Values) -> Result<Self> {
        let span = values.span;
        let value = String::parse(values)?;

        Colour::from_str(&value)
            .ok_or_else(|| Error::new(span, format_args!("invalid colour: \"{}\"", value)))
    }
}

impl AttributeOption for HelpBehaviour {
    fn parse(values: Values) -> Result<Self> {
        let span = values.span;
        let value = String::parse(values)?;

        HelpBehaviour::from_str(&value)
            .ok_or_else(|| Error::new(span, format_args!("invalid help behaviour: \"{}\"", value)))
    }
}

impl AttributeOption for Checks {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        <Vec<Ident> as AttributeOption>::parse(values).map(Checks)
    }
}

impl AttributeOption for Permissions {
    fn parse(values: Values) -> Result<Self> {
        let perms = <Vec<Ident> as AttributeOption>::parse(values)?;

        let mut permissions = Permissions::default();
        for permission in perms {
            let p = match Permissions::from_str(&permission.to_string()) {
                Some(p) => p,
                None => return Err(Error::new(permission.span(), "invalid permission")),
            };

            permissions.0 |= p.0;
        }

        Ok(permissions)
    }
}

impl<T: AttributeOption> AttributeOption for AsOption<T> {
    #[inline]
    fn parse(values: Values) -> Result<Self> {
        Ok(AsOption(Some(T::parse(values)?)))
    }
}

macro_rules! attr_option_num {
    ($($n:ty),*) => {
        $(
            impl AttributeOption for $n {
                fn parse(values: Values) -> Result<Self> {
                    validate(&values, &[ValueKind::SingleList])?;

                    Ok(match &values.literals[0] {
                        Lit::Int(l) => l.base10_parse::<$n>()?,
                        l => {
                            let s = l.to_str();
                            // Use `as_str` to guide the compiler to use `&str`'s parse method.
                            // We don't want to use our `parse` method here (`impl AttributeOption for String`).
                            match s.as_str().parse::<$n>() {
                                Ok(n) => n,
                                Err(_) => return Err(Error::new(l.span(), "invalid integer")),
                            }
                        }
                    })
                }
            }

            impl AttributeOption for Option<$n> {
                #[inline]
                fn parse(values: Values) -> Result<Self> {
                    <$n as AttributeOption>::parse(values).map(Some)
                }
            }
        )*
    }
}

attr_option_num!(u16, u32, usize);
