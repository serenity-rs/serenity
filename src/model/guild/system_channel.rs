use bitflags::__impl_bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::internal::prelude::StdResult;
use crate::model::utils::U64Visitor;

/// Describes a system channel flags.
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(not(feature = "model"), derive(Debug, Deserialize, Serialize))]
pub struct SystemChannelFlags {
    pub bits: u64,
}

#[cfg(feature = "model")]
__impl_bitflags! {
    SystemChannelFlags: u64 {
        /// Suppress member join notifications.
        SUPPRESS_JOIN_NOTIFICATIONS = 0b0000_0000_0000_0000_0000_0000_0000_0001;
        /// Suppress server boost notifications.
        SUPPRESS_PREMIUM_SUBSCRIPTIONS = 0b0000_0000_0000_0000_0000_0000_0000_0010;
        /// Suppress server setup tips.
        SUPPRESS_GUILD_REMINDER_NOTIFICATIONS = 0b0000_0000_0000_0000_0000_0000_0000_0100;
    }
}

#[cfg(feature = "model")]
impl<'de> Deserialize<'de> for SystemChannelFlags {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(SystemChannelFlags::from_bits_truncate(deserializer.deserialize_u64(U64Visitor)?))
    }
}

#[cfg(feature = "model")]
impl Serialize for SystemChannelFlags {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}
