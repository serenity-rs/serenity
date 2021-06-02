use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;

use crate::client::Context;
use crate::model::channel::Message;

type Check = for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, bool>;

type DelayHook = for<'fut> fn(&'fut Context, &'fut Message) -> BoxFuture<'fut, ()>;

pub(crate) struct Ratelimit {
    pub delay: Duration,
    pub limit: Option<(Duration, u32)>,
}
pub(crate) struct UnitRatelimit {
    pub last_time: Option<Instant>,
    pub set_time: Instant,
    pub tickets: u32,
    pub awaiting: u32,
    pub is_first_try: bool,
}

impl UnitRatelimit {
    fn new(creation_time: Instant) -> Self {
        Self {
            last_time: None,
            set_time: creation_time,
            tickets: 0,
            awaiting: 0,
            is_first_try: true,
        }
    }
}

/// A bucket offers fine-grained control over the execution of commands.
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
    pub async fn take(&mut self, ctx: &Context, msg: &Message) -> Option<RateLimitInfo> {
        match self {
            Self::Global(counter) => counter.take(ctx, msg, 0).await,
            Self::User(counter) => counter.take(ctx, msg, msg.author.id.0).await,
            Self::Guild(counter) => {
                if let Some(guild_id) = msg.guild_id {
                    counter.take(ctx, msg, guild_id.0).await
                } else {
                    None
                }
            },
            Self::Channel(counter) => counter.take(ctx, msg, msg.channel_id.0).await,
            // This requires the cache, as messages do not contain their channel's
            // category.
            #[cfg(feature = "cache")]
            Self::Category(counter) => {
                if let Some(category_id) = msg.category_id(ctx).await {
                    counter.take(ctx, msg, category_id.0).await
                } else {
                    None
                }
            },
        }
    }

    #[inline]
    pub async fn give(&mut self, ctx: &Context, msg: &Message) {
        match self {
            Self::Global(counter) => counter.give(ctx, msg, 0).await,
            Self::User(counter) => counter.give(ctx, msg, msg.author.id.0).await,
            Self::Guild(counter) => {
                if let Some(guild_id) = msg.guild_id {
                    counter.give(ctx, msg, guild_id.0).await
                }
            },
            Self::Channel(counter) => counter.give(ctx, msg, msg.channel_id.0).await,
            // This requires the cache, as messages do not contain their channel's
            // category.
            #[cfg(feature = "cache")]
            Self::Category(counter) => {
                if let Some(category_id) = msg.category_id(ctx).await {
                    counter.give(ctx, msg, category_id.0).await
                }
            },
        }
    }
}

/// Keeps track of who owns how many tickets and when they accessed the last
/// time.
pub(crate) struct TicketCounter {
    pub ratelimit: Ratelimit,
    pub tickets_for: HashMap<u64, UnitRatelimit>,
    pub check: Option<Check>,
    pub delay_action: Option<DelayHook>,
    pub await_ratelimits: u32,
}

/// Contains information about a rate limit.
#[derive(Debug)]
pub struct RateLimitInfo {
    /// Time to elapse in order to invoke a command again.
    pub rate_limit: Duration,
    /// Amount of active delays by this target.
    pub active_delays: u32,
    /// Maximum delays that this target can invoke.
    pub max_delays: u32,
    /// Whether this is the first time the rate limit info has been
    /// returned for this target without the rate limit to elapse.
    pub is_first_try: bool,
    /// How the command invocation has been treated by the framework.
    pub action: RateLimitAction,
}

/// Action taken for the command invocation.
#[derive(Debug)]
pub enum RateLimitAction {
    /// Invocation has been delayed.
    Delayed,
    /// Tried to delay invocation but maximum of delays reached.
    FailedDelay,
    /// Cancelled the invocation due to time or ticket reasons.
    Cancelled,
}

impl RateLimitInfo {
    /// Gets the duration of the rate limit in seconds.
    #[inline]
    pub fn as_secs(&self) -> u64 {
        self.rate_limit.as_secs()
    }

    /// Gets the duration of the rate limit in milliseconds.
    #[inline]
    pub fn as_millis(&self) -> u128 {
        self.rate_limit.as_millis()
    }

    /// Gets the duration of the rate limit in microseconds.
    #[inline]
    pub fn as_micros(&self) -> u128 {
        self.rate_limit.as_micros()
    }
}

