//! This module exports different types for JSON interactions. It encapsulates the differences
//! between serde_json and simd-json to allow ignoring those in the rest of the codebase.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

use serde::de::DeserializeOwned;
#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

use crate::Result;

#[cfg(not(feature = "simd_json"))]
mod export {
    pub type Value = serde_json::Value;
    pub type JsonMap = serde_json::Map<String, Value>;
    pub const NULL: Value = Value::Null;

    pub use serde_json::{json, Error as JsonError};
}

#[cfg(feature = "simd_json")]
mod export {
    pub type Value = simd_json::OwnedValue;
    pub type JsonMap = simd_json::owned::Object;
    pub const NULL: Value = Value::Static(simd_json::StaticNode::Null);

    pub use simd_json::prelude::{
        TypedContainerValue,
        ValueAsContainer,
        ValueAsMutContainer,
        ValueAsScalar,
    };
    pub use simd_json::{json, Error as JsonError, StaticNode};
}

pub use export::*;

#[cfg(feature = "http")]
pub(crate) async fn decode_resp<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T> {
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::from_slice(&resp.bytes().await?)?;
    #[cfg(feature = "simd_json")]
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

/// Deserialize an instance of type `T` from a string of JSON text.
///
/// If the `simd_json` feature is enabled, this function turns its argument into `Cow::Owned`
/// before deserializing from it. In other words, passing in a `&str` will result in a clone.
#[allow(clippy::missing_errors_doc)]
pub fn from_str<'a, T>(s: impl Into<Cow<'a, str>>) -> Result<T, JsonError>
where
    T: DeserializeOwned,
{
    let s = s.into();
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::from_str(&s)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::from_slice(&mut s.into_owned().into_bytes())?;
    Ok(result)
}

/// Deserialize an instance of type `T` from bytes of JSON text.
#[allow(clippy::missing_errors_doc)]
pub fn from_slice<T>(v: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::from_slice(v)?;
    #[cfg(feature = "simd_json")]
    // We clone here to obtain a mutable reference to the clone, since we don't have a mutable ref
    // to the original.
    let result = simd_json::from_slice(&mut v.to_vec())?;
    Ok(result)
}

/// Interpret a [`Value`] as an instance of type `T`.
#[allow(clippy::missing_errors_doc)]
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::from_value(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::serde::from_owned_value(value)?;
    Ok(result)
}

/// Deserialize an instance of type `T` from bytes of JSON text.
#[allow(clippy::missing_errors_doc)]
pub fn from_reader<R, T>(rdr: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::from_reader(rdr)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::from_reader(rdr)?;
    Ok(result)
}

/// Serialize the given data structure as a String of JSON.
#[allow(clippy::missing_errors_doc)]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::to_string(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::to_string(value)?;
    Ok(result)
}

/// Serialize the given data structure as a pretty-printed String of JSON.
#[allow(clippy::missing_errors_doc)]
pub fn to_string_pretty<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::to_string_pretty(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::to_string_pretty(value)?;
    Ok(result)
}

/// Serialize the given data structure as a JSON byte vector.
#[allow(clippy::missing_errors_doc)]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::to_vec(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::to_vec(value)?;
    Ok(result)
}

/// Serialize the given data structure as a pretty-printed JSON byte vector.
#[allow(clippy::missing_errors_doc)]
pub fn to_vec_pretty<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::to_vec_pretty(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::to_vec_pretty(value)?;
    Ok(result)
}

/// Convert a `T` into a [`Value`] which is an enum that can represent any valid JSON data.
#[allow(clippy::missing_errors_doc)]
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    #[cfg(not(feature = "simd_json"))]
    let result = serde_json::to_value(value)?;
    #[cfg(feature = "simd_json")]
    let result = simd_json::serde::to_owned_value(value)?;
    Ok(result)
}

#[cfg(test)]
#[track_caller]
pub(crate) fn assert_json<T>(data: &T, json: crate::json::Value)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
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
