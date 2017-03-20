use super::*;

macro_rules! generate_simple_attributes {
    ($($name:ident;)*) => {
        $(
            pub fn $name(attr: &syn::Attribute) -> String {
                let mut val = "".to_string();
                match attr.value {
                    syn::MetaItem::List(_, ref vec) => {
                        for lit in vec {
                            if let syn::NestedMetaItem::Literal(syn::Lit::Str(ref li, _)) = *lit {
                                val = li.clone();
                            }
                        }  
                    },
                    syn::MetaItem::NameValue(_, syn::Lit::Str(ref lit, _)) => val = lit.clone(),
                    _ => {},
                }
                val
            }
        )*
    }
}

pub fn parse_fn_args(inputs: &[syn::FnArg], block: syn::Block) -> FnArgs {
    let (mut context_name, mut message_name, mut args_name) = (None, None, None);
    for arg in inputs {
        if let syn::FnArg::Captured(ref pat, ref ty) = *arg {
            let mut typ = "".to_string();
            if let syn::Ty::Path(_, ref pa) = *ty {
                if pa.segments.iter().find(|ref a| a.ident.as_ref() == "Context" || a.ident.as_ref() == "Message").is_some() 
                    && (pa.segments[0].ident.as_ref() != "Context" || pa.segments[0].ident.as_ref() != "Message")  {
                    assert_eq!(pa.segments[0].ident.as_ref(), "serenity");
                }
                for segment in &pa.segments {
                    typ = (match segment.ident.as_ref() {
                        "Context" => "Context",
                        "Message" => "Message",
                        "Vec<string>" => "Vec<String>",
                        _ => continue,
                    }).to_string();
                }
            }
            if let syn::Pat::Ref(ref p, _) = *pat {
                if let syn::Pat::Path(_, ref pa) = **p {
                    let sa = pa.segments[0].ident.clone().as_ref().to_string();
                    if sa == "_" {
                        continue
                    }
                    match sa.clone() {
                        ref s if typ == "Context" => context_name = Some(s.clone()),
                        ref s if typ == "Message" => message_name = Some(s.clone()),
                        ref s if typ == "Vec<String>" => args_name = Some(s.clone()),
                        _ => {}
                    }
                }
            }
        }
    }

    match (context_name, message_name, args_name) {
        (Some(c), Some(m), Some(a)) => FnArgs::new(FnArgsType::ContextMessageArgs(c, m, a), block),
        (Some(c), Some(m), None) => FnArgs::new(FnArgsType::ContextMessage(c, m), block),
        (Some(c), None, None) => FnArgs::new(FnArgsType::Context(c), block),
        _ => unreachable!()
    }
}

pub fn parse_bucket(attr: &syn::Attribute) -> Bucket {
    fn set_bucket_field(nested_items: &[syn::NestedMetaItem], index: usize, field: &mut isize) {
        if let syn::NestedMetaItem::Literal(syn::Lit::Int(ref lit, _)) = *nested_items.get(index).unwrap() {
            *field = *lit as isize;
        }
    }
    let (mut delay, mut limit, mut timespan) = (0, 0, 0);
    match attr.value {
        syn::MetaItem::Word(_) => panic!(r#"The "bucket" attribute requires the delay and/or the limit!"#),
        syn::MetaItem::List(_, ref vec) => {
            assert!(!vec.is_empty(), r#"The "bucket" attribute requires the delay and/or the limit!"#);
            if vec.len() == 1 {
                set_bucket_field(&vec, 1, &mut delay);
            } else if vec.len() <= 3 {
                set_bucket_field(&vec, 1, &mut delay);
                set_bucket_field(&vec, 1, &mut limit);
                set_bucket_field(&vec, 1, &mut timespan);
            }
        },
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref lit, _)) => {
            for l in lit.split(':') {
                if delay == 0 {
                    delay = l.parse().unwrap();
                } else if limit == 0 {
                    limit = l.parse().unwrap();
                } else if timespan == 0 {
                    timespan = l.parse().unwrap();
                }
            }
        },
        _ => {},
    }
    if delay != 0 {
        Bucket::Simple(delay as i64)
    } else {
        Bucket::Complex(delay as i64, limit as i64, timespan as i32)
    }
}

pub fn parse_checks(attr: &syn::Attribute) -> Vec<String> {
    let mut checks = Vec::new();
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let syn::NestedMetaItem::Literal(syn::Lit::Str(ref li, _)) = *lit {
                    checks.push(li.clone());
                }
            } 
         },
        syn::MetaItem::NameValue(_, syn::Lit::Str(ref lit, _)) => checks.push(lit.clone()),
        _ => {}
    }
    checks
}

generate_simple_attributes! {
    parse_description; 
    parse_example; 
    parse_usage;
}

pub fn parse_args(attr: &syn::Attribute) -> Args {
    let (mut min_args, mut max_args) = (0, 0);
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let &syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, syn::Lit::Int(li, _))) = lit {
                    if ident == "min" {
                        min_args = li;
                    } else if ident == "max" {
                        max_args = li;
                    }
                }
            }  
        },
        _ => {},
    }
    if min_args != 0 {
        Args::Min(min_args as i32)
    } else if max_args != 0 {
        Args::Max(max_args as i32)
    } else if min_args != 0 && max_args != 0 {
        Args::MaxAndMin(max_args as i32, min_args as i32)
    } else {
        unreachable!()
    }
}

pub fn parse_permissions(attr: &syn::Attribute) -> u64 {
    let mut perms = 0;
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let &syn::NestedMetaItem::Literal(syn::Lit::Int(ref li, _)) = lit {
                    perms = *li;
                }
            }  
        },
        syn::MetaItem::NameValue(_, syn::Lit::Int(ref lit, _)) => perms = *lit,
        _ => {},
    }
    perms
}

pub fn parse_help_available(attr: &syn::Attribute) -> bool {
    let mut ha = true;
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let &syn::NestedMetaItem::Literal(syn::Lit::Bool(li)) = lit {
                    ha = li;
                }
            }  
        },
        syn::MetaItem::NameValue(_, syn::Lit::Bool(lit)) => ha = lit,
        _ => {},
    }
    ha
}

pub fn parse_only(attr: &syn::Attribute) -> Only {
    let (mut guild, mut dm, mut owner) = (false, false, false);
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let &syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, syn::Lit::Bool(li))) = lit {
                    match ident.as_ref() {
                        "guild" => guild = li,
                        "dm" => dm = li,
                        "owner" => owner = li,
                        _ => {}
                    }
                }
            }  
        },
        _ => {},
    }
    if guild {
        Only::Guild
    } else if dm {
        Only::Dm
    } else if owner {
        Only::Owner
    } else {
        unreachable!()
    }
}

pub fn parse_aliases(attr: &syn::Attribute) -> Vec<String> {
    let mut aliases = Vec::new();
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let &syn::NestedMetaItem::Literal(syn::Lit::Str(ref li, _)) = lit {
                    aliases.push(li.clone());
                }
            }  
        },
        _ => {},
    }
    aliases
}

pub fn parse_group(attr: &syn::Attribute) -> (String, String) {
    let mut group = "".to_string();
    let mut prefix = "".to_string();
    match attr.value {
        syn::MetaItem::List(_, ref vec) => {
            for lit in vec {
                if let syn::NestedMetaItem::Literal(syn::Lit::Str(ref li, _)) = *lit {
                    group = li.clone();
                }
                if let syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref ident, syn::Lit::Str(ref li, _))) = *lit {
                    if ident.as_ref() == "prefix" {
                        prefix = li.clone();
                    }
                }
            }  
        },
        _ => {},
    }
    (group, prefix)
}