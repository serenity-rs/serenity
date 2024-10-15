#[cfg(feature = "model")]
use crate::builder::{Builder as _, GetEntitlements};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
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
        /// A durable one-time purchase.
        Durable = 2,
        /// A consumable one-time purchase.
        Consumable = 3,
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
    /// For consumable items, whether or not the entitlement has been consumed.
    pub consumed: Option<bool>,
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

    /// For a one-time purchase consumable SKU (of kind [`Consumable`]), marks the entitlement as
    /// consumed. On success, the [`consumed`] field will be set to `Some(true)`.
    ///
    /// # Errors
    ///
    /// Will fail if the corresponding SKU is not of kind [`Consumable`].
    ///
    /// [`Consumable`]: SkuKind::Consumable
    /// [`consumed`]: Entitlement::consumed
    #[cfg(feature = "model")]
    pub async fn consume(&mut self, http: &Http) -> Result<()> {
        http.consume_entitlement(self.id).await?;
        self.consumed = Some(true);
        Ok(())
    }

    /// Returns all entitlements for the current application, active and expired.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "model")]
    pub async fn list(
        cache_http: impl CacheHttp,
        builder: GetEntitlements,
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
        /// Entitlement was purchased by a user.
        Purchase = 1,
        /// Entitlement for a Discord Nitro subscription.
        PremiumSubscription = 2,
        /// Entitlement was gifted by an app developer.
        DeveloperGift = 3,
        /// Entitlement was purchased by a developer in application test mode.
        TestModePurchase = 4,
        /// Entitlement was granted when the corresponding SKU was free.
        FreePurchase = 5,
        /// Entitlement was gifted by another user.
        UserGift = 6,
        /// Entitlement was claimed by user for free as a Nitro Subscriber.
        PremiumPurchase = 7,
        /// Entitlement was purchased as an app subscription.
        ApplicationSubscription = 8,
        _ => Unknown(u8),
    }
}

pub enum EntitlementOwner {
    Guild(GuildId),
    User(UserId),
}
