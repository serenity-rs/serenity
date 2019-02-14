use std::fmt::Debug;
use std::fmt;
use model::channel::Message;
use client::Context;
use framework::standard::{Args, CommandOptions};

pub type CheckFunction = dyn Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> CheckResult
    + Send
    + Sync
    + 'static;

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

/// A check can be part of a command or group and will be executed to
/// determine whether a user is permitted to use related item.
///
/// Additionally, a check may hold additional settings.
pub struct Check {
    /// Name listed in help-system.
    pub name: String,
    /// Function that will be executed.
    pub function: Box<CheckFunction>,
    /// Whether a check should be evaluated in the help-system.
    /// `false` will ignore check and won't fail execution.
    pub check_in_help: bool,
    /// Whether a check shall be listed in the help-system.
    /// `false` won't affect whether the check will be evaluated help,
    /// solely `check_in_help` sets this.
    pub display_in_help: bool,
}

impl Check {
    pub(crate) fn new<F>(name: &str, function: F, check_in_help: bool, display_in_help: bool) -> Self
    where F: Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> CheckResult  + Send + Sync + 'static {
        Self {
            name: name.to_string(),
            function: Box::new(function),
            check_in_help,
            display_in_help,
        }
    }
}

impl Debug for Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Check")
            .field(&format!("name: {}", self.name))
            .field(&"function: <fn>")
            .field(&format!("check_in_help: {}", self.check_in_help))
            .field(&format!("display_in_help: {}", self.display_in_help))
            .finish()
    }
}

/// A builder to create a [`Check`].
///
/// [`Check`]: struct.Check.html
#[derive(Debug)]
pub struct CreateCheck(pub Check);

impl CreateCheck {
    /// Creates a new builder to construct a [`Check`].
    /// [`Check`]s always require a `function`, otherwise they would not be
    /// testable as there is nothing to run.
    ///
    /// [`Check`]: struct.Check.html
    pub fn new<F>(function: F) -> Self
    where F: Fn(&mut Context, &Message, &mut Args, &CommandOptions) -> CheckResult  + Send + Sync + 'static {
        Self(
            Check {
                name: String::default(),
                function: Box::new(function),
                check_in_help: true,
                display_in_help: true,
            }
        )
    }

    /// Sets name of the [`Check`] that will be displayed in the help-system.
    ///
    /// [`Check`]: struct.Check.html
    #[inline]
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.0.name = name.to_string();

        self
    }

    /// If set to `true`, the created [`Check`] will be tested in the help-system
    /// as well.
    /// If set to `false`, the [`Check`]'s [`function`] won't be run.
    ///
    /// **Note**:
    /// Having many checks `true` will affect the time generating the help-post.
    /// Therefore setting performance intensive checks to `false` should
    /// be considered.
    /// However, if set to `false`, the [`Check`] will be considered as *passed*.
    ///
    /// [`Check`]: struct.Check.html
    /// [`function`]: struct.Check.html#structfield.function
    #[inline]
    pub fn check_in_help(&mut self, check_in_help: bool) -> &mut Self {
        self.0.check_in_help = check_in_help;

        self
    }

    /// Hides [`Check`] from being listed in the help-system.
    /// This does not affect whether the [`Check`] will be run.
    ///
    /// [`Check`]: struct.Check.html
    #[inline]
    pub fn display_in_help(&mut self, display_in_help: bool) -> &mut Self {
        self.0.display_in_help = display_in_help;

        self
    }
}
