//! This module exports different types for JSON interactions.
//! It encapsulates the differences between serde_json and simd-json to allow
//! ignoring those in the rest of the codebase.

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use serde::de::{Deserialize, DeserializeOwned};
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

#[cfg(feature = "http")]
pub(crate) async fn decode_resp<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T> {
    #[cfg(not(feature = "simd-json"))]
    let result = serde_json::from_slice(&resp.bytes().await?)?;
    #[cfg(feature = "simd-json")]
    let result = simd_json::from_slice(&mut resp.bytes().await?.to_vec())?;
    Ok(result)
}

/// Converts a HashMap into a final [`JsonMap`] representation.
pub fn hashmap_to_json_map<H, T>(map: HashMap<T, Value, H>) -> JsonMap
where
    H: BuildHasher,
    T: Eq + Hash + ToString,
{
    map.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[allow(clippy::missing_errors_doc)] // It's obvious
pub fn to_string<T>(v: &T) -> Result<String>
where
    T: Serialize,
{
    #[cfg(not(feature = "simd-json"))]
    let result = serde_json::to_string(v)?;
    #[cfg(feature = "simd-json")]
    let result = simd_json::to_string(v)?;
    Ok(result)
}

#[allow(clippy::missing_errors_doc)] // It's obvious
pub fn from_str<'a, T>(s: &'a mut str) -> Result<T>
where
    T: Deserialize<'a>,
{
    #[cfg(not(feature = "simd-json"))]
    let result = serde_json::from_str(s)?;
    #[cfg(feature = "simd-json")]
    let result = simd_json::from_str(s)?;
    Ok(result)
}

pub(crate) fn from_value<T>(v: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    #[cfg(not(feature = "simd-json"))]
    let result = serde_json::from_value(v)?;
    #[cfg(feature = "simd-json")]
    let result = simd_json::serde::from_owned_value(v)?;
    Ok(result)
}

#[cfg(test)]
pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    #[cfg(not(feature = "simd-json"))]
    let result = serde_json::to_value(value)?;
    #[cfg(feature = "simd-json")]
    let result = simd_json::serde::to_owned_value(value)?;
    Ok(result)
}

#[cfg(test)]
#[track_caller]
pub(crate) fn assert_json<T>(data: &T, json: crate::json::Value)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    // test serialization
    let serialized = to_value(data).unwrap();
    assert!(
        serialized == json,
        "data->JSON serialization failed\nexpected: {json:?}\n     got: {serialized:?}"
    );

    // test deserialization
    let deserialized = from_value::<T>(json).unwrap();
    assert!(
        &deserialized == data,
        "JSON->data deserialization failed\nexpected: {data:?}\n     got: {deserialized:?}"
    );
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
