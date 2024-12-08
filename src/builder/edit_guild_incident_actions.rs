#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for editing guild incident actions.
///
/// [Discord's docs]: https://github.com/discord/discord-api-docs/pull/6396
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildIncidentActions {
    invites_disabled_until: Option<Timestamp>,
    dms_disabled_until: Option<Timestamp>,
}

impl EditGuildIncidentActions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the time invites to the guild will be disabled until. Must be no further than 1 day in
    /// the future.
    pub fn invites_disabled_until(mut self, timestamp: Timestamp) -> Self {
        self.invites_disabled_until = Some(timestamp);
        self
    }

    /// Sets the time dms for users within the guild will be disabled until. Must be no further
    /// than 1 day in the future.
    pub fn dms_disabled_until(mut self, timestamp: Timestamp) -> Self {
        self.dms_disabled_until = Some(timestamp);
        self
    }

    /// Modifies the guild's incident actions.
    ///
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given. See [Discord's docs] for more details.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    ///
    /// [Discord's docs]: https://github.com/discord/discord-api-docs/pull/6396
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<IncidentsData> {
        http.edit_guild_incident_actions(guild_id, &self).await
    }
}
