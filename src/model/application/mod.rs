//! Models about OAuth2 applications.

use std::collections::HashMap;

mod command;
pub use command::*;
mod command_interaction;
pub use command_interaction::*;
mod component;
pub use component::*;
mod component_interaction;
pub use component_interaction::*;
mod interaction;
pub use interaction::*;
mod modal_interaction;
pub use modal_interaction::*;
mod oauth;
pub use oauth::*;
mod ping_interaction;
pub use ping_interaction::*;

use super::prelude::*;

/// Partial information about the given application.
///
/// Discord docs: [application field of Ready](https://discord.com/developers/docs/topics/gateway-events#ready-ready-event-fields)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PartialCurrentApplicationInfo {
    /// The unique Id of the user.
    pub id: ApplicationId,
    /// The flags associated with the application.
    pub flags: ApplicationFlags,
}

/// Information about the current application and its owner.
///
/// [Discord docs](https://discord.com/developers/docs/resources/application#application-object-application-structure).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CurrentApplicationInfo {
    pub id: ApplicationId,
    pub name: FixedString,
    pub icon: Option<ImageHash>,
    pub description: FixedString,
    #[serde(default)]
    pub rpc_origins: FixedArray<String>,
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    #[serde(default)]
    pub terms_of_service_url: Option<FixedString>,
    #[serde(default)]
    pub privacy_policy_url: Option<FixedString>,
    pub owner: Option<User>,
    // omitted `summary` because it deprecated
    pub verify_key: FixedString,
    pub team: Option<Team>,
    #[serde(default)]
    pub guild_id: Option<GuildId>,
    #[serde(default)]
    pub primary_sku_id: Option<SkuId>,
    #[serde(default)]
    pub slug: Option<FixedString>,
    #[serde(default)]
    pub cover_image: Option<FixedString>,
    #[serde(default)]
    pub flags: Option<ApplicationFlags>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub install_params: Option<InstallParams>,
    #[serde(default)]
    pub custom_install_url: Option<FixedString>,
    /// The application's role connection verification entry point, which when configured will
    /// render the app as a verification method in the guild role verification configuration.
    pub role_connections_verification_url: Option<FixedString>,
    #[serde(default)]
    pub integration_types_config: HashMap<InstallationContext, InstallationContextConfig>,
    pub approximate_guild_count: Option<u32>,
    pub approximate_user_install_count: Option<u32>,
    pub guild: Option<PartialGuild>,
    pub redirect_uris: Option<Vec<String>>,
    pub interactions_endpoint_url: Option<String>,
}

impl ApplicationId {
    /// Returns the store url for the application.
    ///
    /// If included in a message, will render as a rich embed. See the [Discord docs] for details.
    ///
    /// [Discord docs]: https://discord.com/developers/docs/monetization/managing-your-store#linking-to-your-store
    #[must_use]
    pub fn store_url(self) -> String {
        format!("https://discord.com/application-directory/{self}/store")
    }
}

enum_number! {
    /// An enum representing the [installation contexts].
    ///
    /// [interaction contexts](https://discord.com/developers/docs/resources/application#application-object-application-integration-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InstallationContext {
        Guild = 0,
        User = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// An enum representing the different [interaction contexts].
    ///
    /// [interaction contexts](https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-context-types).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[non_exhaustive]
    pub enum InteractionContext {
        /// Interaction can be used within servers
        Guild = 0,
        /// Interaction can be used within DMs with the app's bot user
        BotDm = 1,
        /// Interaction can be used within Group DMs and DMs other than the app's bot user
        PrivateChannel = 2,
        _ => Unknown(u8),
    }
}

/// Information about how the [`CurrentApplicationInfo`] is installed.
///
/// [Discord docs](https://discord.com/developers/docs/resources/application#application-object-application-integration-types).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstallationContextConfig {
    pub oauth2_install_params: Option<InstallParams>,
}

/// Information about the Team group of the application.
///
/// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-team-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Team {
    /// The icon of the team.
    pub icon: Option<ImageHash>,
    /// The snowflake ID of the team.
    pub id: GenericId,
    /// The name of the team.
    pub name: FixedString,
    /// The members of the team
    pub members: FixedArray<TeamMember>,
    /// The user id of the team owner.
    pub owner_user_id: UserId,
}

