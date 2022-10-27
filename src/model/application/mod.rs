//! Models about OAuth2 applications.

pub mod command;
pub mod component;
pub mod interaction;
pub mod oauth;

pub use interaction::{Interaction, InteractionType, MessageFlags, MessageInteraction};

use self::oauth::Scope;
use super::id::{ApplicationId, GenericId, GuildId, SkuId, UserId};
use super::user::User;
use super::Permissions;

/// Partial information about the given application.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub name: String,
    pub icon: Option<String>,
    pub description: String,
    #[serde(default)]
    pub rpc_origins: Vec<String>,
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    #[serde(default)]
    pub terms_of_service_url: Option<String>,
    #[serde(default)]
    pub privacy_policy_url: Option<String>,
    // TODO: this is an optional field according to Discord and should be Option<User>
    pub owner: User,
    pub verify_key: String,
    pub team: Option<Team>,
    #[serde(default)]
    pub guild_id: Option<GuildId>,
    #[serde(default)]
    pub primary_sku_id: Option<SkuId>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub cover_image: Option<String>,
    #[serde(default)]
    pub flags: Option<ApplicationFlags>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub install_params: Option<InstallParams>,
    #[serde(default)]
    pub custom_install_url: Option<String>,
}

/// Information about the Team group of the application.
///
/// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-team-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Team {
    /// The icon of the team.
    pub icon: Option<String>,
    /// The snowflake ID of the team.
    pub id: GenericId,
    /// The name of the team.
    pub name: String,
    /// The members of the team
    pub members: Vec<TeamMember>,
    /// The user id of the team owner.
    pub owner_user_id: UserId,
}

/// Information about a Member on a Team.
///
/// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-team-member-object).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamMember {
    /// The member's membership state.
    pub membership_state: MembershipState,
    /// The list of permissions of the member on the team.
    ///
    /// NOTE: Will always be ["*"] for now.
    pub permissions: Vec<String>,
    /// The ID of the team they are a member of.
    pub team_id: GenericId,
    /// The user type of the team member.
    pub user: User,
}

enum_number! {
    /// [Discord docs](https://discord.com/developers/docs/topics/teams#data-models-membership-state-enum).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum MembershipState {
        Invited = 1,
        Accepted = 2,
        _ => Unknown(u8),
    }
}

bitflags! {
    /// The flags of the application.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/application#application-object-application-flags).
    #[derive(Default)]
    pub struct ApplicationFlags: u64 {
        const GATEWAY_PRESENCE = 1 << 12;
        const GATEWAY_PRESENCE_LIMITED = 1 << 13;
        const GATEWAY_GUILD_MEMBERS = 1 << 14;
        const GATEWAY_GUILD_MEMBERS_LIMITED = 1 << 15;
        const VERIFICATION_PENDING_GUILD_LIMIT = 2 << 16;
        const EMBEDDED = 2 << 17;
        const GATEWAY_MESSAGE_CONTENT = 2 << 18;
        const GATEWAY_MESSAGE_CONTENT_LIMITED = 2 << 19;
    }
}

/// Settings for the application's default in-app authorization link
///
/// [Discord docs](https://discord.com/developers/docs/resources/application#install-params-object-install-params-structure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallParams {
    pub scopes: Vec<Scope>,
    pub permissions: Permissions,
}
