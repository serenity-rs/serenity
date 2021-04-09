//! This module exports different types for JSON interactions.
//! It encapsulates the differences between serde_json and simd-json to allow
//! ignoring those in the rest of the codebase.

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
use serde_json::Number;
#[cfg(feature = "simd-json")]
pub use simd_json::json;

#[cfg(not(feature = "simd-json"))]
pub type JsonMap = serde_json::Map<String, Value>;
#[cfg(feature = "simd-json")]
pub type JsonMap = simd_json::owned::Object;

#[cfg(not(feature = "simd-json"))]
pub const NULL: Value = Value::Null;
#[cfg(feature = "simd-json")]
pub const NULL: Value = Value::Static(simd_json::StaticNode::Null);

#[cfg(not(feature = "simd-json"))]
pub(crate) fn to_vec<T>(v: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    Ok(serde_json::to_vec(v)?)
}

#[cfg(feature = "simd-json")]
pub(crate) fn to_vec<T>(v: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    Ok(simd_json::to_vec(v)?)
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
pub(crate) fn from_reader<R, T>(r: R) -> Result<T>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    Ok(serde_json::from_reader(r)?)
}
#[cfg(feature = "simd-json")]
pub(crate) fn from_reader<R, T>(r: R) -> Result<T>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    Ok(simd_json::from_reader(r)?)
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

#[cfg(not(feature = "simd-json"))]
pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    Ok(serde_json::to_value(value)?)
}

#[cfg(feature = "simd-json")]
pub(crate) fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    Ok(simd_json::serde::to_owned_value(value)?)
}

#[cfg(not(feature = "simd-json"))]
pub(crate) fn from_number<T>(n: T) -> Value
where
    serde_json::Number: From<T>,
{
    Value::Number(Number::from(n))
}
#[cfg(feature = "simd-json")]
pub(crate) fn from_number<T>(n: T) -> Value
where
    Value: From<T>,
{
    Value::from(n)
}

pub mod prelude {
    #[cfg(feature = "simd-json")]
    pub use simd_json::{Builder, Mutable, Value as ValueTrait, ValueAccess};

    pub use super::*;
}