/// Information about a Member on a Team.
///
/// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-team-member-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct TeamMember {
    /// The member's membership state.
    pub membership_state: MembershipState,
    /// The ID of the team they are a member of.
    pub team_id: GenericId,
    /// The user type of the team member.
    pub user: User,
    /// The [`TeamMemberRole`] of the team member.
    pub role: TeamMemberRole,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-membership-state-enum).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[non_exhaustive]
    pub enum MembershipState {
        Invited = 1,
        Accepted = 2,
        _ => Unknown(u8),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TeamMemberRole {
    Admin,
    Developer,
    ReadOnly,
    #[serde(untagged)]
    Other(String),
}

impl TeamMemberRole {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Admin => 3,
            Self::Developer => 2,
            Self::ReadOnly => 1,
            Self::Other(_) => 0,
        }
    }
}

impl PartialEq for TeamMemberRole {
    fn eq(&self, other: &Self) -> bool {
        self.discriminant() == other.discriminant()
    }
}

impl Eq for TeamMemberRole {}

impl PartialOrd for TeamMemberRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TeamMemberRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.discriminant().cmp(&other.discriminant())
    }
}

bitflags! {
    /// The flags of the application.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/application#application-object-application-flags).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
    pub struct ApplicationFlags: u64 {
        /// Indicates if an app uses the Auto Moderation API
        const APPLICATION_AUTO_MODERATION_RULE_CREATE_BADGE = 1 << 6;
        /// Intent required for bots in 100 or more servers to receive presence_update events
        const GATEWAY_PRESENCE = 1 << 12;
        /// Intent required for bots in under 100 servers to receive presence_update events, found
        /// on the Bot page in your app's settings
        const GATEWAY_PRESENCE_LIMITED = 1 << 13;
        /// Intent required for bots in 100 or more servers to receive member-related events like
        /// guild_member_add. See the list of member-related events under [GUILD_MEMBERS](https://discord.com/developers/docs/topics/gateway#list-of-intents)
        const GATEWAY_GUILD_MEMBERS = 1 << 14;
        /// Intent required for bots in under 100 servers to receive member-related events like
        /// guild_member_add, found on the Bot page in your app's settings. See the list of
        /// member-related events under [GUILD_MEMBERS](https://discord.com/developers/docs/topics/gateway#list-of-intents)
        const GATEWAY_GUILD_MEMBERS_LIMITED = 1 << 15;
        /// Indicates unusual growth of an app that prevents verification
        const VERIFICATION_PENDING_GUILD_LIMIT = 1 << 16;
        /// Indicates if an app is embedded within the Discord client (currently unavailable
        /// publicly)
        const EMBEDDED = 1 << 17;
        /// Intent required for bots in 100 or more servers to receive [message content](https://support-dev.discord.com/hc/en-us/articles/4404772028055).
        const GATEWAY_MESSAGE_CONTENT = 1 << 18;
        /// Intent required for bots in under 100 servers to receive [message content](https://support-dev.discord.com/hc/en-us/articles/4404772028055),
        /// found on the Bot page in your app's settings
        const GATEWAY_MESSAGE_CONTENT_LIMITED = 1 << 19;
        /// Indicates if an app has registered global application commands
        const APPLICATION_COMMAND_BADGE = 1 << 19;
    }
}

/// Settings for the application's default in-app authorization link
///
/// [Discord docs](https://discord.com/developers/docs/resources/application#install-params-object-install-params-structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstallParams {
    pub scopes: FixedArray<Scope>,
    pub permissions: Permissions,
}

#[cfg(test)]
mod team_role_ordering {
    use super::TeamMemberRole;

    fn other(val: &str) -> TeamMemberRole {
        TeamMemberRole::Other(String::from(val))
    }

    #[test]
    fn test_normal_ordering() {
        let mut roles = [
            TeamMemberRole::Developer,
            TeamMemberRole::Admin,
            other(""),
            TeamMemberRole::ReadOnly,
            other("test"),
        ];

        roles.sort();

        assert_eq!(roles, [
            other(""),
            other("test"),
            TeamMemberRole::ReadOnly,
            TeamMemberRole::Developer,
            TeamMemberRole::Admin,
        ]);
    }

    #[test]
    fn test_other_eq() {
        assert_eq!(other("").cmp(&other("")), std::cmp::Ordering::Equal);
    }
}
