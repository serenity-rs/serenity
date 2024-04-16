macro_rules! system_channel {
    ($(#[doc = $doc:literal] $x:ident = $y:literal: $name:literal)+) => {
        /// Describes a system channel flags.
        ///
        /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-object-system-channel-flags).
        #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
        #[derive(Copy, Clone, serde::Serialize, serde::Deserialize, Default, Debug, Eq, Hash, PartialEq)]
        pub struct SystemChannelFlags(u64);

        bitflags::bitflags! {
            impl SystemChannelFlags: u64 {
                $(
                    #[doc = $doc]
                    const $x = 1 << $y;
                )+
            }
        }

        impl SystemChannelFlags {
            /// Returns a list of names of all contained system channel settings.
            #[must_use]
            pub fn names(self) -> Vec<&'static str> {
                let mut names = Vec::with_capacity(self.0.count_ones() as _);

                $(
                    if self.contains(SystemChannelFlags::$x) {
                        names.push($name);
                    }
                )*

                names
            }
        }
    };
}

// why are your flags all negative anyways
// wish i could add a custom ser/de impl that applys `!`
system_channel! {
    /// Suppress member join notifications
    SUPPRESS_JOIN_NOTIFICATIONS = 0: "suppress welcome messages"
    /// Suppress server boost notifications
    SUPPRESS_PREMIUM_SUBSCRIPTIONS = 1: "suppress boost notifications"
    /// Suppress server setup tips
    SUPPRESS_GUILD_REMINDER_NOTIFICATIONS = 2: "suppress server setup tips"
    /// Hide member join sticker reply buttons
    SUPPRESS_JOIN_NOTIFICATION_REPLYS = 3: "suppress welcome stickers"
    /// Suppress role subscription purchase and renewal notifications
    SUPPRESS_ROLE_SUBSCRIPTION_PURCHASE_NOTIFICATIONS = 4: "suppress subscription messages" // ive guessed these, the others come from discord's own audit log (though i added suppress)
    /// Hide role subscription sticker reply buttons
    SUPPRESS_ROLE_SUBSCRIPTION_PURCHASE_NOTIFICATION_REPLIES = 5: "suppress subscription stickers"
}
