use chrono::Utc;
use client::Context;
use model::id::{ChannelId, GuildId, UserId};
use std::{
    collections::HashMap,
    default::Default
};

#[cfg(feature = "cache")]
type Check = Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool + Send + Sync + 'static;

#[cfg(not(feature = "cache"))]
type Check = Fn(&mut Context, ChannelId, UserId) -> bool + Send + Sync + 'static;

pub(crate) struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>,
}

#[derive(Default)]
pub(crate) struct MemberRatelimit {
    pub last_time: i64,
    pub set_time: i64,
    pub tickets: i32,
}

pub(crate) struct Bucket {
    pub ratelimit: Ratelimit,
    pub users: HashMap<u64, MemberRatelimit>,
    pub check: Option<Box<Check>>,
}

impl Bucket {
    pub fn take(&mut self, user_id: u64) -> i64 {
        let time = Utc::now().timestamp();
        let user = self.users
            .entry(user_id)
            .or_insert_with(MemberRatelimit::default);

        if let Some((timespan, limit)) = self.ratelimit.limit {
            if (user.tickets + 1) > limit {
                if time < (user.set_time + timespan) {
                    return (user.set_time + timespan) - time;
                } else {
                    user.tickets = 0;
                    user.set_time = time;
                }
            }
        }

        if time < user.last_time + self.ratelimit.delay {
            (user.last_time + self.ratelimit.delay) - time
        } else {
            user.tickets += 1;
            user.last_time = time;

            0
        }
    }
}
