use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta};
use proc_macro2::Span;

use crate::util::*;
use crate::OnlyIn;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueKind {
    // #[<name>]
    Name,

    // #[<name> = <value>]
    Equals,

    // #[<name>([<value>, ...])]
    List,

    // #[<name>(<value>)]
    SingleList,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueKind::Name => f.pad("`#[<name>]`"),
            ValueKind::Equals => f.pad("`#[<name> = <value>]`"),
            ValueKind::List => f.pad("`#[<name>([<value>, ...])]`"),
            ValueKind::SingleList => f.pad("`#[<name>(<value>)]`"),
        }
    }
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
        Meta::Word(name) => Ok(Values::new(name, ValueKind::Name, Vec::new(), attr.span())),
        Meta::List(MetaList {
            ident: name,
            paren_token: _,
            nested,
        }) => {
            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

            let mut lits = Vec::with_capacity(nested.len());

            for meta in nested {
                match meta {
                    NestedMeta::Literal(l) => lits.push(l),
                    NestedMeta::Meta(m) => match m {
                        Meta::Word(w) => lits.push(Lit::Str(LitStr::new(&w.to_string(), w.span()))),
                        Meta::List(_) => return Err(Error::new(attr.span(), "cannot nest a list")),
                        Meta::NameValue(_) => {
                            return Err(Error::new(attr.span(), "cannot nest a name-value pair"));
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
        Meta::NameValue(MetaNameValue {
            ident: name,
            eq_token: _,
            lit,
        }) => Ok(Values::new(name, ValueKind::Equals, vec![lit], attr.span())),
    }
}

struct DisplaySlice<'a, T: 'a>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            0 => f.write_str("nothing")?,
            1 => write!(f, "{}", (self.0)[0])?,
            _ => {
                let mut iter = self.0.iter().enumerate();

                if let Some((idx, item)) = iter.next() {
                    write!(f, "{}: {}", idx, item)?;

                    for (idx, item) in iter {
                        write!(f, "\n{}: {}", idx, item)?;
                    }
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
            format_args!("the attribute must be in of these forms:\n{}", DisplaySlice(forms)),
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

        let res = values.literals.into_iter().map(|lit| lit.to_str()).collect();

        Ok(res)
    }
}

impl AttributeOption for String {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Equals, ValueKind::SingleList])?;

        Ok(values.literals[0].to_str())
    }
}

impl AttributeOption for bool {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::Name, ValueKind::SingleList])?;

        Ok(if values.literals.is_empty() {
            true
        } else {
            values.literals[0].to_bool()
        })
    }
}

impl AttributeOption for Vec<Ident> {
    fn parse(values: Values) -> Result<Self> {
        validate(&values, &[ValueKind::List])?;

        Ok(values
            .literals
            .into_iter()
            .map(|s| Ident::new(&s.to_str(), Span::call_site()))
            .collect())
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
            match s.as_str() {
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

        let only = values.literals[0].to_str();
        let only = match &only[..] {
            "guilds" => OnlyIn::Guild,
            "dms" => OnlyIn::Dm,
            _ => panic!("invalid only type: {:?}", only),
        };

        Ok(only)
    }
}

macro_rules! attr_option_num {
    ($($n:ty),*) => {
        $(
            impl AttributeOption for $n {
                fn parse(values: Values) -> Result<Self> {
                    validate(&values, &[ValueKind::SingleList])?;

                    Ok(match &values.literals[0] {
                        Lit::Int(l) => l.value() as $n,
                        l => {
                            let s = l.to_str();
                            // We use `as_str()` here for forcing method resolution
                            // to choose `&str`'s `parse` method, not our trait's `parse` method.
                            // (`AttributeOption` is implemented for `String`)
                            s.as_str().parse().expect("invalid integer")
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
