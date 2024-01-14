//! These prelude re-exports are a set of exports that are commonly used from within the library.
//!
//! These are not publicly re-exported to the end user, and must stay as a private module.

pub use std::result::Result as StdResult;

pub use small_fixed_array::{FixedArray, FixedString, TruncatingInto};

pub use crate::error::{Error, Result};
pub use crate::json::{JsonMap, Value};
