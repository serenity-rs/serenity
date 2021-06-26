use std::collections::HashMap;

use crate::internal::prelude::*;
use crate::model::channel::ChannelType;

#[derive(Debug, Clone, Default)]
pub struct CreateThread(pub HashMap<&'static str, Value>);

impl CreateThread {
    /// The name of the thread.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name<D: ToString>(&mut self, name: D) -> &mut Self {
        self.0.insert("name", Value::String(name.to_string()));

        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: Can only be set to 60, 1440, 4320, 10080 currently.
    pub fn auto_archive_duration(&mut self, duration: u16) -> &mut Self {
        self.0.insert("auto_archive_duration", Value::Number(Number::from(duration)));

        self
    }

    /// The thread type, which can be [`ChannelType::PublicThread`] or [`ChannelType::PrivateThread`].
    ///
    /// **Note**: It defaults to [`ChannelType::PrivateThread`] in order to match the behavior when thread documentation was first published.
    /// This is a bit of a weird default though, and thus is highly likely to change in the future,
    /// so it is recommended to always explicitly setting it to avoid any breaking change.
    ///
    /// [`ChannelType::PublicThread`]: crate::model::channel::ChannelType::PublicThread
    /// [`ChannelType::PrivateThread`]: crate::model::channel::ChannelType::PrivateThread
    pub fn kind(&mut self, kind: ChannelType) -> &mut Self {
        self.0.insert("type", Value::Number(Number::from(kind as u8)));

        self
    }
}
