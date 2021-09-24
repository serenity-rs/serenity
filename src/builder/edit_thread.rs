use std::collections::HashMap;

use crate::internal::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct EditThread(pub HashMap<&'static str, Value>);

impl EditThread {
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

    /// The archive status of the thread.
    ///
    /// **Note**: A thread that is `locked` can only be unarchived if the user has the `MANAGE_THREADS` permission.
    pub fn archived(&mut self, archived: bool) -> &mut Self {
        self.0.insert("archived", Value::Bool(archived));

        self
    }

    /// The lock status of the thread.
    pub fn locked(&mut self, lock: bool) -> &mut Self {
        self.0.insert("locked", Value::Bool(lock));

        self
    }
}
