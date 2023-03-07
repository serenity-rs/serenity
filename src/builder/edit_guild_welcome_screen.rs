use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::json;
use crate::model::guild::GuildWelcomeChannelEmoji;

/// A builder to specify the fields to edit in a [`GuildWelcomeScreen`].
///
/// [`GuildWelcomeScreen`]: crate::model::guild::GuildWelcomeScreen
#[derive(Clone, Debug, Default)]
pub struct EditGuildWelcomeScreen(pub HashMap<&'static str, Value>);

impl EditGuildWelcomeScreen {
    /// Whether the welcome screen is enabled or not.
    pub fn enabled(&mut self, enabled: bool) -> &mut Self {
        self.0.insert("enabled", Value::from(enabled));

        self
    }

    /// The server description shown in the welcome screen.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));

        self
    }

    pub fn create_welcome_channel<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut CreateGuildWelcomeChannel) -> &mut CreateGuildWelcomeChannel,
    {
        let mut data = CreateGuildWelcomeChannel::default();
        f(&mut data);

        self.add_welcome_channel(data);

        self
    }

    pub fn add_welcome_channel(&mut self, channel: CreateGuildWelcomeChannel) -> &mut Self {
        let new_data = json::hashmap_to_json_map(channel.0);

        let channels =
            self.0.entry("welcome_channels").or_insert_with(|| Value::from(Vec::<Value>::new()));
        let channels_array = channels.as_array_mut().expect("Must be an array.");

        channels_array.push(Value::from(new_data));

        self
    }

    pub fn set_welcome_channels(&mut self, channels: Vec<CreateGuildWelcomeChannel>) -> &mut Self {
        let new_channels = channels
            .into_iter()
            .map(|f| Value::from(json::hashmap_to_json_map(f.0)))
            .collect::<Vec<Value>>();

        self.0.insert("welcome_channels", Value::from(new_channels));

        self
    }
}

/// A builder for creating a [`GuildWelcomeChannel`].
///
/// [`GuildWelcomeChannel`]: crate::model::guild::GuildWelcomeChannel
#[derive(Clone, Debug, Default)]
pub struct CreateGuildWelcomeChannel(pub HashMap<&'static str, Value>);

impl CreateGuildWelcomeChannel {
    /// The Id of the channel to show. It is required.
    pub fn id(&mut self, id: u64) -> &mut Self {
        self.0.insert("channel_id", Value::from(id.to_string()));

        self
    }

    /// The description shown for the channel. It is required.
    pub fn description<D: ToString>(&mut self, description: D) -> &mut Self {
        self.0.insert("description", Value::from(description.to_string()));

        self
    }

    /// The emoji shown for the channel.
    pub fn emoji(&mut self, emoji: GuildWelcomeChannelEmoji) -> &mut Self {
        match emoji {
            GuildWelcomeChannelEmoji::Unicode(name) => {
                self.0.insert("emoji_name", Value::from(name));
            },
            GuildWelcomeChannelEmoji::Custom {
                id,
                name,
            } => {
                self.0.insert("emoji_id", Value::from(id.to_string()));
                self.0.insert("emoji_name", Value::from(name));
            },
        }

        self
    }
}
