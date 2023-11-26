use std::collections::HashMap;
use std::sync::Arc;

use derivative::Derivative;

use crate::framework::standard::*;

#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub enum Map<D: Send + Sync + 'static> {
    WithPrefixes(GroupMap<D>),
    Prefixless(GroupMap<D>, CommandMap<D>),
}

pub trait ParseMap {
    type Storage;

    fn get(&self, n: &str) -> Option<Self::Storage>;
    fn min_length(&self) -> usize;
    fn max_length(&self) -> usize;
    fn is_empty(&self) -> bool;
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct CommandMap<D: Send + Sync + 'static> {
    cmds: HashMap<String, (&'static Command<D>, Arc<CommandMap<D>>)>,
    min_length: usize,
    max_length: usize,
}

impl<D: Send + Sync + 'static> CommandMap<D> {
    pub fn new(cmds: &[&'static Command<D>], conf: &Configuration<D>) -> Self {
        let mut map = Self::default();

        for cmd in cmds {
            let sub_map = Arc::new(Self::new(cmd.options.sub_commands, conf));

            for name in cmd.options.names {
                let len = name.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                let name =
                    if conf.case_insensitive { name.to_lowercase() } else { (*name).to_string() };

                map.cmds.insert(name, (*cmd, Arc::clone(&sub_map)));
            }
        }

        map
    }
}

impl<D: Send + Sync + 'static> ParseMap for CommandMap<D> {
    type Storage = (&'static Command<D>, Arc<CommandMap<D>>);

    #[inline]
    fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.cmds.get(name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
#[allow(clippy::type_complexity)]
pub struct GroupMap<D: Send + Sync + 'static> {
    groups: HashMap<&'static str, (&'static CommandGroup<D>, Arc<GroupMap<D>>, Arc<CommandMap<D>>)>,
    min_length: usize,
    max_length: usize,
}

impl<D: Send + Sync + 'static> GroupMap<D> {
    pub fn new(groups: &[&'static CommandGroup<D>], conf: &Configuration<D>) -> Self {
        let mut map = Self::default();

        for group in groups {
            let subgroups_map = Arc::new(Self::new(group.options.sub_groups, conf));
            let commands_map = Arc::new(CommandMap::new(group.options.commands, conf));

            for prefix in group.options.prefixes {
                let len = prefix.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                map.groups.insert(
                    *prefix,
                    (*group, Arc::clone(&subgroups_map), Arc::clone(&commands_map)),
                );
            }
        }

        map
    }
}

impl<D: Send + Sync + 'static> ParseMap for GroupMap<D> {
    type Storage = (&'static CommandGroup<D>, Arc<GroupMap<D>>, Arc<CommandMap<D>>);

    #[inline]
    fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.groups.get(&name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}
