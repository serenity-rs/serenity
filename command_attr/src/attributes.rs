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
                    write!(f, "{}: {}", idx, item)?;
                }
            }
        }

        Ok(())
    }
}

pub trait AttributeOption: Sized {
    const NAME: &'static str;
    const ACCEPTED_FORMS: &'static [ValueKind];

    fn validate(vals: &Values) {
        assert!(
            vals.name == Self::NAME,
            "expected attribute name to be {:?}",
            Self::NAME
        );

        let is_accepted = if Self::ACCEPTED_FORMS.contains(&ValueKind::List)
            && vals.kind == ValueKind::SingleList
        {
            true
        } else {
            Self::ACCEPTED_FORMS.contains(&vals.kind)
        };

        assert!(
            is_accepted,
            "expected attribute {:?} to be in one of the forms \n{}",
            Self::NAME,
            DisplaySlice(Self::ACCEPTED_FORMS)
        );
    }

    fn parse(vals: Values) -> Self;
}

macro_rules! define_attribute_options {
    (list_of_strings => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub Vec<String>);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::List];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    let res = vals.literals.into_iter().map(|lit| lit.to_str()).collect();

                    $name(res)
                }
            }
        )*
    };
    (string => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub String);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::Equals, ValueKind::SingleList];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    $name(vals.literals[0].to_str())
                }
            }
        )*
    };
    (number/$type:ty => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub $type);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::SingleList];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    $name(
                        if let Lit::Int(l) = &vals.literals[0] {
                            l.value() as $type
                        } else {
                            vals.literals[0].to_str().parse().expect("invalid integer")
                        }
                    )
                }
            }
        )*
    };
    (boolean => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub bool);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::Name, ValueKind::SingleList];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    $name(
                        if vals.literals.is_empty() {
                            true
                        } else {
                            vals.literals[0].to_bool()
                        }
                    )
                }
            }
        )*
    };
    (list_of_idents => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub Vec<Ident>);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::List];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    $name(
                        vals.literals
                        .into_iter()
                        .map(|s| Ident::new(&s.to_str(), Span::call_site()))
                        .collect(),
                    )
                }
            }
        )*
    };
    (boolean_or_string => [ $($name:ident, $n:expr);* ]) => {
        $(
            #[derive(Debug)]
            pub struct $name(pub Option<String>);

            impl AttributeOption for $name {
                const NAME: &'static str = $n;
                const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::Name, ValueKind::SingleList];

                fn parse(vals: Values) -> Self {
                    Self::validate(&vals);

                    $name(
                        if vals.literals.is_empty() {
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
                        }
                    )
                }
            }
        )*
    };
}

#[derive(Debug)]
pub struct Only(pub(crate) OnlyIn);

impl AttributeOption for Only {
    const NAME: &'static str = "only_in";
    const ACCEPTED_FORMS: &'static [ValueKind] = &[ValueKind::SingleList];

    fn parse(vals: Values) -> Self {
        Self::validate(&vals);

        let only = vals.literals[0].to_str();
        let only = match &only[..] {
            "guilds" => OnlyIn::Guild,
            "dms" => OnlyIn::Dm,
            _ => panic!("invalid only type: {:?}", only),
        };

        Only(only)
    }
}

define_attribute_options!(list_of_strings => [
    Aliases, "aliases";
    AllowedRoles, "allowed_roles"
]);

define_attribute_options!(string => [
    Description, "description";
    Usage, "usage"
]);

define_attribute_options!(number/u8 => [
    MinArgs, "min_args";
    MaxArgs, "max_args";
    NumArgs, "num_args"
]);

define_attribute_options!(boolean => [
    HelpAvailable, "help_available";
    OwnerPrivilege, "owner_privilege";
    OwnersOnly, "owners_only"
]);

define_attribute_options!(list_of_idents => [
    AChecks, "checks";
    SubCommands, "sub";
    RequiredPermissions, "required_permissions"
]);

// For the help command
define_attribute_options!(string => [
    SuggestionText, "suggestion_text";
    NoHelpAvailableText, "no_help_available_text";
    UsageLabel, "usage_label";
    UsageSampleLabel, "usage_sample_label";
    UngroupedLabel, "ungrouped_label";
    GroupedLabel, "grouped_label";
    DescriptionLabel, "description_label";
    AliasesLabel, "aliases_label";
    GuildOnlyText, "guild_only_text";
    ChecksLabel, "checks_label";
    DmOnlyText, "dm_only_text";
    DmAndGuildText, "dm_and_guild_text";
    AvailableText, "available_text";
    CommandNotFoundText, "command_not_found_text";
    IndividualCommandTip, "individual_command_tip";
    GroupPrefix, "group_prefix";

    LackingRole, "lacking_role";
    LackingPermissions, "lacking_permissions";
    LackingOwnership, "lacking_ownership";
    WrongChannel, "wrong_channel"
]);

define_attribute_options!(number/u32 => [
    EmbedErrorColour, "embed_error_colour";
    EmbedSuccessColour, "embed_success_colour"
]);

define_attribute_options!(number/usize => [
    MaxLevenshteinDistance, "max_levenshtein_distance"
]);

define_attribute_options!(boolean_or_string => [
    StrikeThroughCommandsTipInDm, "strikethrough_commands_tip_in_dm";
    StrikeThroughCommandsTipInGuild, "strikethrough_commands_tip_in_guild"
]);
