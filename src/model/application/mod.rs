//! Models about OAuth2 applications.

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

use super::id::{ApplicationId, GenericId, GuildId, SkuId, UserId};
use super::misc::ImageHash;
use super::user::User;
use super::Permissions;
use crate::internal::prelude::*;

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
    /// The list of permissions of the member on the team.
    ///
    /// NOTE: Will always be ["*"] for now.
    pub permissions: FixedArray<String>,
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
