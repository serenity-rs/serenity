enum_number! {
    /// The guild's premium tier, depends on the amount of users boosting the guild currently
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-premium-tier).
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PremiumTier {
        /// No tier, considered None
        #[default]
        Tier0 = 0,
        Tier1 = 1,
        Tier2 = 2,
        Tier3 = 3,
        _ => Unknown(u8),
    }
}
