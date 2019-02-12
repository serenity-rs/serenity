//! Routes are used for ratelimiting. These are to differentiate between the
//! different _types_ of routes - such as getting the current user's channels -
//! for the most part, with the exception being major parameters.
//!
//! [Taken from] the Discord docs, major parameters are:
//!
//! > Additionally, rate limits take into account major parameters in the URL.
//! > For example, `/channels/:channel_id` and
//! > `/channels/:channel_id/messages/:message_id` both take `channel_id` into
//! > account when generating rate limits since it's the major parameter. The
//! only current major parameters are `channel_id` and `guild_id`.
//!
//! This results in the two URIs of `GET /channels/4/messages/7` and
//! `GET /channels/5/messages/8` being rate limited _separately_. However, the
//! two URIs of `GET /channels/10/messages/11` and
//! `GET /channels/10/messages/12` will count towards the "same ratelimit", as
//! the major parameter - `10` is equivalent in both URIs' format.
//!
//! # Examples
//!
//! First: taking the first two URIs - `GET /channels/4/messages/7` and
//! `GET /channels/5/messages/8` - and assuming both buckets have a `limit` of
//! `10`, requesting the first URI will result in the response containing a
//! `remaining` of `9`. Immediately after - prior to buckets resetting -
//! performing a request to the _second_ URI will also contain a `remaining` of
//! `9` in the response, as the major parameter - `channel_id` - is different
//! in the two requests (`4` and `5`).
//!
//! Second: take for example the last two URIs. Assuming the bucket's `limit` is
//! `10`, requesting the first URI will return a `remaining` of `9` in the
//! response. Immediately after - prior to buckets resetting - performing a
//! request to the _second_ URI will return a `remaining` of `8` in the
//! response, as the major parameter - `channel_id` - is equivalent for the two
//! requests (`10`).
//!
//! Major parameters are why some variants (i.e. all of the channel/guild
//! variants) have an associated u64 as data. This is the Id of the parameter,
//! differentiating between different ratelimits.
//!
//! [Taken from]: https://discordapp.com/developers/docs/topics/rate-limits#rate-limits
#![cfg_attr(feature = "cargo-clippy", allow(zero_ptr))]

use chrono::{DateTime, Utc};
use futures::sync::oneshot::{self, Receiver, Sender};
use futures::{Future, future};
use hyper::{
    header::HeaderMap,
    Response,
    StatusCode,
};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{i64, str, u8};
use super::{Error, Path, Result};
use tokio::{
    runtime::current_thread,
    timer::Delay,
};

#[derive(Debug)]
pub enum RateLimitError {
    /// When the decoding of a header could not be properly decoded as UTF-8.
    DecodingUtf8,
    /// When the decoding of a header could not be properly decoded as an `i64`.
    DecodingInteger,
}

impl Display for RateLimitError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for RateLimitError {
    fn description(&self) -> &str {
        use self::RateLimitError::*;

        match *self {
            DecodingInteger => "Error decoding a header into an i64",
            DecodingUtf8 => "Error decoding a header from UTF-8",
        }
    }
}

#[derive(Clone, Debug)]
pub enum RateLimit {
    Global(i64),
    NotReached(RateLimitHeaders),
    Reached(i64),
}

impl RateLimit {
    fn from_headers(headers: &HeaderMap) -> Result<Self> {
        if headers.get("x-ratelimit-global").is_some() {
            if let Some(retry_after) = parse_header(headers, "retry-after")? {
                debug!("Global ratelimited for {}ms", retry_after);

                return Ok(RateLimit::Global(retry_after));
            }

            warn!("Globally ratelimited with no retry-after? Skipping...");
        }

        if let Some(retry_after) = parse_header(headers, "retry-after")? {
            return Ok(RateLimit::Reached(retry_after));
        }

        let limit = parse_header(headers, "x-ratelimit-limit")?.map(|x| x as u8);
        let remaining = parse_header(headers, "x-ratelimit-remaining")?.map(|x| x as u8);
        let reset = parse_header(headers, "x-ratelimit-remaining")?;

        Ok(RateLimit::NotReached(RateLimitHeaders {
            limit,
            remaining,
            reset,
        }))
    }
}

#[derive(Clone, Debug)]
pub struct RateLimitHeaders {
    pub limit: Option<u8>,
    pub remaining: Option<u8>,
    pub reset: Option<i64>,
}

