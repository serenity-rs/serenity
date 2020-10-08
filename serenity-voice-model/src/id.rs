//! A collection of newtypes defining type-strong IDs.
use serde::{Deserialize, Serialize};
use crate::util::json_safe_u64;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GuildId(#[serde(with = "json_safe_u64")] pub u64);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserId(#[serde(with = "json_safe_u64")] pub u64);
