use crate::model::prelude::*;

use nonmax::NonMaxU32;

use super::*;

/// Various information about integrations.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-object),
/// [extra fields 1](https://discord.com/developers/docs/topics/gateway-events#integration-create),
/// [extra fields 2](https://discord.com/developers/docs/topics/gateway-events#integration-update),
#[bool_to_bitflags::bool_to_bitflags]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct Integration {
    pub id: IntegrationId,
    pub name: FixedString,
    #[serde(rename = "type")]
    pub kind: FixedString,
    pub enabled: bool,
    pub syncing: Option<bool>,
    pub role_id: Option<RoleId>,
    pub enable_emoticons: Option<bool>,
    #[serde(rename = "expire_behavior")]
    pub expire_behaviour: Option<IntegrationExpireBehaviour>,
    pub expire_grace_period: Option<NonMaxU32>,
    pub user: Option<User>,
    pub account: IntegrationAccount,
    pub synced_at: Option<Timestamp>,
    pub subscriber_count: Option<NonMaxU64>,
    pub revoked: Option<bool>,
    pub application: Option<IntegrationApplication>,
    pub scopes: Option<Vec<Scope>>,
    /// Only present in [`IntegrationCreateEvent`] and [`IntegrationUpdateEvent`].
    pub guild_id: Option<GuildId>,
}

enum_number! {
    /// The behavior once the integration expires.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-object-integration-expire-behaviors).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
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
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationAccount {
    pub id: FixedString,
    pub name: FixedString,
}

/// Integration application object.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#integration-application-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntegrationApplication {
    pub id: ApplicationId,
    pub name: FixedString,
    pub icon: Option<ImageHash>,
    pub description: FixedString,
    pub bot: Option<User>,
}
