//! Models about OAuth2 applications.

use super::{
    id::UserId,
    user::User,
    utils::default_true
};

/// Information about a user's application. An application does not necessarily
/// have an associated bot user.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationInfo {
    /// The bot user associated with the application. See [`BotApplication`] for
    /// more information.
    ///
    /// [`BotApplication`]: struct.BotApplication.html
    pub bot: Option<BotApplication>,
    /// Indicator of whether the bot is public.
    ///
    /// If a bot is public, anyone may invite it to their [`Guild`]. While a bot
    /// is private, only the owner may add it to a guild.
    ///
    /// [`Guild`]: ../guild/struct.Guild.html
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
    /// The given secret to the application.
    ///
    /// This is not equivalent to the application's bot user's token.
    pub secret: String,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// Information about an application with an application's bot user.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}

/// Information about the current application and its owner.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentApplicationInfo {
    pub description: String,
    pub icon: Option<String>,
    pub id: UserId,
    pub name: String,
    pub owner: User,
    #[serde(default)] pub rpc_origins: Vec<String>,
    pub bot_public: bool,
    pub bot_require_code_grant: bool,
    #[serde(skip)]
    pub(crate) _nonexhaustive: (),
}
