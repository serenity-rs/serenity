#[cfg(feature = "model")]
use crate::builder::{Builder as _, GetEntitlements};
#[cfg(feature = "model")]
use crate::http::CacheHttp;
use crate::model::prelude::*;

/// A premium offering that can be made available to an application's users and guilds.
///
/// [Discord docs](https://discord.com/developers/docs/monetization/skus#sku-object).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sku {
    /// The unique ID of the SKU.
    pub id: SkuId,
    /// The class of the SKU.
    #[serde(rename = "type")]
    pub kind: SkuKind,
    /// Id of the SKU's parent application.
    pub application_id: ApplicationId,
    /// The customer-facing name of the premium offering.
    pub name: String,
    /// A system-generated URL slug based on the SKU.
    pub slug: String,
    /// Flags indicating the type of subscription the SKU represents.
    pub flags: SkuFlags,
}

impl Sku {
    /// Returns the store url for this SKU. If included in a message, will render as a rich embed.
    /// See the [Discord docs] for details.
    ///
    /// [Discord docs]: https://discord.com/developers/docs/monetization/skus#linking-to-your-skus
    #[must_use]
    pub fn url(&self) -> String {
        format!(
            "https://discord.com/application-directory/{}/store/{}",
            self.application_id, self.id
        )
    }
}

enum_number! {
    /// Differentiates between SKU classes.
    ///
    /// [Discord docs](https://discord.com/developers/docs/monetization/skus#sku-object-sku-types).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum SkuKind {
        /// Represents a recurring subscription.
        Subscription = 5,
        /// A system-generated group for each SKU created of type [`SkuKind::Subscription`].
        SubscriptionGroup = 6,
        _ => Unknown(u8),
    }
}

bitflags! {
    /// Differentates between user and server subscriptions.
    ///
    /// [Discord docs](https://discord.com/developers/docs/monetization/skus#sku-object-sku-flags).
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    pub struct SkuFlags: u64 {
        /// SKU is available for purchase.
        const AVAILABLE = 1 << 2;
        /// Recurring SKU that can be purchased by a user and applied to a single server. Grants
        /// access to every user in that server.
        const GUILD_SUBSCRIPTION = 1 << 7;
        /// Recurring SKU purchased by a user for themselves. Grants access to the purchasing user
        /// in every server.
        const USER_SUBSCRIPTION = 1 << 8;
    }
}

/// Represents that a user or guild has access to a premium offering in the application.
///
/// [Discord docs](https://discord.com/developers/docs/monetization/entitlements#entitlement-object-entitlement-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entitlement {
    /// The ID of the entitlement.
    pub id: EntitlementId,
    /// The ID of the corresponding SKU.
    pub sku_id: SkuId,
    /// The ID of the parent application.
    pub application_id: ApplicationId,
    /// The ID of the user that is granted access to the SKU.
    pub user_id: Option<UserId>,
    /// The type of the entitlement.
    #[serde(rename = "type")]
    pub kind: EntitlementKind,
    /// Whether the entitlement has been deleted or not. Entitlements are not deleted when they
    /// expire.
    pub deleted: bool,
    /// Start date after which the entitlement is valid. Not present when using test entitlements.
    pub starts_at: Option<Timestamp>,
    /// End date after which the entitlement is no longer valid. Not present when using test
    /// entitlements.
    pub ends_at: Option<Timestamp>,
    /// The ID of the guild that is granted access to the SKU.
    pub guild_id: Option<GuildId>,
}

impl Entitlement {
    /// Returns a link to the SKU corresponding to this entitlement. See [`Sku::url`] for details.
    #[must_use]
    pub fn sku_url(&self) -> String {
        format!(
            "https://discord.com/application-directory/{}/store/{}",
            self.application_id, self.sku_id
        )
    }

    /// Returns all entitlements for the current application, active and expired.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "model")]
    pub async fn list(
        cache_http: impl CacheHttp,
        builder: GetEntitlements<'_>,
    ) -> Result<Vec<Entitlement>> {
        builder.execute(cache_http, ()).await
    }
}

enum_number! {
    /// Differentiates between Entitlement types.
    ///
    /// [Discord docs](https://discord.com/developers/docs/monetization/entitlements#entitlement-object-entitlement-types).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum EntitlementKind {
        /// Entitlement was purchased as an app subscription.
        ApplicationSubscription = 8,
        _ => Unknown(u8),
    }
}

pub enum EntitlementOwner {
    Guild(GuildId),
    User(UserId),
}
