use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use std::default::Default;
use ::model::{ChannelId, Region, VerificationLevel};

pub struct EditGuild(pub ObjectBuilder);

impl EditGuild {
    pub fn afk_channel<C: Into<ChannelId>>(self, channel: Option<C>) -> Self {
        EditGuild(match channel {
            Some(channel) => self.0.insert("afk_channel_id", channel.into().0),
            None => self.0.insert("afk-channel_id", Value::Null),
        })
    }

    pub fn afk_timeout(self, timeout: u64) -> Self {
        EditGuild(self.0.insert("afk_timeout", timeout))
    }

    pub fn icon(self, icon: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("icon",
                    icon.map_or_else(|| Value::Null,
                                     |x| Value::String(x.to_owned()))))
    }

    pub fn name(self, name: &str) -> Self {
        EditGuild(self.0.insert("name", name))
    }

    pub fn region(self, region: Region) -> Self {
        EditGuild(self.0.insert("region", region.name()))
    }

    pub fn splash(self, splash: Option<&str>) -> Self {
        EditGuild(self.0
            .insert("splash",
                    splash.map_or_else(|| Value::Null,
                                       |x| Value::String(x.to_owned()))))
    }

    pub fn verification_level<V>(self, verification_level: V) -> Self
        where V: Into<VerificationLevel> {
        EditGuild(self.0.insert("verification_level",
                                verification_level.into().num()))
    }
}

impl Default for EditGuild {
    fn default() -> EditGuild {
        EditGuild(ObjectBuilder::new())
    }
}
