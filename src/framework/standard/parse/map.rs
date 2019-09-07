use std::collections::HashMap;
use std::sync::Arc;

use super::super::*;

#[derive(Debug)]
pub enum Map {
    WithPrefixes(GroupMap),
    Prefixless(GroupMap, CommandMap),
}

pub trait ParseMap {
    type Storage;

    fn get(&self, n: &str) -> Option<Self::Storage>;
    fn min_length(&self) -> usize;
    fn max_length(&self) -> usize;
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct CommandMap {
    cmds: HashMap<&'static str, (&'static Command, Arc<CommandMap>)>,
    min_length: usize,
    max_length: usize,
}

impl CommandMap {
    pub fn new(cmds: &[&'static Command]) -> Self {
        let mut map = Self::default();

        for cmd in cmds {
            let sub_map = Arc::new(Self::new(&cmd.options.sub_commands));

            for name in cmd.options.names {
                let len = name.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                map.cmds.insert(*name, (*cmd, sub_map.clone()));
            }
        }

        map
    }
}

impl ParseMap for CommandMap {
    type Storage = (&'static Command, Arc<CommandMap>);

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
        self.cmds.get(&name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct GroupMap {
    groups: HashMap<&'static str, (&'static CommandGroup, Arc<GroupMap>, Arc<CommandMap>)>,
    min_length: usize,
    max_length: usize,
}

impl GroupMap {
    pub fn new(groups: &[&'static CommandGroup]) -> Self {
        let mut map = Self::default();

        for group in groups {
            let subgroups_map = Arc::new(Self::new(&group.options.sub_groups));
            let commands_map = Arc::new(CommandMap::new(&group.options.commands));

            for prefix in group.options.prefixes {
                let len = prefix.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                map.groups.insert(*prefix, (*group, subgroups_map.clone(), commands_map.clone()));
            }
        }

        map
    }
}

impl ParseMap for GroupMap {
    type Storage = (&'static CommandGroup, Arc<GroupMap>, Arc<CommandMap>);

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