/// A set of data containing information about the ratelimits for a particular
/// [`Route`], which is stored in the [`ROUTES`] mutex.
///
/// See the [Discord docs] on ratelimits for more information.
///
/// **Note**: You should _not_ mutate any of the fields, as this can cause 429s.
///
/// [`ROUTES`]: struct.ROUTES.html
/// [`Route`]: enum.Route.html
/// [Discord docs]: https://discordapp.com/developers/docs/topics/rate-limits
// todo: impl Debug
#[derive(Debug)]
pub struct Bucket {
    /// The total number of requests that can be made in a period of time.
    pub limit: i64,
    /// A queue of requests that were held back due to a pre-emptive ratelimit.
    pub queue: VecDeque<Sender<()>>,
    /// The number of requests remaining in the period of time.
    pub remaining: i64,
    /// When the interval resets and the [`limit`] resets to the value of
    /// [`remaining`].
    ///
    /// [`limit`]: #structfield.limit
    /// [`remaining`]: #structfield.remaining
    pub reset: i64,
    /// Whether the bucket has a timeout in the background to release (part of)
    /// the queue.
    pub timeout: bool,
}

impl Default for Bucket {
    fn default() -> Self {
        Self {
            limit: i64::MAX,
            queue: VecDeque::new(),
            remaining: i64::MAX,
            reset: i64::MAX,
            timeout: false,
        }
    }
}

