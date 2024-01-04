use std::borrow::Cow;

#[cfg(feature = "http")]
use super::Builder;
#[cfg(feature = "http")]
use crate::http::CacheHttp;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder to add parameters when using [`GuildId::add_member`].
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#add-guild-member).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct AddMember<'a> {
    access_token: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "<[RoleId]>::is_empty")]
    roles: Cow<'a, [RoleId]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
}

impl<'a> AddMember<'a> {
    /// Constructs a new builder with the given access token, leaving all other fields empty.
    pub fn new(access_token: impl Into<Cow<'a, str>>) -> Self {
        Self {
            access_token: access_token.into(),
            roles: Cow::default(),
            nick: None,
            mute: None,
            deaf: None,
        }
    }

    /// Sets the OAuth2 access token for this request, replacing the current one.
    ///
    /// Requires the access token to have the `guilds.join` scope granted.
    pub fn access_token(mut self, access_token: impl Into<Cow<'a, str>>) -> Self {
        self.access_token = access_token.into();
        self
    }

    /// Sets the member's nickname.
    ///
    /// Requires the [Manage Nicknames] permission.
    ///
    /// [Manage Nicknames]: crate::model::permissions::Permissions::MANAGE_NICKNAMES
    pub fn nickname(mut self, nickname: impl Into<Cow<'a, str>>) -> Self {
        self.nick = Some(nickname.into());
        self
    }

    /// Sets the list of roles that the member should have.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// [Manage Roles]: crate::model::permissions::Permissions::MANAGE_ROLES
    pub fn roles(mut self, roles: impl Into<Cow<'a, [RoleId]>>) -> Self {
        self.roles = roles.into();
        self
    }

    /// Whether to mute the member.
    ///
    /// Requires the [Mute Members] permission.
    ///
    /// [Mute Members]: crate::model::permissions::Permissions::MUTE_MEMBERS
    pub fn mute(mut self, mute: bool) -> Self {
        self.mute = Some(mute);
        self
    }

    /// Whether to deafen the member.
    ///
    /// Requires the [Deafen Members] permission.
    ///
    /// [Deafen Members]: crate::model::permissions::Permissions::DEAFEN_MEMBERS
    pub fn deafen(mut self, deafen: bool) -> Self {
        self.deaf = Some(deafen);
        self
    }
}

#[cfg(feature = "http")]
#[async_trait::async_trait]
impl Builder for AddMember<'_> {
    type Context<'ctx> = (GuildId, UserId);
    type Built = Option<Member>;

    /// Adds a [`User`] to this guild with a valid OAuth2 access token.
    ///
    /// Returns the created [`Member`] object, or nothing if the user is already a member of the
    /// guild.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Self::Built> {
        cache_http.http().add_guild_member(ctx.0, ctx.1, &self).await
    }
}
