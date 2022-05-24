use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};

#[cfg(feature = "http")]
use crate::builder::CreateAutocompleteResponse;
#[cfg(feature = "http")]
use crate::http::Http;
use crate::internal::prelude::*;
#[cfg(feature = "http")]
use crate::json;
use crate::json::prelude::*;
use crate::model::application::interaction::application_command::CommandData;
#[cfg(feature = "http")]
use crate::model::application::interaction::InteractionResponseType;
use crate::model::application::interaction::InteractionType;
use crate::model::guild::Member;
use crate::model::id::{ApplicationId, ChannelId, GuildId, InteractionId};
use crate::model::user::User;

/// An interaction received when the user fills in an autocomplete option
#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct AutocompleteInteraction {
    /// Id of the interaction.
    pub id: InteractionId,
    /// Id of the application this interaction is for.
    pub application_id: ApplicationId,
    /// The type of interaction.
    #[serde(rename = "type")]
    pub kind: InteractionType,
    /// The data of the interaction which was triggered.
    pub data: CommandData,
    /// The guild Id this interaction was sent from, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<GuildId>,
    /// The channel Id this interaction was sent from.
    pub channel_id: ChannelId,
    /// The `member` data for the invoking user.
    ///
    /// **Note**: It is only present if the interaction is triggered in a guild.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<Member>,
    /// The `user` object for the invoking user.
    pub user: User,
    /// A continuation token for responding to the interaction.
    pub token: String,
    /// Always `1`.
    pub version: u8,
    /// The guild's preferred locale.
    pub guild_locale: Option<String>,
    /// The selected language of the invoking user.
    pub locale: String,
}

#[cfg(feature = "http")]
impl AutocompleteInteraction {
    /// Creates a response to an autocomplete interaction.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Http`] if the API returns an error.
    ///
    /// [`Error::Http`]: crate::error::Error::Http
    pub async fn create_autocomplete_response<F>(&self, http: impl AsRef<Http>, f: F) -> Result<()>
    where
        F: FnOnce(&mut CreateAutocompleteResponse) -> &mut CreateAutocompleteResponse,
    {
        let mut response = CreateAutocompleteResponse::default();
        f(&mut response);
        let data = json::hashmap_to_json_map(response.0);

        let map = json!({
            "type": InteractionResponseType::Autocomplete as u8,
            "data": data,
        });

        http.as_ref().create_interaction_response(self.id.0, &self.token, &map).await
    }
}

impl<'de> Deserialize<'de> for AutocompleteInteraction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("guild_id").and_then(Value::as_str).and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(member) = map.get_mut("member").and_then(Value::as_object_mut) {
                member.insert("guild_id".to_string(), from_number(guild_id));
            }

            if let Some(data) = map.get_mut("data") {
                if let Some(resolved) = data.get_mut("resolved") {
                    if let Some(roles) = resolved.get_mut("roles") {
                        if let Some(values) = roles.as_object_mut() {
                            for value in values.values_mut() {
                                value.as_object_mut().expect("couldn't deserialize").insert(
                                    "guild_id".to_string(),
                                    Value::from(guild_id.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }

        let id = map
            .remove("id")
            .ok_or_else(|| DeError::custom("expected id"))
            .and_then(InteractionId::deserialize)
            .map_err(DeError::custom)?;

        let application_id = map
            .remove("application_id")
            .ok_or_else(|| DeError::custom("expected application id"))
            .and_then(ApplicationId::deserialize)
            .map_err(DeError::custom)?;

        let kind = map
            .remove("type")
            .ok_or_else(|| DeError::custom("expected type"))
            .and_then(InteractionType::deserialize)
            .map_err(DeError::custom)?;

        let data = map
            .remove("data")
            .ok_or_else(|| DeError::custom("expected data"))
            .and_then(CommandData::deserialize)
            .map_err(DeError::custom)?;

        let guild_id = map
            .remove("guild_id")
            .map(GuildId::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let channel_id = map
            .remove("channel_id")
            .ok_or_else(|| DeError::custom("expected channel_id"))
            .and_then(ChannelId::deserialize)
            .map_err(DeError::custom)?;

        let member =
            map.remove("member").map(Member::deserialize).transpose().map_err(DeError::custom)?;

        let user =
            map.remove("user").map(User::deserialize).transpose().map_err(DeError::custom)?;

        let user = user
            .or_else(|| member.as_ref().map(|m| m.user.clone()))
            .ok_or_else(|| DeError::custom("expected user or member"))?;

        let token = map
            .remove("token")
            .ok_or_else(|| DeError::custom("expected token"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        let version = map
            .remove("version")
            .ok_or_else(|| DeError::custom("expected version"))
            .and_then(u8::deserialize)
            .map_err(DeError::custom)?;

        let guild_locale = map
            .remove("guild_locale")
            .map(String::deserialize)
            .transpose()
            .map_err(DeError::custom)?;

        let locale = map
            .remove("locale")
            .ok_or_else(|| DeError::custom("expected locale"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;

        Ok(Self {
            id,
            application_id,
            kind,
            data,
            guild_id,
            channel_id,
            member,
            user,
            token,
            version,
            guild_locale,
            locale,
        })
    }
}
