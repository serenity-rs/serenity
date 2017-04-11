use super::*;

/// Various information about integrations.
#[derive(Clone, Debug, Deserialize)]
pub struct Integration {
    pub id: IntegrationId,
    pub account: IntegrationAccount,
    pub enabled: bool,
    #[serde(rename="expire_behaviour")]
    pub expire_behaviour: u64,
    pub expire_grace_period: u64,
    pub kind: String,
    pub name: String,
    pub role_id: RoleId,
    pub synced_at: u64,
    pub syncing: bool,
    pub user: User,
}

impl From<Integration> for IntegrationId {
    /// Gets the Id of integration.
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

/// Integration account object.
#[derive(Clone, Debug, Deserialize)]
pub struct IntegrationAccount {
    pub id: String,
    pub name: String,
}
