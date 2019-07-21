use std::collections::HashMap;
use std::sync::Arc;

use super::super::*;

#[derive(Debug)]
pub enum Map {
    WithPrefixes(GroupMap),
    Prefixless(GroupMap, CommandMap),
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

    #[inline]
    pub fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    pub fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<(&'static Command, Arc<CommandMap>)> {
        self.cmds.get(&name).cloned()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
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
            let subgroups_map = Arc::new(Self::new(&group.sub_groups));
            let commands_map = Arc::new(CommandMap::new(&group.commands));

            for prefix in group.options.prefixes {
                let len = prefix.chars().count();
                map.min_length = std::cmp::min(len, map.min_length);
                map.max_length = std::cmp::max(len, map.max_length);

                map.groups.insert(*prefix, (*group, subgroups_map.clone(), commands_map.clone()));
            }
        }

        map
    }

    #[inline]
    pub fn min_length(&self) -> usize {
        self.min_length
    }

    #[inline]
    pub fn max_length(&self) -> usize {
        self.max_length
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<(&'static CommandGroup, Arc<GroupMap>, Arc<CommandMap>)> {
        self.groups.get(&name).cloned()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}
