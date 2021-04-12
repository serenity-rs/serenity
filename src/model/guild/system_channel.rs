use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Deserialize, Serialize)]
    pub struct SystemChannelFlags: u32 {
        /// To suppress the welcome messages.
        const SUPPRESS_JOIN_NOTIFICATIONS = 1 << 0;
        /// To suppress the server boosting messages.
        const SUPPRESS_PREMIUM_SUBSCRIPTIONS = 1 << 1;
        /// To supress the server tips.
        const SUPPRESS_GUILD_REMINDER_NOTIFICATIONS = 1 << 2;
    }
}
