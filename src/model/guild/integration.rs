use super::*;

/// Various information about integrations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Integration {
    pub id: IntegrationId,
    pub guild_id: GuildId,
    pub account: IntegrationAccount,
    pub enabled: bool,
    #[serde(rename = "expire_behaviour")]
    pub expire_behaviour: Option<IntegrationExpireBehaviour>,
    pub expire_grace_period: Option<u64>,
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub role_id: Option<RoleId>,
    pub synced_at: Option<u64>,
    pub syncing: Option<bool>,
    pub user: Option<User>,
    pub enable_emoticons: Option<bool>,
    pub subscriber_count: Option<u64>,
    pub revoked: Option<bool>,
    pub application: Option<IntegrationApplication>,
}

/// The behavior once the integration expires.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[non_exhaustive]
#[repr(u8)]
pub enum IntegrationExpireBehaviour {
    RemoveRole = 0,
    Kick = 1,
    Unknown = !0,
}

enum_number!(IntegrationExpireBehaviour {
    RemoveRole,
    Kick
});

impl From<Integration> for IntegrationId {
    /// Gets the Id of integration.
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

/// Integration account object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationAccount {
    pub id: String,
    pub name: String,
}

/// Integration application object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationApplication {
    pub id: ApplicationId,
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    pub summary: String,
    pub bot: Option<User>,
}
