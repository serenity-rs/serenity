use model::prelude::*;
use super::super::utils::{deserialize_emojis, deserialize_roles};


/// Partial information about a [`Guild`]. This does not include information
/// like member data.
///
/// [`Guild`]: struct.Guild.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialGuild {
    pub id: GuildId,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u64,
    pub default_message_notifications: DefaultMessageNotificationLevel,
    pub embed_channel_id: Option<ChannelId>,
    pub embed_enabled: bool,
    #[serde(deserialize_with = "deserialize_emojis")] pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.a
    ///
    /// Refer to [`Guild::features`] for more information.
    ///
    /// [`Guild::features`]: struct.Guild.html#structfield.features
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub mfa_level: MfaLevel,
    pub name: String,
    pub owner_id: UserId,
    pub region: String,
    #[serde(deserialize_with = "deserialize_roles",
            serialize_with = "serialize_gen_map")]
    pub roles: HashMap<RoleId, Role>,
    pub splash: Option<String>,
    pub verification_level: VerificationLevel,
}

impl PartialGuild {
    /// Returns a formatted URL of the guild's icon, if the guild has an icon.
    pub fn icon_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/icons/{}/{}.webp"), self.id, icon))
    }

    /// Returns the Id of the shard associated with the guild.
    ///
    /// When the cache is enabled this will automatically retrieve the total
    /// number of shards.
    ///
    /// When the cache is not enabled, the total number of shards being used
    /// will need to be passed.
    ///
    /// # Examples
    ///
    /// Retrieve the Id of the shard for a guild with Id `81384788765712384`,
    /// using 17 shards:
    ///
    /// ```rust,ignore
    /// use serenity::utils;
    ///
    /// // assumes a `guild` has already been bound
    ///
    /// assert_eq!(guild.shard_id(17), 7);
    /// ```
    #[cfg(feature = "utils")]
    #[inline]
    pub fn shard_id(&self, shard_count: u64) -> u64 { self.id.shard_id(shard_count) }

    /// Returns the formatted URL of the guild's splash image, if one exists.
    pub fn splash_url(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|icon| format!(cdn!("/splashes/{}/{}.webp"), self.id, icon))
    }

    /// Obtain a reference to a role by its name.
    ///
    /// **Note**: If two or more roles have the same name, obtained reference will be one of
    /// them.
    ///
    /// # Examples
    ///
    /// Obtain a reference to a [`Role`] by its name.
    ///
    /// ```rust,no_run
    /// use serenity::model::prelude::*;
    /// use serenity::prelude::*;
    ///
    /// struct Handler;
    ///
    /// use serenity::CACHE;
    ///
    /// impl EventHandler for Handler {
    ///     fn message(&self, _: Context, msg: Message) {
    ///         let guild = msg.guild_id().unwrap().get().unwrap();
    ///         let possible_role = guild.role_by_name("role_name");
    ///
    ///         if let Some(role) = possible_role {
    ///             println!("Obtained role's reference: {:?}", role);
    ///         }
    ///     }
    /// }
    ///
    /// let mut client = Client::new("token", Handler).unwrap();
    ///
    /// client.start().unwrap();
    /// ```
    pub fn role_by_name(&self, role_name: &str) -> Option<&Role> {
        self.roles.values().find(|role| role_name == role.name)
    }
}
