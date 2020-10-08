#[cfg(feature = "driver")]
use crate::model::id::{
	GuildId as DriverGuild,
	UserId as DriverUser,
};
#[cfg(feature = "serenity")]
use serenity::model::id::{
	ChannelId as SerenityChannel,
	GuildId as SerenityGuild,
	UserId as SerenityUser,
};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ChannelId(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GuildId(pub u64);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UserId(pub u64);

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl From<u64> for ChannelId {
	fn from(id: u64) -> Self {
		Self(id)
	}
}

#[cfg(feature = "serenity")]
impl From<SerenityChannel> for ChannelId {
	fn from(id: SerenityChannel) -> Self {
		Self(id.0)
	}
}

impl Display for GuildId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl From<u64> for GuildId {
	fn from(id: u64) -> Self {
		Self(id)
	}
}

#[cfg(feature = "serenity")]
impl From<SerenityGuild> for GuildId {
	fn from(id: SerenityGuild) -> Self {
		Self(id.0)
	}
}

#[cfg(feature = "driver")]
impl From<GuildId> for DriverGuild {
	fn from(id: GuildId) -> Self {
		Self(id.0)
	}
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl From<u64> for UserId {
	fn from(id: u64) -> Self {
		Self(id)
	}
}

#[cfg(feature = "serenity")]
impl From<SerenityUser> for UserId {
	fn from(id: SerenityUser) -> Self {
		Self(id.0)
	}
}

#[cfg(feature = "driver")]
impl From<UserId> for DriverUser {
	fn from(id: UserId) -> Self {
		Self(id.0)
	}
}
