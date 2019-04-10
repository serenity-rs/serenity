pub mod suffixes {
    pub const COMMAND: &str = "COMMAND";
    pub const COMMAND_OPTIONS: &str = "COMMAND_OPTIONS";
    pub const HELP: &str = "HELP_COMMAND";
    pub const HELP_OPTIONS: &str = "HELP_OPTIONS";
    pub const GROUP: &str = "GROUP";
    pub const GROUP_OPTIONS: &str = "GROUP_OPTIONS";
}

pub use self::suffixes::*;
