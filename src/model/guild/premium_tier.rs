/// The guild's premium tier, depends on the amount of users boosting the guild currently
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PremiumTier {
    /// No tier, considered None
    Tier0,
    Tier1,
    Tier2,
    Tier3,
    #[doc(hidden)]
    __Nonexhaustive,
}

enum_number!(
    PremiumTier {
        Tier0,
        Tier1,
        Tier2,
        Tier3,
        __Nonexhaustive,
    }
);

impl PremiumTier {
    pub fn num(self) -> u64 {
        match self {
            PremiumTier::Tier0 => 0,
            PremiumTier::Tier1 => 1,
            PremiumTier::Tier2 => 2,
            PremiumTier::Tier3 => 3,
            PremiumTier::__Nonexhaustive => unreachable!(),
        }
    }
}

impl Default for PremiumTier {
    fn default() -> Self {
        PremiumTier::Tier0
    }
}
