use std::fmt::Debug;
use std::fmt;
use crate::model::channel::Message;
use crate::client::Context;
use crate::framework::standard::{Args, CommandOptions};

/// This type describes why a check has failed and occurs on
/// [`CheckResult::Failure`].
///
/// **Note**:
/// The bot-developer is supposed to process this `enum` as the framework is not.
/// It solely serves as a way to inform a user about why a check
/// has failed and for the developer to log given failure (e.g. bugs or statstics)
/// occurring in [`Check`]s.
///
/// [`Check`]: struct.Check.html
/// [`CheckResult::Failure`]: enum.CheckResult.html#variant.Failure
#[derive(Clone, Debug)]
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

/// Returned from [`Check`]s.
/// If `Success`, the [`Check`] is considered as passed.
/// If `Failure`, the [`Check`] is considered as failed and can return further
/// information on the cause via [`Reason`].
///
/// [`Check`]: struct.Check.html
/// [`Reason`]: struct.Reason.html
#[derive(Clone, Debug)]
pub enum CheckResult {
   Success,
   Failure(Reason),
}

impl CheckResult {
    /// Creates a new [`CheckResult::Failure`] with [`Reason::User`].
    ///
    /// [`CheckResult::Failure`]: enum.CheckResult.html#variant.Failure
    /// [`Reason::User`]: struct.Reason.html#variant.User
    pub fn new_user<D>(d: D) -> Self
        where D: fmt::Display {
        CheckResult::Failure(Reason::User(d.to_string()))
    }

    /// Creates a new [`CheckResult::Failure`] with [`Reason::Log`].
    ///
    /// [`CheckResult::Failure`]: enum.CheckResult.html#variant.Failure
    /// [`Reason::Log`]: struct.Reason.html#variant.Log
    pub fn new_log<D>(d: D) -> Self
        where D: fmt::Display {
        CheckResult::Failure(Reason::Log(d.to_string()))
    }

    /// Creates a new [`CheckResult::Failure`] with [`Reason::Unknown`].
    ///
    /// [`CheckResult::Failure`]: enum.CheckResult.html#variant.Failure
    /// [`Reason::Unknown`]: struct.Reason.html#variant.Unknown
    pub fn new_unknown() -> Self {
        CheckResult::Failure(Reason::Unknown)
    }

    /// Creates a new [`CheckResult::Failure`] with [`Reason::UserAndLog`].
    ///
    /// [`CheckResult::Failure`]: enum.CheckResult.html#variant.Failure
    /// [`Reason::UserAndLog`]: struct.Reason.html#variant.UserAndLog
    pub fn new_user_and_log<D>(user: D, log: D) -> Self
        where D: fmt::Display {
        CheckResult::Failure(Reason::UserAndLog {
            user: user.to_string(),
            log: log.to_string(),
        })
    }

    /// Returns `true` if [`CheckResult`] is [`CheckResult::Success`] and
    /// `false` if not.
    ///
    /// [`CheckResult`]: enum.CheckResult.html
    /// [`CheckResult::Success`]: enum.CheckResult.html#variant.Success
    pub fn is_success(&self) -> bool {
        if let CheckResult::Success = self {
            return true;
        }

        false
    }
}

impl From<bool> for CheckResult {
    fn from(succeeded: bool) -> Self {
        if succeeded {
            CheckResult::Success
        } else {
            CheckResult::Failure(Reason::Unknown)
        }
    }
}

impl From<Reason> for CheckResult {
    fn from(reason: Reason) -> Self {
        CheckResult::Failure(reason)
    }
}

pub type CheckFunction = fn(&mut Context, &Message, &mut Args, &CommandOptions) -> CheckResult;

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
