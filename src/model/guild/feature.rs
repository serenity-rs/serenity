/// A special feature, such as for VIP guilds, that a [`Guild`] has had granted
/// to them.
///
/// [`Guild`]: struct.Guild.html
#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub enum Feature {
    /// The [`Guild`] can set a custom [`splash`][`Guild::splash`] image on
    /// invite URLs.
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Guild::splash`]: struct.Guild.html#structfield.splash
    #[serde(rename = "INVITE_SPLASH")]
    InviteSplash,
    /// The [`Guild`] can set a Vanity URL, which is a custom-named permanent
    /// invite code.
    ///
    /// [`Guild`]: struct.Guild.html
    #[serde(rename = "VANITY_URL")]
    VanityUrl,
    /// The [`Guild`] has access to VIP voice channel regions.
    ///
    /// [`Guild`]: struct.Guild.html
    #[serde(rename = "VIP_REGIONS")]
    VipRegions,
}
