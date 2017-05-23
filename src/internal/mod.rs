#[macro_use]
pub mod macros;

pub mod prelude;

#[cfg(feature="gateway")]
pub mod ws_impl;

#[cfg(feature="voice")]
mod timer;

#[cfg(feature="voice")]
pub use self::timer::Timer;
