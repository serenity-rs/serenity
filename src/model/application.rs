//! Models about OAuth2 applications.

use std::fmt;

use super::{id::UserId, user::User, utils::*};

/// Information about a user's application. An application does not necessarily
/// have an associated bot user.
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApplicationInfo {
    /// The bot user associated with the application. See [`BotApplication`] for
    /// more information.
    pub bot: Option<BotApplication>,
    /// Indicator of whether the bot is public.
    ///
    /// If a bot is public, anyone may invite it to their [`Guild`]. While a bot
    /// is private, only the owner may add it to a guild.
    ///
    /// [`Guild`]: super::guild::Guild
    #[serde(default = "default_true")]
    pub bot_public: bool,
    /// Indicator of whether the bot requires an OAuth2 code grant.
    pub bot_require_code_grant: bool,
    /// A description of the application, assigned by the application owner.
    pub description: String,
    /// A set of bitflags assigned to the application, which represent gated
    /// feature flags that have been enabled for the application.
    pub flags: Option<u64>,
    /// A hash pointing to the application's icon.
    ///
    /// This is not necessarily equivalent to the bot user's avatar.
    pub icon: Option<String>,
    /// The unique numeric Id of the application.
    pub id: UserId,
    /// The name assigned to the application by the application owner.
    pub name: String,
    /// A list of redirect URIs assigned to the application.
    pub redirect_uris: Vec<String>,
    /// A list of RPC Origins assigned to the application.
    pub rpc_origins: Vec<String>,
    /// The application team group.
    pub team: Option<Vec<Team>>,
    /// The given secret to the application.
    ///
    /// This is not equivalent to the application's bot user's token.
    pub secret: String,
}

impl fmt::Debug for ApplicationInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApplicationInfo")
            .field("bot", &self.bot)
            .field("bot_public", &self.bot_public)
            .field("bot_require_code_grant", &self.bot_require_code_grant)
            .field("description", &self.description)
            .field("flags", &self.flags)
            .field("icon", &self.icon)
            .field("id", &self.id)
            .field("name", &self.name)
            .field("redirect_uris", &self.redirect_uris)
            .field("rpc_origins", &self.rpc_origins)
            .field("team", &self.team)
            .finish()
    }
}

/// Information about an application with an application's bot user.
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotApplication {
    /// The unique Id of the bot user.
    pub id: UserId,
    /// A hash of the avatar, if one is assigned.
    ///
    /// Can be used to generate a full URL to the avatar.
    pub avatar: Option<String>,
    /// Indicator of whether it is a bot.
    #[serde(default)]
    pub bot: bool,
    /// The discriminator assigned to the bot user.
    ///
    /// While discriminators are not unique, the `username#discriminator` pair
    /// is.
    pub discriminator: u16,
    /// The bot user's username.
    pub name: String,
    /// The token used to authenticate as the bot user.
    ///
    /// **Note**: Keep this information private, as untrusted sources can use it
    /// to perform any action with a bot user.
    pub token: String,
}

impl fmt::Debug for BotApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotApplication")
            .field("id", &self.id)
            .field("avatar", &self.avatar)
            .field("bot", &self.bot)
            .field("discriminator", &self.discriminator)
            .field("name", &self.name)
            .finish()
    }
}

/// Partial information about the given application.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialCurrentApplicationInfo {
    /// The unique Id of the user.
    pub id: UserId,
    /// The flags associated with the application.
    ///
    /// These flags are unknown and are not yet documented in the Discord API
    /// documentation.
    pub flags: u64,
}

/// Information about the current application and its owner.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CurrentApplicationInfo {
    pub description: String,
    pub icon: Option<String>,
    pub id: UserId,
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
    #[serde(deserialize_with = "deserialize_u64")]
    pub id: u64,
    /// The name of the team.
    pub name: String,
    /// The members of the team
    pub members: Vec<TeamMember>,
    /// The user id of the team owner.
    pub owner_user_id: UserId,
}

/// Infromation about a Member on a Team.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamMember {
    /// The member's membership state.
    pub membership_state: MembershipState,
    /// The list of permissions of the member on the team.
    ///
    /// NOTE: Will always be ["*"] for now.
    pub permissions: Vec<String>,
    /// The ID of the team they are a member of.
    #[serde(deserialize_with = "deserialize_u64")]
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
