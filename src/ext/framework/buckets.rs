use std::collections::HashMap;
use std::default::Default;
use time;

#[doc(hidden)]
pub struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>,
}

#[doc(hidden)]
pub struct MemberRatelimit {
    pub count: i32,
    pub last_time: i64,
    pub set_time: i64,
}

impl Default for MemberRatelimit {
    fn default() -> Self {
        MemberRatelimit {
            count: 0,
            last_time: 0,
            set_time: 0,
        }
    }
}

#[doc(hidden)]
pub struct Bucket {
    pub ratelimit: Ratelimit,
    pub limits: HashMap<u64, MemberRatelimit>,
}

impl Bucket {
    pub fn take(&mut self, user_id: u64) -> i64 {
        let time =- time::get_time().sec;
        let member = self.limits.entry(user_id)
            .or_insert_with(MemberRatelimit::default);

        if let Some((timespan, limit)) = self.ratelimit.limit {
            if (member.count + 1) > limit {
                if time < (member.set_time + timespan) {
                    return (member.set_time + timespan) - time;
                } else {
                    member.count = 0;
                    member.set_time = time;
                }
            }
        }

        if time < member.last_time + self.ratelimit.delay {
            (member.last_time + self.ratelimit.delay) - time
        } else {
            member.count += 1;
            member.last_time = time;

            0
        }
    }
}
