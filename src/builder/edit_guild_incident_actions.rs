#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for editing guild incident actions.
///
/// [Discord's docs]: https://discord.com/developers/docs/resources/guild#modify-guild-incident-actions
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

    /// Sets the time until which guild invites will remain disabled, which can be at most 24 hours
    /// in the future.
    pub fn invites_disabled_until(mut self, timestamp: Timestamp) -> Self {
        self.invites_disabled_until = Some(timestamp);
        self
    }

    /// Sets the time at which direct messages for users within the guild will remain disabled,
    /// which can be at most 24 hours in the future.
    pub fn dms_disabled_until(mut self, timestamp: Timestamp) -> Self {
        self.dms_disabled_until = Some(timestamp);
        self
    }

    /// Modifies the guild's incident actions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if invalid data is given. See [Discord's docs] for more details.
    ///
    /// May also return [`Error::Json`] if there is an error in deserializing the API response.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http, guild_id: GuildId) -> Result<IncidentsData> {
        http.edit_guild_incident_actions(guild_id, &self).await
    }
}
