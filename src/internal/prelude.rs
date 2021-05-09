//! These prelude re-exports are a set of exports that are commonly used from
//! within the library.
//!
//! These are not publicly re-exported to the end user, and must stay as a
//! private module.

pub type JsonMap = Map<String, Value>;

pub use std::result::Result as StdResult;

pub use serde_json::{Map, Number, Value};

#[cfg(feature = "client")]
pub use crate::client::ClientError;
pub use crate::error::{Error, Result};
