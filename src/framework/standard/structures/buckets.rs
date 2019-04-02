use chrono::Utc;
use crate::client::Context;
use crate::model::id::{ChannelId, GuildId, UserId};
use std::collections::HashMap;

type Check = dyn Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool + Send + Sync + 'static;

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

#[derive(Default)]
pub struct BucketBuilder {
    pub(crate) delay: i64,
    pub(crate) time_span: i64,
    pub(crate) limit: i32,
    pub(crate) check: Option<Box<Check>>,
}

impl BucketBuilder {
    /// The "break" time between invocations of a command.
    ///
    /// Expressed in seconds.
    #[inline]
    pub fn delay(&mut self, n: i64) -> &mut Self {
        self.delay = n;

        self
    }

    /// How long the bucket will apply for.
    ///
    /// Expressed in seconds.
    #[inline]
    pub fn time_span(&mut self, n: i64) -> &mut Self {
        self.time_span = n;

        self
    }

    /// Number of invocations allowed per [`time_span`].
    ///
    /// Expressed in seconds.
    ///
    /// [`time_span`]: #method.time_span
    #[inline]
    pub fn limit(&mut self, n: i32) -> &mut Self {
        self.limit = n;

        self
    }

    /// Middleware confirming (or denying) that the bucket is eligible to apply.
    /// For instance, to limit the bucket to just one user.
    #[inline]
    pub fn check<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool + Send + Sync + 'static
    {
        self.check = Some(Box::new(f));

        self
    }
}
