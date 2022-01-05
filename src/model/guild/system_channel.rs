use bitflags::__impl_bitflags;

/// Describes a system channel flags.
#[derive(Copy, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Default)]
pub struct SystemChannelFlags {
    pub bits: u64,
}

__impl_bitflags! {
    SystemChannelFlags: u64 {
        /// Suppress member join notifications.
        SUPPRESS_JOIN_NOTIFICATIONS = 1 << 0;
        /// Suppress server boost notifications.
        SUPPRESS_PREMIUM_SUBSCRIPTIONS = 1 << 1;
        /// Suppress server setup tips.
        SUPPRESS_GUILD_REMINDER_NOTIFICATIONS = 1 << 2;
        /// Hide member join sticker reply buttons.
        SUPPRESS_JOIN_NOTIFICATION_REPLIES = 1 << 3;
    }
}

impl_bitflags_serde!(SystemChannelFlags: u64);
