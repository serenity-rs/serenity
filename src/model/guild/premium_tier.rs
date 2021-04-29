/// The guild's premium tier, depends on the amount of users boosting the guild currently
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum PremiumTier {
    /// No tier, considered None
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    Unknown = !0,
}

enum_number!(PremiumTier {
    Tier0,
    Tier1,
    Tier2,
    Tier3
});

impl Default for PremiumTier {
    fn default() -> Self {
        PremiumTier::Tier0
    }
}
