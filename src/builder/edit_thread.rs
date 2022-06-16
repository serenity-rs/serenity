#[derive(Debug, Clone, Default, Serialize)]
pub struct EditThread {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invitable: Option<bool>,
}

impl EditThread {
    /// The name of the thread.
    ///
    /// **Note**: Must be between 2 and 100 characters long.
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());

        self
    }

    /// Duration in minutes to automatically archive the thread after recent activity.
    ///
    /// **Note**: Can only be set to 60, 1440, 4320, 10080 currently.
    pub fn auto_archive_duration(&mut self, duration: u16) -> &mut Self {
        self.auto_archive_duration = Some(duration);

        self
    }

    /// The archive status of the thread.
    ///
    /// **Note**: A thread that is `locked` can only be unarchived if the user has the `MANAGE_THREADS` permission.
    pub fn archived(&mut self, archived: bool) -> &mut Self {
        self.archived = Some(archived);

        self
    }

    /// The lock status of the thread.
    pub fn locked(&mut self, lock: bool) -> &mut Self {
        self.locked = Some(lock);

        self
    }

    /// Whether non-moderators can add other non-moderators to a thread.
    ///
    /// **Note**: Only available on private threads.
    pub fn invitable(&mut self, invitable: bool) -> &mut Self {
        self.invitable = Some(invitable);

        self
    }
}
