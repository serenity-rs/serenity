use crate::util::*;
use crate::OnlyIn;
use proc_macro2::Span;
use std::fmt;
use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta};

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
}

impl Values {
    #[inline]
    pub fn new(name: Ident, kind: ValueKind, literals: Vec<Lit>) -> Self {
        Values {
            name,
            literals,
            kind,
        }
    }
}

pub fn parse_values(attr: &Attribute) -> Result<Values> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Word(name) => Ok(Values::new(name, ValueKind::Name, Vec::new())),
        Meta::List(MetaList {
            ident: name,
            paren_token: _,
            nested,
        }) => {
            let mut lits = Vec::new();

            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

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

            Ok(Values::new(name, kind, lits))
        }
        Meta::NameValue(MetaNameValue {
            ident: name,
            eq_token: _,
            lit,
        }) => Ok(Values::new(name, ValueKind::Equals, vec![lit])),
    }
}

struct DisplaySlice<'a, T: 'a>(&'a [T]);

impl<'a, T: fmt::Display> fmt::Display for DisplaySlice<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            0 => f.write_str("nothing")?,
            1 => write!(f, "{}", (self.0)[0])?,
            _ => {
                for (idx, item) in self.0.iter().enumerate() {
                    writeln!(f, "{}: {}", idx, item)?;
                }
            }
        }

        Ok(())
    }
}

pub trait AttributeOption: Sized {
    const FORMS: &'static [ValueKind];

    fn validate(vals: &Values, name: &str) {
        assert_eq!(vals.name, name, "expected attribute name to be {:?}", name);

        let is_accepted =
            if Self::FORMS.contains(&ValueKind::List) && vals.kind == ValueKind::SingleList {
                true
            } else {
                Self::FORMS.contains(&vals.kind)
            };

        assert!(
            is_accepted,
            "expected attribute {:?} to be in one of the forms \n{}",
            name,
            DisplaySlice(Self::FORMS)
        );
    }

    fn parse(&mut self, name: &str, vals: Values);
}

impl AttributeOption for Vec<String> {
    const FORMS: &'static [ValueKind] = &[ValueKind::List];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);

        let res = vals.literals.into_iter().map(|lit| lit.to_str()).collect();

        *self = res;
    }
}

impl AttributeOption for String {
    const FORMS: &'static [ValueKind] = &[ValueKind::Equals, ValueKind::SingleList];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);

        *self = vals.literals[0].to_str();
    }
}

impl AttributeOption for bool {
    const FORMS: &'static [ValueKind] = &[ValueKind::Name, ValueKind::SingleList];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);

        *self = if vals.literals.is_empty() {
            true
        } else {
            vals.literals[0].to_bool()
        };
    }
}

impl AttributeOption for Vec<Ident> {
    const FORMS: &'static [ValueKind] = &[ValueKind::List];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);
        *self = vals
            .literals
            .into_iter()
            .map(|s| Ident::new(&s.to_str(), Span::call_site()))
            .collect();
    }
}

impl AttributeOption for Option<String> {
    const FORMS: &'static [ValueKind] = &[ValueKind::Name, ValueKind::SingleList];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);

        *self = if vals.literals.is_empty() {
            Some(String::new())
        } else if let Lit::Bool(b) = &vals.literals[0] {
            if b.value {
                Some(String::new())
            } else {
                None
            }
        } else {
            let s = vals.literals[0].to_str();
            match s.as_str() {
                "true" => Some(String::new()),
                "false" => None,
                _ => Some(s),
            }
        };
    }
}

impl AttributeOption for OnlyIn {
    const FORMS: &'static [ValueKind] = &[ValueKind::SingleList];

    fn parse(&mut self, name: &str, vals: Values) {
        Self::validate(&vals, name);

        let only = vals.literals[0].to_str();
        let only = match &only[..] {
            "guilds" => OnlyIn::Guild,
            "dms" => OnlyIn::Dm,
            _ => panic!("invalid only type: {:?}", only),
        };

        *self = only;
    }
}

macro_rules! attr_option_num {
    ($($n:ty),*) => {
        $(
            impl AttributeOption for $n {
                const FORMS: &'static [ValueKind] = &[ValueKind::SingleList];

                fn parse(&mut self, name: &str, vals: Values) {
                    Self::validate(&vals, name);

                    *self = if let Lit::Int(l) = &vals.literals[0] {
                        l.value() as $n
                    } else {
                        let s = vals.literals[0].to_str();
                        // We use `as_str()` here for forcing method resolution
                        // to choose `&str`'s `parse` method, not our trait's `parse` method.
                        // (`AttributeOption` is implemented for `String`)
                        s.as_str().parse().expect("invalid integer")
                    };
                }
            }

            impl AttributeOption for Option<$n> {
                const FORMS: &'static [ValueKind] = &[ValueKind::SingleList];

                fn parse(&mut self, name: &str, vals: Values) {
                    Self::validate(&vals, name);

                    let mut n: $n = 0;
                    n.parse(name, vals);

                    *self = Some(n);
                }
            }
        )*
    }
}

attr_option_num!(u16, u32, usize);
