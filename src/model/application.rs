//! Models about OAuth2 applications.

use super::id::{snowflake, ApplicationId, UserId};
use super::user::User;

/// Partial information about the given application.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialCurrentApplicationInfo {
    /// The unique Id of the user.
    pub id: ApplicationId,
    /// The flags associated with the application.
    pub flags: ApplicationFlags,
}

/// Information about the current application and its owner.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CurrentApplicationInfo {
    pub description: String,
    pub icon: Option<String>,
    pub id: ApplicationId,
    pub name: String,
    pub owner: User,
    #[serde(default)]
    pub rpc_origins: Vec<String>,
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    pub team: Option<Team>,
}

/// Information about the Team group of the application.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Team {
    /// The icon of the team.
    pub icon: Option<String>,
    /// The snowflake ID of the team.
    #[serde(with = "snowflake")]
    pub id: u64,
    /// The name of the team.
    pub name: String,
    /// The members of the team
    pub members: Vec<TeamMember>,
    /// The user id of the team owner.
    pub owner_user_id: UserId,
}

/// Information about a Member on a Team.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamMember {
    /// The member's membership state.
    pub membership_state: MembershipState,
    /// The list of permissions of the member on the team.
    ///
    /// NOTE: Will always be ["*"] for now.
    pub permissions: Vec<String>,
    /// The ID of the team they are a member of.
    #[serde(with = "snowflake")]
    pub team_id: u64,
    /// The user type of the team member.
    pub user: User,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum MembershipState {
    Invited = 1,
    Accepted = 2,
    Unknown = !0,
}

enum_number!(MembershipState {
    Invited,
    Accepted
});

bitflags! {
    /// The flags of the application.
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
