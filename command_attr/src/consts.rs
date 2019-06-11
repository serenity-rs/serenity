pub mod suffixes {
    pub const COMMAND: &str = "COMMAND";
    pub const COMMAND_OPTIONS: &str = "COMMAND_OPTIONS";
    pub const HELP_OPTIONS: &str = "_OPTIONS";
    pub const GROUP: &str = "GROUP";
    pub const GROUP_OPTIONS: &str = "GROUP_OPTIONS";
    pub const CHECK: &str = "CHECK";
}

pub use self::suffixes::*;
