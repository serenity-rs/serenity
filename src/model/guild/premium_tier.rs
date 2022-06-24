enum_number! {
    /// The guild's premium tier, depends on the amount of users boosting the guild currently
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum PremiumTier {
        /// No tier, considered None
        Tier0 = 0,
        Tier1 = 1,
        Tier2 = 2,
        Tier3 = 3,
        _ => Unknown(u8),
    }
}

impl Default for PremiumTier {
    fn default() -> Self {
        PremiumTier::Tier0
    }
}
