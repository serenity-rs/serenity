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

#[derive(Default)]
pub(crate) struct UnitRatelimitTimes {
    pub last_time: Option<Instant>,
    pub set_time: Option<Instant>,
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
    pub async fn take(&mut self, ctx: &Context, msg: &Message) -> Option<BucketAction> {
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

    #[inline]
    pub async fn give(&mut self, ctx: &Context, msg: &Message) {
        match self {
            Self::Global(counter) => counter.give(ctx, msg, 0).await,
            Self::User(counter) => counter.give(ctx, msg, msg.author.id.0).await,
            Self::Guild(counter) => {
                if let Some(guild_id) = msg.guild_id {
                    counter.give(ctx, msg, guild_id.0).await
                }
            }
            Self::Channel(counter) => counter.give(ctx, msg, msg.channel_id.0).await,
            // This requires the cache, as messages do not contain their channel's
            // category.
            #[cfg(feature = "cache")]
            Self::Category(counter) =>
                if let Some(category_id) = msg.category_id(ctx).await {
                    counter.give(ctx, msg, category_id.0).await
                }
            }
        }
}

/// Keeps track of who owns how many tickets and when they accessed the last
/// time.
pub(crate) struct TicketCounter {
    pub ratelimit: Ratelimit,
    pub tickets_for: HashMap<u64, UnitRatelimit>,
    pub check: Option<Check>,
    pub await_ratelimits: bool,
}

/// A bucket may return results based on how it set up.
///
/// By default, it will return `CancelWith` when a limit is hit.
/// This is intended to cancel the command invocation and propagate the
/// duration to the user.
///
/// If the bucket is set to await durations, it will suggest to wait
/// for the bucket by returning `DelayFor` and then delay for the duration,
/// and then try taking a ticket again.
pub enum BucketAction {
    CancelWith(Duration),
    DelayFor(Duration),
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
    pub async fn take(&mut self, ctx: &Context, msg: &Message, id: u64) -> Option<BucketAction> {
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

        // Check if too many tickets have been taken already.
        // If all tickets are exhausted, return the needed delay
        // for this invocation.
        if let Some((timespan, limit)) = ratelimit.limit {

            if (ticket_owner.tickets + 1) > limit {

                if let Some(res) = ticket_owner
                    .set_time
                    .and_then(|x| (x + timespan).checked_duration_since(now))
                {
                    return Some(if self.await_ratelimits {
                        BucketAction::DelayFor(res)
                    } else {
                        BucketAction::CancelWith(res)
                    })
                } else {
                    ticket_owner.tickets = 0;
                    ticket_owner.set_time = Some(now);
                }
            }
        }

        // Check if `ratelimit.delay`-time passed between the last and
        // the current invocation
        // If the time did not pass, return the needed delay for this
        // invocation.
        if let Some(ratelimit) = ticket_owner
            .last_time
            .and_then(|x| (x + ratelimit.delay).checked_duration_since(now))
        {
            return Some(if self.await_ratelimits {
                BucketAction::DelayFor(ratelimit)
            } else {
                BucketAction::CancelWith(ratelimit)
            })
        } else {
            ticket_owner.tickets += 1;
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
                return
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
            // substracting the delay once, we allow for one more ticket to be
            // taken.
            // If we would reset the value to `None`, we risk resetting the
            // bucket.
            ticket_owner.last_time = ticket_owner
                .last_time
                .and_then(|i| i.checked_sub(delay));
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
    pub(crate) await_ratelimits: bool,
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

    /// If this is set to `true`, the invocation of the command will be delayed
    /// and won't return a duration to wait to dispatch erros, but actually
    /// await until the duration has been elapsed.
    ///
    /// By default, ratelimits will become dispatch errors.
    #[inline]
    pub fn await_ratelimits(&mut self) -> &mut Self {
        self.await_ratelimits = true;

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
