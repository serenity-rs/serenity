#[macro_use]
pub mod macros;

pub mod prelude;

#[cfg(feature = "voice")]
mod timer;

#[cfg(feature = "voice")]
pub use self::timer::Timer;
