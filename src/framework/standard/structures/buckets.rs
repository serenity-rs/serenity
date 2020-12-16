use crate::client::Context;
use crate::model::channel::Message;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::time::{Duration, Instant};

type Check =
    for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, bool>;

pub(crate) struct Ratelimit {
    pub delay: Duration,
    pub limit: Option<(Duration, u32)>,
}

#[derive(Default)]
pub(crate) struct UnitRatelimit {
    pub last_time: Option<Instant>,
    pub set_time: Option<Instant>,
    pub tickets: u32,
}

pub(crate) enum Bucket {
    /// The bucket will collect tickets for every invocation of a command.
    Global(TicketCounter),
    /// The bucket will collect tickets per user.
    User(TicketCounter),
    /// The bucket will collect tickets per guild.
    Guild(TicketCounter),
    /// The bucket will collect tickets per channel.
    Channel(TicketCounter),
    /// The bucket will collect tickets per category.
    ///
    /// This requires the cache, as messages do not contain their channel's
    /// category.
    #[cfg(feature = "cache")]
    Category(TicketCounter),
}

impl Bucket {
    #[inline]
    pub async fn take(&mut self, ctx: &Context, msg: &Message) -> Option<Duration> {
        match self {
            Self::Global(counter) => counter.take(ctx, msg, 0).await,
            Self::User(counter) => counter.take(ctx, msg, msg.author.id.0).await,
            Self::Guild(counter) => {
                if let Some(guild_id) = msg.guild_id {
                    counter.take(ctx, msg, guild_id.0).await
                } else {
                    None
                }
            }
            Self::Channel(counter) => counter.take(ctx, msg, msg.channel_id.0).await,
            // This requires the cache, as messages do not contain their channel's
            // category.
            #[cfg(feature = "cache")]
            Self::Category(counter) =>
                if let Some(category_id) = msg.category_id(ctx).await {
                    counter.take(ctx, msg, category_id.0).await
                } else {
                    None
                },
        }
    }
}

pub(crate) struct TicketCounter {
    pub ratelimit: Ratelimit,
    pub tickets_for: HashMap<u64, UnitRatelimit>,
    pub check: Option<Check>,
}

impl TicketCounter {
    pub async fn take(&mut self, ctx: &Context, msg: &Message, id: u64) -> Option<Duration> {
        if let Some(ref check) = self.check {

            if !(check)(ctx, msg).await {
                return None
            }
        }

        let now = Instant::now();
        let Self {
            tickets_for, ratelimit, ..
        } = self;
        let ticket_owner = tickets_for.entry(id).or_default();

        if let Some((timespan, limit)) = ratelimit.limit {
            if (ticket_owner.tickets + 1) > limit {
                if let Some(res) = ticket_owner
                    .set_time
                    .and_then(|x| (x + timespan).checked_duration_since(now))
                {
                    return Some(res);
                } else {
                    ticket_owner.tickets = 0;
                    ticket_owner.set_time = Some(now);
                }
            }
        }

        if let Some(res) = ticket_owner
            .last_time
            .and_then(|x| (x + ratelimit.delay).checked_duration_since(now))
        {
            return Some(res);
        } else {
            ticket_owner.tickets += 1;
            ticket_owner.last_time = Some(now);
        }

        None
    }
}

/// Decides what a bucket will use to collect tickets for.
pub enum LimitedFor {
    /// The bucket will collect tickets for every invocation of a command.
    Global,
    /// The bucket will collect tickets per user.
    User,
    /// The bucket will collect tickets per guild.
    Guild,
    /// The bucket will collect tickets per channel.
    Channel,
    /// The bucket will collect tickets per category.
    ///
    /// This requires the cache, as messages do not contain their channel's
    /// category.
    #[cfg(feature = "cache")]
    Category,
}

impl Default for LimitedFor {
    /// We use the previous behaviour of buckets as default.
    fn default() -> Self {
        Self::User
    }
}

#[derive(Default)]
pub struct BucketBuilder {
    pub(crate) delay: Duration,
    pub(crate) time_span: Duration,
    pub(crate) limit: u32,
    pub(crate) check: Option<Check>,
    pub(crate) limited_for: LimitedFor,
}

impl BucketBuilder {
    /// A bucket collecting tickets per command invocation.
    pub fn new_global() -> Self {
        Self {
            limited_for: LimitedFor::Global,
            ..Default::default()
        }
    }

    /// A bucket collecting tickets per user.
    pub fn new_user() -> Self {
        Self {
            limited_for: LimitedFor::User,
            ..Default::default()
        }
    }

    /// A bucket collecting tickets per guild.
    pub fn new_guild() -> Self {
        Self {
            limited_for: LimitedFor::Guild,
            ..Default::default()
        }
    }

    /// A bucket collecting tickets per channel.
    pub fn new_channel() -> Self {
        Self {
            limited_for: LimitedFor::Channel,
            ..Default::default()
        }
    }

    /// A bucket collecting tickets per channel category.
    ///
    /// This requires the cache, as messages do not contain their channel's
    /// category.
    #[cfg(feature = "cache")]
    pub fn new_category() -> Self {
        Self {
            limited_for: LimitedFor::Category,
            ..Default::default()
        }
    }

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
    /// [`time_span`]: Self::time_span
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

    /// Limit the bucket for a specific type of `target`.
    #[inline]
    pub fn limit_for(&mut self, target: LimitedFor) -> &mut Self {
        self.limited_for = target;

        self
    }

    /// Constructs the bucket.
    #[inline]
    pub(crate) fn construct(self) -> Bucket {
        let counter = TicketCounter {
            ratelimit: Ratelimit {
                delay: self.delay,
                limit: Some((self.time_span, self.limit)),
            },
            tickets_for: HashMap::new(),
            check: self.check,
        };

        match self.limited_for {
            LimitedFor::User => Bucket::User(counter),
            LimitedFor::Guild => Bucket::Guild(counter),
            LimitedFor::Channel => Bucket::Channel(counter),
            // This requires the cache, as messages do not contain their channel's
            // category.
            #[cfg(feature = "cache")]
            LimitedFor::Category => Bucket::Category(counter),
            LimitedFor::Global => Bucket::Global(counter),
        }
    }
}
