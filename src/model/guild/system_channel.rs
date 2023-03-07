bitflags! {
    /// Describes a system channel flags.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-system-channel-flags).
    #[derive(Default)]
    pub struct SystemChannelFlags: u64 {
        /// Suppress member join notifications.
        const SUPPRESS_JOIN_NOTIFICATIONS = 1 << 0;
        /// Suppress server boost notifications.
        const SUPPRESS_PREMIUM_SUBSCRIPTIONS = 1 << 1;
        /// Suppress server setup tips.
        const SUPPRESS_GUILD_REMINDER_NOTIFICATIONS = 1 << 2;
        /// Hide member join sticker reply buttons.
        const SUPPRESS_JOIN_NOTIFICATION_REPLIES = 1 << 3;
    }
}
