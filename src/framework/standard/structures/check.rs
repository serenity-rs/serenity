use std::fmt::Debug;
use std::fmt;
use crate::model::channel::Message;
use crate::client::Context;
use crate::framework::standard::{Args, CommandOptions};
use futures::future::BoxFuture;

/// This type describes why a check has failed.
///
/// **Note**:
/// The bot-developer is supposed to process this `enum` as the framework is not.
/// It solely serves as a way to inform a user about why a check
/// has failed and for the developer to log given failure (e.g. bugs or statistics)
/// occurring in [`Check`]s.
///
/// [`Check`]: struct.Check.html
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Reason {
    /// No information on the failure.
    Unknown,
    /// Information dedicated to the user.
    User(String),
    /// Information purely for logging purposes.
    Log(String),
    /// Information for the user but also for logging purposes.
    UserAndLog { user: String, log: String },
}

pub type CheckFunction = for<'fut> fn(
    &'fut Context,
    &'fut Message,
    &'fut mut Args,
    &'fut CommandOptions,
) -> BoxFuture<'fut, Result<(), Reason>>;

/// A check can be part of a command or group and will be executed to
/// determine whether a user is permitted to use related item.
///
/// Additionally, a check may hold additional settings.
pub struct Check {
    /// Name listed in help-system.
    pub name: &'static str,
    /// Function that will be executed.
    pub function: CheckFunction,
    /// Whether a check should be evaluated in the help-system.
    /// `false` will ignore check and won't fail execution.
    pub check_in_help: bool,
    /// Whether a check shall be listed in the help-system.
    /// `false` won't affect whether the check will be evaluated help,
    /// solely `check_in_help` sets this.
    pub display_in_help: bool,
}

impl Debug for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Check")
            .field("name", &self.name)
            .field("function", &"<fn>")
            .field("check_in_help", &self.check_in_help)
            .field("display_in_help", &self.display_in_help)
            .finish()
    }
}

impl PartialEq for Check {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
