//! This module exports different types for JSON interactions.
//! It encapsulates the differences between serde_json and simd-json to allow
//! ignoring those in the rest of the codebase.

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

#[cfg(feature = "gateway")]
use serde::de::Deserialize;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::Result;

#[cfg(not(feature = "simd-json"))]
pub type Value = serde_json::Value;
#[cfg(feature = "simd-json")]
pub type Value = simd_json::OwnedValue;

#[cfg(not(feature = "simd-json"))]
pub use serde_json::json;
#[cfg(not(feature = "simd-json"))]
pub use serde_json::Error as JsonError;
#[cfg(feature = "simd-json")]
pub use simd_json::json;
#[cfg(feature = "simd-json")]
pub use simd_json::Error as JsonError;

#[cfg(not(feature = "simd-json"))]
pub type JsonMap = serde_json::Map<String, Value>;
#[cfg(feature = "simd-json")]
pub type JsonMap = simd_json::owned::Object;

#[cfg(not(feature = "simd-json"))]
pub const NULL: Value = Value::Null;
#[cfg(feature = "simd-json")]
pub const NULL: Value = Value::Static(simd_json::StaticNode::Null);

#[cfg(all(feature = "http", not(feature = "simd-json")))]
pub(crate) async fn decode_resp<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T> {
    let bytes = resp.bytes().await?;
    serde_json::from_slice(&bytes).map_err(From::from)
}

#[cfg(all(feature = "http", feature = "simd-json"))]
pub(crate) async fn decode_resp<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T> {
    let mut bytes = resp.bytes().await?.to_vec();
    simd_json::from_slice(&mut bytes).map_err(From::from)
}

/// Converts a HashMap into a final [`JsonMap`] representation.
pub fn hashmap_to_json_map<H, T>(map: HashMap<T, Value, H>) -> JsonMap
where
    H: BuildHasher,
    T: Eq + Hash + ToString,
{
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[cfg(not(feature = "simd-json"))]
pub(crate) fn to_string<T>(v: &T) -> Result<String>
where
    T: Serialize,
{
    Ok(serde_json::to_string(v)?)
}

#[cfg(feature = "simd-json")]
pub(crate) fn to_string<T>(v: &T) -> Result<String>
where
    T: Serialize,
{
    Ok(simd_json::to_string(v)?)
}

#[cfg(not(feature = "simd-json"))]
pub(crate) fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    Ok(serde_json::from_str(s)?)
}

#[cfg(feature = "simd-json")]
pub(crate) fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    Ok(simd_json::from_str(s)?)
}

#[cfg(not(feature = "simd-json"))]
pub(crate) fn from_value<T>(v: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(v)?)
}

#[cfg(feature = "simd-json")]
pub(crate) fn from_value<T>(v: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(simd_json::serde::from_owned_value(v)?)
}

#[cfg(all(any(feature = "builder", feature = "http"), not(feature = "simd-json"), test))]
pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    Ok(serde_json::to_value(value)?)
}

#[cfg(all(any(feature = "builder", feature = "http"), feature = "simd-json", test))]
pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    Ok(simd_json::serde::to_owned_value(value)?)
}

pub mod prelude {
    #[cfg(not(feature = "simd-json"))]
    pub use serde_json::{
        from_reader,
        from_slice,
        from_str,
        from_value,
        to_string,
        to_string_pretty,
        to_value,
        to_vec,
        to_vec_pretty,
    };
    #[cfg(feature = "simd-json")]
    pub use simd_json::{
        from_reader,
        from_slice,
        from_str,
        serde::from_owned_value as from_value,
        serde::to_owned_value as to_value,
        to_string,
        to_string_pretty,
        to_vec,
        to_vec_pretty,
    };
    #[cfg(feature = "simd-json")]
    pub use simd_json::{Builder, Mutable, StaticNode, Value as ValueTrait, ValueAccess};

    pub use super::*;
}
