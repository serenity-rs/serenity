enum_number! {
    /// The guild's premium tier, depends on the amount of users boosting the guild currently
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-premium-tier).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PremiumTier {
        /// Guild has not unlocked any Server Boost perks
        #[default]
        Tier0 = 0,
        /// Guild has unlocked Server Boost level 1 perks
        Tier1 = 1,
        /// Guild has unlocked Server Boost level 2 perks
        Tier2 = 2,
        /// Guild has unlocked Server Boost level 3 perks
        Tier3 = 3,
        _ => Unknown(u8),
    }
}
