use crate::http::client::Http;
use crate::internal::prelude::*;
use crate::model::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct CreateBotAuthParameters {
    _client_id: UserId,
    _scopes: Vec<Oauth2Scope>,
    _permissions: Permissions,
    _guild_id: GuildId,
    _disable_guild_select: bool,
}

impl CreateBotAuthParameters {
    /// Builds the url with the provided data.
    pub fn build(self) -> String {
        let mut valid_data = vec![];
        let bits = self._permissions.bits();

        if self._client_id.0 != 0 {
            valid_data.push(("cliend_id", self._client_id.0.to_string()));
        }

        if !self._scopes.is_empty() {
            valid_data.push((
                "scope",
                self._scopes.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(" "),
            ));
        }

        if bits != 0 {
            valid_data.push(("permissions", bits.to_string()));
        }

        if self._guild_id.0 != 0 {
            valid_data.push(("guild", self._guild_id.0.to_string()));
        }

        if self._disable_guild_select {
            valid_data.push(("disable_guild_select", self._disable_guild_select.to_string()));
        }

        let url = reqwest::Url::parse_with_params(
            "https://discord.com/api/oauth2/authorize",
            &valid_data,
        )
        .expect("Wait what");

        url.to_string()
    }

    /// Your app's client id.
    pub fn client_id<U: Into<UserId>>(&mut self, client_id: U) -> &mut Self {
        self._client_id = client_id.into();
        self
    }

    /// Will try to fill the client_id automatically from the current application.
    pub async fn auto_client_id(&mut self, http: impl AsRef<Http>) -> Result<&mut Self> {
        self._client_id = http.as_ref().get_current_application_info().await.map(|v| v.id)?;
        Ok(self)
    }

    /// Needs to include `bot` for the bot flow.
    pub fn scopes_owned(&mut self, scopes: Vec<Oauth2Scope>) -> &mut Self {
        self._scopes = scopes;
        self
    }

    pub fn scopes(&mut self, scopes: &[Oauth2Scope]) -> &mut Self {
        self._scopes = scopes.to_vec();
        self
    }

    /// The permissions you're requesting.
    pub fn permissions(&mut self, permissions: Permissions) -> &mut Self {
        self._permissions = permissions;
        self
    }

    /// Pre-fills the dropdown picker with a guild for the user.
    pub fn guild_id<G: Into<GuildId>>(&mut self, guild_id: G) -> &mut Self {
        self._guild_id = guild_id.into();
        self
    }

    /// If set to `true`, disallows the user from changing the guild dropdown.
    pub fn disable_guild_select(&mut self, disable: bool) -> &mut Self {
        self._disable_guild_select = disable;
        self
    }
}
