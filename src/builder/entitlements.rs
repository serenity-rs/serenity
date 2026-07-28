use std::borrow::Cow;

use nonmax::NonMaxU8;

#[cfg(feature = "http")]
use crate::http::Http;
use crate::model::prelude::*;

/// Builds a request to create a test [`Entitlement`].
///
/// This is a helper for [`Http::create_test_entitlement`].
///
/// [`Http::create_test_entitlement`]: crate::http::Http::create_test_entitlement
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateTestEntitlement {
    sku_id: SkuId,
    owner_id: GenericId,
    owner_type: u8,
}

impl CreateTestEntitlement {
    pub fn new(sku_id: SkuId, owner: EntitlementOwner) -> Self {
        let (owner_id, owner_type) = owner.deconstruct();

        Self {
            sku_id,
            owner_id,
            owner_type,
        }
    }

    /// Creates a test entitlement.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http) -> Result<Entitlement> {
        http.create_test_entitlement(&self).await
    }
}

pub enum EntitlementOwner {
    Guild(GuildId),
    User(UserId),
}

impl EntitlementOwner {
    fn deconstruct(self) -> (GenericId, u8) {
        match self {
            EntitlementOwner::Guild(id) => (id.get().into(), 1),
            EntitlementOwner::User(id) => (id.get().into(), 2),
        }
    }
}

/// Builds a request to fetch active and ended [`Entitlement`]s.
///
/// This is a helper for [`Http::get_entitlements`] used via [`Entitlement::list`].
///
/// [`Http::get_entitlements`]: crate::http::Http::get_entitlements
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct GetEntitlements<'a> {
    user_id: Option<UserId>,
    sku_ids: Option<Cow<'a, [SkuId]>>,
    before: Option<EntitlementId>,
    after: Option<EntitlementId>,
    limit: Option<NonMaxU8>,
    guild_id: Option<GuildId>,
    exclude_ended: Option<bool>,
}

impl<'a> GetEntitlements<'a> {
    /// Filters the returned entitlements by the given [`UserId`].
    pub fn user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Filters the returned entitlements by the given [`SkuId`]s.
    pub fn sku_ids(mut self, sku_ids: impl Into<Cow<'a, [SkuId]>>) -> Self {
        self.sku_ids = Some(sku_ids.into());
        self
    }

    /// Filters the returned entitlements to before the given [`EntitlementId`].
    pub fn before(mut self, before: EntitlementId) -> Self {
        self.before = Some(before);
        self
    }

    /// Filters the returned entitlements to after the given [`EntitlementId`].
    pub fn after(mut self, after: EntitlementId) -> Self {
        self.after = Some(after);
        self
    }

    /// Limits the number of entitlements that may be returned.
    ///
    /// This is limited to `0..=100`.
    pub fn limit(mut self, limit: NonMaxU8) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Filters the returned entitlements by the given [`GuildId`].
    pub fn guild_id(mut self, guild_id: GuildId) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    /// Filters the returned entitlements to only active entitlements, if `true`.
    pub fn exclude_ended(mut self, exclude_ended: bool) -> Self {
        self.exclude_ended = Some(exclude_ended);
        self
    }

    pub fn into_owned(self) -> GetEntitlements<'static> {
        let Self {
            user_id,
            sku_ids,
            before,
            after,
            limit,
            guild_id,
            exclude_ended,
        } = self;
        GetEntitlements {
            user_id,
            sku_ids: sku_ids.map(|s| Cow::Owned(s.into_owned())),
            before,
            after,
            limit,
            guild_id,
            exclude_ended,
        }
    }

    /// Returns all entitlements for the current application, active and expired.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "http")]
    pub async fn execute(self, http: &Http) -> Result<Vec<Entitlement>> {
        http.get_entitlements(
            self.user_id,
            self.sku_ids.as_deref(),
            self.before,
            self.after,
            self.limit,
            self.guild_id,
            self.exclude_ended,
        )
        .await
    }
}
