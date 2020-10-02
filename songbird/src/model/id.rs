#[cfg(feature = "serenity")]
use serenity::model::id::{
	GuildId as SerenityGuild,
	UserId as SerenityUser,
};

pub struct GuildId(pub u64);

pub struct UserId(pub u64);

#[cfg(feature = "serenity")]
impl From<SerenityGuild> for GuildId {
	fn from(g: SerenityGuild) -> Self {
		Self(g.0)
	}
}

#[cfg(feature = "serenity")]
impl From<SerenityUser> for UserId {
	fn from(g: SerenityUser) -> Self {
		Self(g.0)
	}
}
