use crate::client::Context;
use crate::model::id::{ChannelId, GuildId, UserId};
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::time::{Duration, Instant};

type Check =
    for<'fut> fn(&'fut Context, Option<GuildId>, ChannelId, UserId) -> BoxFuture<'fut, bool>;

pub(crate) struct Ratelimit {
    pub delay: Duration,
    pub limit: Option<(Duration, u32)>,
}

#[derive(Default)]
pub(crate) struct MemberRatelimit {
    pub last_time: Option<Instant>,
    pub set_time: Option<Instant>,
    pub tickets: u32,
}

pub(crate) struct Bucket {
    pub ratelimit: Ratelimit,
    pub users: HashMap<u64, MemberRatelimit>,
    pub check: Option<Check>,
}

impl Bucket {
    pub fn take(&mut self, user_id: u64) -> Option<Duration> {
        let now = Instant::now();
        let Self {
            users, ratelimit, ..
        } = self;
        let user = users.entry(user_id).or_default();

        if let Some((timespan, limit)) = ratelimit.limit {
            if (user.tickets + 1) > limit {
                if let Some(res) = user
                    .set_time
                    .and_then(|x| (x + timespan).checked_duration_since(now))
                {
                    return Some(res);
                } else {
                    user.tickets = 0;
                    user.set_time = Some(now);
                }
            }
        }

        if let Some(res) = user
            .last_time
            .and_then(|x| (x + ratelimit.delay).checked_duration_since(now))
        {
            return Some(res);
        } else {
            user.tickets += 1;
            user.last_time = Some(now);
        }

        None
    }
}

#[derive(Default)]
pub struct BucketBuilder {
    pub(crate) delay: Duration,
    pub(crate) time_span: Duration,
    pub(crate) limit: u32,
    pub(crate) check: Option<Check>,
}

impl BucketBuilder {
    /// The "break" time between invocations of a command.
    ///
    /// Expressed in seconds.
    #[inline]
    pub fn delay(&mut self, secs: u64) -> &mut Self {
        self.delay = Duration::from_secs(secs);

        self
    }

    /// How long the bucket will apply for.
    ///
    /// Expressed in seconds.
    #[inline]
    pub fn time_span(&mut self, secs: u64) -> &mut Self {
        self.time_span = Duration::from_secs(secs);

        self
    }

    /// Number of invocations allowed per [`time_span`].
    ///
    /// Expressed in seconds.
    ///
    /// [`time_span`]: #method.time_span
    #[inline]
    pub fn limit(&mut self, n: u32) -> &mut Self {
        self.limit = n;

        self
    }

    /// Middleware confirming (or denying) that the bucket is eligible to apply.
    /// For instance, to limit the bucket to just one user.
    #[inline]
    pub fn check(&mut self, f: Check) -> &mut Self {
        self.check = Some(f);

        self
    }
}
