use std::borrow::Cow;

use arrayvec::ArrayVec;
use url::Url;

#[cfg(feature = "http")]
use crate::http::Http;
#[cfg(feature = "http")]
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for constructing an invite link with custom OAuth2 scopes.
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct CreateBotAuthParameters<'a> {
    client_id: Option<ApplicationId>,
    scopes: Cow<'a, [Scope]>,
    permissions: Permissions,
    guild_id: Option<GuildId>,
    disable_guild_select: bool,
}

impl<'a> CreateBotAuthParameters<'a> {
    /// Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the url with the provided data.
    #[must_use]
    pub fn build(self) -> String {
        let mut valid_data = ArrayVec::<_, 5>::new();
        let bits = self.permissions.bits();

        if let Some(client_id) = self.client_id {
            valid_data.push(("client_id", client_id.to_string()));
        }

        if !self.scopes.is_empty() {
            valid_data.push(("scope", join_to_string(',', self.scopes.iter())));
        }

        if bits != 0 {
            valid_data.push(("permissions", bits.to_string()));
        }

        if let Some(guild_id) = self.guild_id {
            valid_data.push(("guild", guild_id.to_string()));
        }

        if self.disable_guild_select {
            valid_data.push(("disable_guild_select", self.disable_guild_select.to_string()));
        }

        let url = Url::parse_with_params("https://discord.com/api/oauth2/authorize", &valid_data)
            .expect("failed to construct URL");

        url.to_string()
    }

    /// Specify the client Id of your application.
    pub fn client_id(mut self, client_id: ApplicationId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Automatically fetch and set the client Id of your application by inquiring Discord's API.
    ///
    /// # Errors
    ///
    /// Returns an [`HttpError::UnsuccessfulRequest`] if the user is not authorized for this
    /// endpoint.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    #[cfg(feature = "http")]
    pub async fn auto_client_id(mut self, http: &Http) -> Result<Self> {
        self.client_id = http.get_current_application_info().await.map(|v| Some(v.id))?;
        Ok(self)
    }

    /// Specify the scopes for your application.
    ///
    /// **Note**: This needs to include the [`Bot`] scope.
    ///
    /// [`Bot`]: Scope::Bot
    pub fn scopes(mut self, scopes: impl Into<Cow<'a, [Scope]>>) -> Self {
        self.scopes = scopes.into();
        self
    }

    /// Specify the permissions your application requires.
    pub fn permissions(mut self, permissions: Permissions) -> Self {
        self.permissions = permissions;
        self
    }

    /// Specify the Id of the guild to prefill the dropdown picker for the user.
    pub fn guild_id(mut self, guild_id: GuildId) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    /// Specify whether the user cannot change the guild in the dropdown picker.
    pub fn disable_guild_select(mut self, disable: bool) -> Self {
        self.disable_guild_select = disable;
        self
    }
}
