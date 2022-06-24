use super::*;
use crate::model::Timestamp;

/// Various information about integrations.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-object).
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
    pub synced_at: Option<Timestamp>,
    pub syncing: Option<bool>,
    pub user: Option<User>,
    pub enable_emoticons: Option<bool>,
    pub subscriber_count: Option<u64>,
    pub revoked: Option<bool>,
    pub application: Option<IntegrationApplication>,
}

enum_number! {
    /// The behavior once the integration expires.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-object-integration-expire-behaviors).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum IntegrationExpireBehaviour {
        RemoveRole = 0,
        Kick = 1,
        _ => Unknown(u8),
    }
}

impl From<Integration> for IntegrationId {
    /// Gets the Id of integration.
    fn from(integration: Integration) -> IntegrationId {
        integration.id
    }
}

/// Integration account object.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-account-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationAccount {
    pub id: String,
    pub name: String,
}

/// Integration application object.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-application-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationApplication {
    pub id: ApplicationId,
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    pub bot: Option<User>,
}
