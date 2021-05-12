use url::Url;

use crate::http::client::Http;
use crate::internal::prelude::*;
use crate::model::prelude::*;

/// A builder for constructing an invite link with custom OAuth2 scopes.
#[derive(Debug, Clone, Default)]
pub struct CreateBotAuthParameters {
    client_id: UserId,
    scopes: Vec<OAuth2Scope>,
    permissions: Permissions,
    guild_id: GuildId,
    disable_guild_select: bool,
}

impl CreateBotAuthParameters {
    /// Builds the url with the provided data.
    pub fn build(self) -> String {
        let mut valid_data = vec![];
        let bits = self.permissions.bits();

        if self.client_id.0 != 0 {
            valid_data.push(("cliend_id", self.client_id.0.to_string()));
        }

        if !self.scopes.is_empty() {
            valid_data.push((
                "scope",
                self.scopes.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(" "),
            ));
        }

        if bits != 0 {
            valid_data.push(("permissions", bits.to_string()));
        }

        if self.guild_id.0 != 0 {
            valid_data.push(("guild", self.guild_id.0.to_string()));
        }

        if self.disable_guild_select {
            valid_data.push(("disable_guild_select", self.disable_guild_select.to_string()));
        }

        let url = Url::parse_with_params("https://discord.com/api/oauth2/authorize", &valid_data)
            .expect("failed to construct URL");

        url.to_string()
    }

    /// Specify the client Id of your application.
    pub fn client_id<U: Into<UserId>>(&mut self, client_id: U) -> &mut Self {
        self.client_id = client_id.into();
        self
    }

    /// Automatically fetch and set the client Id of your application by inquiring Discord's API.
    ///
    /// # Errors
    ///
    /// Returns an
    /// [`HttpError::UnsuccessfulRequest(Unauthorized)`][`HttpError::UnsuccessfulRequest`]
    /// If the user is not authorized for this endpoint.
    ///
    /// [`HttpError::UnsuccessfulRequest`]: crate::http::HttpError::UnsuccessfulRequest
    pub async fn auto_client_id(&mut self, http: impl AsRef<Http>) -> Result<&mut Self> {
        self.client_id = http.as_ref().get_current_application_info().await.map(|v| v.id)?;
        Ok(self)
    }

    /// Specify the scopes for your application.
    ///
    /// **Note**: This needs to include the [`Bot`] scope.
    ///
    /// [`Bot`]: crate::model::oauth2::OAuth2Scope::Bot
    pub fn scopes(&mut self, scopes: &[OAuth2Scope]) -> &mut Self {
        self.scopes = scopes.to_vec();
        self
    }

    /// Specify the permissions your application requires.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self.permissions = permissions;
        self
    }

    /// Specify the Id of the guild to prefill the dropdown picker for the user.
    pub fn guild_id<G: Into<GuildId>>(&mut self, guild_id: G) -> &mut Self {
        self.guild_id = guild_id.into();
        self
    }

    /// Specify whether the user cannot change the guild in the dropdown picker.
    pub fn disable_guild_select(&mut self, disable: bool) -> &mut Self {
        self.disable_guild_select = disable;
        self
    }
}