impl Bucket {
    fn take(&mut self) -> Option<Receiver<()>> {
        if self.reset == 0 {
            let (tx, rx) = oneshot::channel();

            self.queue.push_back(tx);

            Some(rx)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Global {
    pub blocked: bool,
    pub queue: Arc<Mutex<VecDeque<Receiver<()>>>>,
}

#[derive(Clone, Debug)]
pub struct RateLimiter {
    /// The global mutex is a mutex unlocked and then immediately re-locked
    /// prior to every request, to abide by Discord's global ratelimit.
    ///
    /// The global ratelimit is the total number of requests that may be made
    /// across the entirity of the API within an amount of time. If this is
    /// reached, then the global mutex is unlocked for the amount of time
    /// present in the "Retry-After" header.
    ///
    /// While locked, all requests are blocked until each request can acquire
    /// the lock.
    ///
    /// The only reason that you would need to use the global mutex is to
    /// block requests yourself. This has the side-effect of potentially
    /// blocking many of your event handlers or framework commands.
    pub global: Arc<Mutex<Global>>,
    /// The calculated offset of the time difference between Discord and the client
    /// in seconds.
    ///
    /// This does not have millisecond precision as calculating that isn't
    /// realistic.
    ///
    /// This is used in ratelimiting to help determine how long to wait for
    /// pre-emptive ratelimits. For example, if the client is 2 seconds ahead, then
    /// the client would think the ratelimit is over 2 seconds before it actually is
    /// and would then send off queued requests. Using an offset, we can know that
    /// there's actually still 2 seconds left (+/- some milliseconds).
    ///
    /// This isn't a definitive solution to fix all problems, but it can help with
    /// some precision gains.
    offset: Option<i64>,
    /// The routes mutex is a HashMap of each [`Route`] and their respective
    /// ratelimit information.
    ///
    /// See the documentation for [`RateLimit`] for more infomation on how the
    /// library handles ratelimiting.
    ///
    /// # Examples
    ///
    /// View the `reset` time of the route for `ChannelsId(7)`:
    ///
    /// ```rust,no_run
    /// use serenity::http::ratelimiting::{ROUTES, Route};
    ///
    /// if let Some(route) = ROUTES.lock().get(&Route::ChannelsId(7)) {
    ///     println!("Reset time at: {}", route.lock().reset);
    /// }
    /// ```
    ///
    /// [`RateLimit`]: struct.RateLimit.html
    /// [`Route`]: enum.Route.html
    pub routes: Arc<Mutex<HashMap<Path, Bucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            global: Arc::new(Mutex::new(Global::default())),
            offset: None,
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn take(&mut self, route: &Path)
        -> impl Future<Item = (), Error = Error> + Send {
        // TODO: handle global
        let mut routes = self.routes.lock();
        let bucket = routes.entry(*route).or_insert_with(Default::default);
        let take = bucket.take();

        match take {
            Some(rx) => {
                if !bucket.timeout {
                    let reset_ms = (bucket.reset * 1000) as u64;
                    let now = Utc::now();
                    let now_millis = now.timestamp_subsec_millis() as i64;
                    let now_ms = (now.timestamp() * 1000) + now_millis;
                    let wait_ms = reset_ms.saturating_sub(now_ms as u64);
                    let duration = Duration::from_millis(wait_ms as u64);

                    let done = Delay::new(Instant::now() + duration)
                        .map(|_| {
                            ()
                        }).map_err(|why| {
                            warn!("Err with pre-ratelimit sleep: {:?}", why);

                            ()
                        });

                    current_thread::spawn(done);
                }

                future::Either::A(rx.from_err())
            },
            None => future::Either::B(future::ok(())),
        }
    }

    pub fn handle<'a>(&'a mut self, route: &'a Path, response: &'a Response<Vec<u8>>)
        -> Result<Option<impl Future<Item = (), Error = ()> + Send>> {
        let mut routes = self.routes.lock();
        let bucket = routes.entry(*route).or_insert_with(Default::default);

        if response.status() != StatusCode::TOO_MANY_REQUESTS {
            return Ok(None);
        }

        match RateLimit::from_headers(&response.headers())? {
            RateLimit::Global(millis) => {
                debug!("Globally ratelimited for {:?}ms", millis);

                self.global.lock().blocked = true;
                let global = Arc::clone(&self.global);
                let duration = Duration::from_millis(millis as u64);
                let delay = Delay::new(Instant::now() + duration);

                let done = delay
                    .map(move |_| {
                        let mut global = global.lock();
                        global.blocked = false;
                    })
                    .map_err(|why| {
                        warn!("Err with global ratelimit timer: {:?}", why);

                        ()
                    });

                Ok(Some(future::Either::A(done)))
            },
            RateLimit::NotReached(headers) => {
                let RateLimitHeaders { limit, remaining, reset } = headers;

                if let Some(reset) = reset {
                    if reset != bucket.reset {
                        bucket.reset = reset;

                        if let Some(limit) = limit {
                            bucket.limit = limit as i64;
                        }

                        if let Some(remaining) = remaining {
                            bucket.remaining = remaining as i64;
                        }
                    }
                }

                Ok(None)
            },
            RateLimit::Reached(millis) => {
                debug!("Ratelimited on route {:?} for {:?}ms", route, millis);

                let duration = Duration::from_millis(millis as u64);

                let done = Delay::new(Instant::now() + duration)
                    .map_err(|why| {
                        warn!("Err with ratelimited timer: {:?}", why);

                        ()
                    });

                Ok(Some(future::Either::B(done)))
            },
        }
    }
}

/*
pub(crate) fn perform<'a, F>(route: Route, f: F) -> Result<Response>
    where F: Fn() -> RequestBuilder<'a> {
    loop {
        // This will block if another thread already has the global
        // unlocked already (due to receiving an x-ratelimit-global).
        let _ = GLOBAL.lock();

        // Perform pre-checking here:
        //
        // - get the route's relevant rate
        // - sleep if that route's already rate-limited until the end of the
        //   'reset' time;
        // - get the global rate;
        // - sleep if there is 0 remaining
        // - then, perform the request
        let bucket = Arc::clone(ROUTES
            .lock()
            .entry(route)
            .or_insert_with(|| {
                Arc::new(Mutex::new(RateLimit {
                    limit: i64::MAX,
                    remaining: i64::MAX,
                    reset: i64::MAX,
                }))
            }));

        let mut lock = bucket.lock();
        lock.pre_hook(&route);

        let response = super::retry(&f)?;

        // Check if an offset has been calculated yet to determine the time
        // difference from Discord can the client.
        //
        // Refer to the documentation for `OFFSET` for more information.
        //
        // This should probably only be a one-time check, although we may want
        // to choose to check this often in the future.
        if unsafe { OFFSET }.is_none() {
            calculate_offset(response.headers.get_raw("date"));
        }

        // Check if the request got ratelimited by checking for status 429,
        // and if so, sleep for the value of the header 'retry-after' -
        // which is in milliseconds - and then `continue` to try again
        //
        // If it didn't ratelimit, subtract one from the RateLimit's
        // 'remaining'
        //
        // Update the 'reset' with the value of the 'x-ratelimit-reset'
        // header
        //
        // It _may_ be possible for the limit to be raised at any time,
        // so check if it did from the value of the 'x-ratelimit-limit'
        // header. If the limit was 5 and is now 7, add 2 to the 'remaining'
        if route == Route::None {
            return Ok(response);
        } else {
            let redo = if response.headers.get_raw("x-ratelimit-global").is_some() {
                let _ = GLOBAL.lock();

                Ok(
                    if let Some(retry_after) = parse_header(&response.headers, "retry-after")? {
                        debug!("Ratelimited on route {:?} for {:?}ms", route, retry_after);
                        thread::sleep(Duration::from_millis(retry_after as u64));

                        true
                    } else {
                        false
                    },
                )
            } else {
                lock.post_hook(&response, &route)
            };

            if !redo.unwrap_or(true) {
                return Ok(response);
            }
        }
    }
}

/// A set of data containing information about the ratelimits for a particular
/// [`Route`], which is stored in the [`ROUTES`] mutex.
///
/// See the [Discord docs] on ratelimits for more information.
///
/// **Note**: You should _not_ mutate any of the fields, as this can help cause
/// 429s.
///
/// [`ROUTES`]: struct.ROUTES.html
/// [`Route`]: enum.Route.html
/// [Discord docs]: https://discordapp.com/developers/docs/topics/rate-limits
#[derive(Clone, Debug, Default)]
pub struct RateLimit {
    /// The total number of requests that can be made in a period of time.
    pub limit: i64,
    /// The number of requests remaining in the period of time.
    pub remaining: i64,
    /// When the interval resets and the the [`limit`] resets to the value of
    /// [`remaining`].
    ///
    /// [`limit`]: #structfield.limit
    /// [`remaining`]: #structfield.remaining
    pub reset: i64,
}

impl RateLimit {
    pub(crate) fn pre_hook(&mut self, route: &Route) {
        if self.limit == 0 {
            return;
        }

        let offset = unsafe { OFFSET }.unwrap_or(0);
        let now = Utc::now().timestamp();
        let current_time = now - offset;

        // The reset was in the past, so we're probably good.
        if current_time > self.reset {
            self.remaining = self.limit;

            return;
        }

        let diff = (self.reset - current_time) as u64;

        if self.remaining == 0 {
            let delay = (diff * 1000) + 500;

            debug!(
                "Pre-emptive ratelimit on route {:?} for {:?}ms",
                route,
                delay
            );
            thread::sleep(Duration::from_millis(delay));

            return;
        }

        self.remaining -= 1;
    }

    pub(crate) fn post_hook(&mut self, response: &Response, route: &Route) -> Result<bool> {
        if let Some(limit) = parse_header(&response.headers, "x-ratelimit-limit")? {
            self.limit = limit;
        }

        if let Some(remaining) = parse_header(&response.headers, "x-ratelimit-remaining")? {
            self.remaining = remaining;
        }

        if let Some(reset) = parse_header(&response.headers, "x-ratelimit-reset")? {
            self.reset = reset;
        }

        Ok(if response.status != StatusCode::TooManyRequests {
            false
        } else if let Some(retry_after) = parse_header(&response.headers, "retry-after")? {
            debug!("Ratelimited on route {:?} for {:?}ms", route, retry_after);
            thread::sleep(Duration::from_millis(retry_after as u64));

            true
        } else {
            false
        })
    }
}
*/

#[allow(dead_code)]
// todo
fn calculate_offset(header: Option<&[Vec<u8>]>) -> Option<i64> {
    // Get the current time as soon as possible.
    let now = Utc::now().timestamp();

    // First get the `Date` header's value and parse it as UTF8.
    let header = header
        .and_then(|h| h.get(0))
        .and_then(|x| str::from_utf8(x).ok());

    if let Some(date) = header {
        // Replace the `GMT` timezone with an offset, and then parse it
        // into a chrono DateTime. If it parses correctly, calculate the
        // diff and then set it as the offset.
        let s = date.replace("GMT", "+0000");
        let parsed = DateTime::parse_from_str(&s, "%a, %d %b %Y %T %z");

        if let Ok(parsed) = parsed {
            let offset = parsed.timestamp();

            let diff = offset - now;

            debug!("[ratelimiting] Set the ratelimit offset to {}", diff);

            return Some(diff);
        }
    }

    None
}

fn parse_header(headers: &HeaderMap, header_raw: &str) -> Result<Option<i64>> {
    headers.get(header_raw).map_or(Ok(None), |header| {
        str::from_utf8(header.as_bytes())
            .map_err(|why| {
                warn!("Error parsing {} as utf8: {:?}", header_raw, why);

                RateLimitError::DecodingUtf8
            })
            .and_then(|v| {
                v.parse::<i64>()
                    .map(Some)
                    .map_err(|why| {
                        warn!("Error parsing {}: {:?} to i64", header_raw, why);

                        RateLimitError::DecodingInteger
                    })
            })
            .map_err(From::from)
    })
}
