use chrono::UTC;
use std::collections::HashMap;
use std::default::Default;

pub(crate) struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>,
}

pub(crate) struct MemberRatelimit {
    pub last_time: i64,
    pub set_time: i64,
    pub tickets: i32,
}

impl Default for MemberRatelimit {
    fn default() -> Self {
        MemberRatelimit {
            last_time: 0,
            set_time: 0,
            tickets: 0,
        }
    }
}

pub(crate) struct Bucket {
    pub ratelimit: Ratelimit,
    pub users: HashMap<u64, MemberRatelimit>,
}

impl Bucket {
    pub fn take(&mut self, user_id: u64) -> i64 {
        let time = UTC::now().timestamp();
        let user = self.users.entry(user_id)
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