impl TicketCounter {
    /// Tries to check whether the invocation is permitted by the ticket counter
    /// and if a ticket can be taken; it does not return a
    /// a ticket but a duration until a ticket can be taken.
    ///
    /// The duration will be wrapped in an action for the caller to perform
    /// if wanted. This may inform them to directly cancel trying to take a ticket
    /// or delay the take until later.
    ///
    /// However there is no contract: It does not matter what
    /// the caller ends up doing, receiving some action eventually means
    /// no ticket can be taken and the duration must elapse.
    pub async fn take(&mut self, ctx: &Context, msg: &Message, id: u64) -> Option<RateLimitInfo> {
        if let Some(ref check) = self.check {
            if !(check)(ctx, msg).await {
                return None;
            }
        }

        let now = Instant::now();
        let Self {
            tickets_for,
            ratelimit,
            ..
        } = self;

        let ticket_owner = tickets_for.entry(id).or_insert_with(|| UnitRatelimit::new(now));

        // Check if too many tickets have been taken already.
        // If all tickets are exhausted, return the needed delay
        // for this invocation.
        if let Some((timespan, limit)) = ratelimit.limit {
            if (ticket_owner.tickets + 1) > limit {
                if let Some(ratelimit) =
                    (ticket_owner.set_time + timespan).checked_duration_since(now)
                {
                    let was_first_try = ticket_owner.is_first_try;

                    // Are delay limits left?
                    let action = if self.await_ratelimits > ticket_owner.awaiting {
                        ticket_owner.awaiting += 1;

                        if let Some(delay_action) = self.delay_action {
                            let ctx = ctx.clone();
                            let msg = msg.clone();

                            tokio::spawn(async move {
                                delay_action(&ctx, &msg).await;
                            });
                        }

                        RateLimitAction::Delayed
                    // Is this bucket utilising delay limits?
                    } else if self.await_ratelimits > 0 {
                        ticket_owner.is_first_try = false;

                        RateLimitAction::FailedDelay
                    } else {
                        ticket_owner.is_first_try = false;

                        RateLimitAction::Cancelled
                    };

                    return Some(RateLimitInfo {
                        rate_limit: ratelimit,
                        active_delays: ticket_owner.awaiting,
                        max_delays: self.await_ratelimits,
                        action,
                        is_first_try: was_first_try,
                    });
                } else {
                    ticket_owner.tickets = 0;
                    ticket_owner.set_time = now;
                }
            }
        }

        // Check if `ratelimit.delay`-time passed between the last and
        // the current invocation
        // If the time did not pass, return the needed delay for this
        // invocation.
        if let Some(ratelimit) =
            ticket_owner.last_time.and_then(|x| (x + ratelimit.delay).checked_duration_since(now))
        {
            let was_first_try = ticket_owner.is_first_try;

            // Are delay limits left?
            let action = if self.await_ratelimits > ticket_owner.awaiting {
                ticket_owner.awaiting += 1;

                if let Some(delay_action) = self.delay_action {
                    let ctx = ctx.clone();
                    let msg = msg.clone();

                    tokio::spawn(async move {
                        delay_action(&ctx, &msg).await;
                    });
                }

                RateLimitAction::Delayed
            // Is this bucket utilising delay limits?
            } else if self.await_ratelimits > 0 {
                ticket_owner.is_first_try = false;

                RateLimitAction::FailedDelay
            } else {
                RateLimitAction::Cancelled
            };

            return Some(RateLimitInfo {
                rate_limit: ratelimit,
                active_delays: ticket_owner.awaiting,
                max_delays: self.await_ratelimits,
                action,
                is_first_try: was_first_try,
            });
        } else {
            ticket_owner.awaiting = ticket_owner.awaiting.saturating_sub(1);
            ticket_owner.tickets += 1;
            ticket_owner.is_first_try = true;
            ticket_owner.last_time = Some(now);
        }

        None
    }

    /// Reverts the last ticket step performed by returning a ticket for the
    /// matching ticket holder.
    /// Only call this if the mutable owner already took a ticket in this
    /// atomic execution of calling `take` and `give`.
    pub async fn give(&mut self, ctx: &Context, msg: &Message, id: u64) {
        if let Some(ref check) = self.check {
            if !(check)(ctx, msg).await {
                return;
            }
        }

        if let Some(ticket_owner) = self.tickets_for.get_mut(&id) {
            // Remove a ticket if one is available.
            if ticket_owner.tickets > 0 {
                ticket_owner.tickets -= 1;
            }

            let delay = self.ratelimit.delay;
            // Substract one step of time that would have to pass.
            // This tries to bypass a problem of keeping track of when tickets
            // were taken.
            // When a ticket is taken, the bucket sets `last_time`, by
            // substracting the delay, once a ticket is allowed to be
            // taken.
            // If the value is set to `None` this could possibly reset the
            // bucket.
            ticket_owner.last_time = ticket_owner.last_time.and_then(|i| i.checked_sub(delay));
        }
    }
}

/// An error struct that can be returned from a command to set the
/// bucket one step back.
#[derive(Debug)]
pub struct RevertBucket;

impl std::fmt::Display for RevertBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RevertBucket")
    }
}

impl std::error::Error for RevertBucket {}

/// Decides what a bucket will use to collect tickets for.
#[derive(Debug)]
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

pub struct BucketBuilder {
    pub(crate) delay: Duration,
    pub(crate) time_span: Duration,
    pub(crate) limit: u32,
    pub(crate) check: Option<Check>,
    pub(crate) delay_action: Option<DelayHook>,
    pub(crate) limited_for: LimitedFor,
    pub(crate) await_ratelimits: u32,
}

impl Default for BucketBuilder {
    fn default() -> Self {
        Self {
            delay: Duration::default(),
            time_span: Duration::default(),
            limit: 1,
            check: None,
            delay_action: None,
            limited_for: LimitedFor::default(),
            await_ratelimits: 0,
        }
    }
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

    /// Number of invocations allowed per [`Self::time_span`].
    #[inline]
    pub fn limit(&mut self, n: u32) -> &mut Self {
        self.limit = n;

        self
    }

    /// Middleware confirming (or denying) that the bucket is eligible to apply.
    /// For instance, to limit the bucket to just one user.
    #[inline]
    pub fn check(&mut self, check: Check) -> &mut Self {
        self.check = Some(check);

        self
    }

    /// This function will be called once a user's invocation has been delayed.
    #[inline]
    pub fn delay_action(&mut self, action: DelayHook) -> &mut Self {
        self.delay_action = Some(action);

        self
    }

    /// Limit the bucket for a specific type of `target`.
    #[inline]
    pub fn limit_for(&mut self, target: LimitedFor) -> &mut Self {
        self.limited_for = target;

        self
    }

    /// If this is set to an `amount` greater than `0`, the invocation of the
    /// command will be delayed `amount` times instead of stopping command
    /// dispatch.
    ///
    /// By default this value is `0` and rate limits will cancel instead.
    #[inline]
    pub fn await_ratelimits(&mut self, amount: u32) -> &mut Self {
        self.await_ratelimits = amount;

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
            delay_action: self.delay_action,
            await_ratelimits: self.await_ratelimits,
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
