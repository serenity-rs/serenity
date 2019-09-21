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
//! only current major parameters are `channel_id`, `guild_id` and `webhook_id`.
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

pub use super::routing::Route;

use reqwest::{
    Client,
    Response,
    header::HeaderMap,
    StatusCode,
};
use crate::internal::prelude::*;
use parking_lot::{Mutex, RwLock};
use std::{
    collections::HashMap,
    sync::Arc,
    str::{
        self,
        FromStr,
    },
    time::Duration,
    thread,
    i64,
    u64,
};
use super::{HttpError, Request};
use log::debug;

/// Ratelimiter for requests to the Discord API.
///
/// This keeps track of ratelimit data for known routes through the
/// [`Ratelimit`] implementation for each route: how many tickets are
/// [`remaining`] until the user needs to wait for the known [`reset`] time, and
/// the [`limit`] of requests that can be made within that time.
///
/// When no tickets are available for some time, then the thread sleeps until
/// that time passes. The mechanism is known as "pre-emptive ratelimiting".
///
/// Occasionally for very high traffic bots, a global ratelimit may be reached
/// which blocks all future requests until the global ratelimit is over,
/// regardless of route. The value of this global ratelimit is never given
/// through the API, so it can't be pre-emptively ratelimited. This only affects
/// the largest of bots.
pub struct Ratelimiter {
    client: Arc<Client>,
    global: Arc<Mutex<()>>,
    // When futures is implemented, make tasks clear out their respective entry
    // when the 'reset' passes.
    routes: Arc<RwLock<HashMap<Route, Arc<Mutex<Ratelimit>>>>>,
    token: String,
}

impl Ratelimiter {
    /// Creates a new ratelimiter, with a shared `reqwest` client and the
    /// bot's token.
    ///
    /// The bot token must be prefixed with `"Bot "`. The ratelimiter does not
    /// prefix it.
    pub fn new(client: Arc<Client>, token: impl Into<String>) -> Self {
        Self::_new(client, token.into())
    }

    fn _new(client: Arc<Client>, token: String) -> Self {
        Self {
            client,
            global: Default::default(),
            routes: Default::default(),
            token,
        }
    }

    /// The routes mutex is a HashMap of each [`Route`] and their respective
    /// ratelimit information.
    ///
    /// See the documentation for [`Ratelimit`] for more information on how the
    /// library handles ratelimiting.
    ///
    /// # Examples
    ///
    /// View the `reset` time of the route for `ChannelsId(7)`:
    ///
    /// ```rust,no_run
    /// use serenity::http::{ratelimiting::{Route}};
    /// # use serenity::http::Http;
    /// # let http = Http::default();
    /// let routes = http.ratelimiter.routes();
    /// let reader = routes.read();
    ///
    /// if let Some(route) = reader.get(&Route::ChannelsId(7)) {
    ///     println!("Reset time at: {}", route.lock().reset());
    /// }
    /// ```
    ///
    /// [`Ratelimit`]: struct.Ratelimit.html
    /// [`Route`]: ../routing/enum.Route.html
    pub fn routes(&self) -> Arc<RwLock<HashMap<Route, Arc<Mutex<Ratelimit>>>>> {
        Arc::clone(&self.routes)
    }

    pub fn perform(&self, req: RatelimitedRequest<'_>) -> Result<Response> {
        let RatelimitedRequest { req } = req;

        loop {
            // This will block if another thread hit the global ratelimit.
            let _ = self.global.lock();

            // Destructure the tuple instead of retrieving the third value to
            // take advantage of the type system. If `RouteInfo::deconstruct`
            // returns a different number of tuple elements in the future,
            // directly accessing a certain index
            // (e.g. `req.route.deconstruct().1`) would mean this code would not
            // indicate it might need to be updated for the new tuple element
            // amount.
            //
            // This isn't normally important, but might be for ratelimiting.
            let (_, route, _) = req.route.deconstruct();

            // Perform pre-checking here:
            //
            // - get the route's relevant rate
            // - sleep if that route's already rate-limited until the end of the
            //   'reset' time;
            // - get the global rate;
            // - sleep if there is 0 remaining
            // - then, perform the request
            let bucket = Arc::clone(&self.routes
                .write()
                .entry(route)
                .or_default());

            bucket.lock().pre_hook(&route);

            let request = req.build(&self.client, &self.token)?;
            let response = request.send()?;

            // Check if the request got ratelimited by checking for status 429,
            // and if so, sleep for the value of the header 'retry-after' -
            // which is in milliseconds - and then `continue` to try again
            //
            // If it didn't ratelimit, subtract one from the Ratelimit's
            // 'remaining'
            //
            // Update `reset` with the value of 'x-ratelimit-reset' header.
            // Similarly, update `reset-after` with the 'x-ratelimit-reset-after' header.
            //
            // It _may_ be possible for the limit to be raised at any time,
            // so check if it did from the value of the 'x-ratelimit-limit'
            // header. If the limit was 5 and is now 7, add 2 to the 'remaining'
            if route == Route::None {
                return Ok(response);
            } else {
                let redo = if response.headers().get("x-ratelimit-global").is_some() {
                    let _ = self.global.lock();

                    Ok(
                        if let Some(retry_after) = parse_header::<u64>(&response.headers(), "retry-after")? {
                            debug!("Ratelimited on route {:?} for {:?}ms", route, retry_after);
                            thread::sleep(Duration::from_millis(retry_after));

                            true
                        } else {
                            false
                        },
                    )
                } else {
                    bucket.lock().post_hook(&response, &route)
                };

                if !redo.unwrap_or(true) {
                    return Ok(response);
                }
            }
        }
    }
}

/// A set of data containing information about the ratelimits for a particular
/// [`Route`], which is stored in [`Http`].
///
/// See the [Discord docs] on ratelimits for more information.
///
/// **Note**: You should _not_ mutate any of the fields, as this can help cause
/// 429s.
///
/// [`Http`]: ../client/struct.Http.html#structfield.routes
/// [`Route`]: ../routing/enum.Route.html
/// [Discord docs]: https://discordapp.com/developers/docs/topics/rate-limits
#[derive(Debug)]
pub struct Ratelimit {
    /// The total number of requests that can be made in a period of time.
    limit: i64,
    /// The number of requests remaining in the period of time.
    remaining: i64,
    /// The absolute time in milliseconds when the interval resets.
    reset: i64,
    /// The total time in milliseconds when the interval resets.
    reset_after: i64,
}

impl Ratelimit {
    #[cfg(feature = "absolute_ratelimits")]
    fn get_delay(&self) -> i64 {
        let now = Utc::now().timestamp_millis();
        self.reset - now
    }

    #[cfg(not(feature = "absolute_ratelimits"))]
    fn get_delay(&self) -> i64 {
        self.reset_after
    }

    pub fn pre_hook(&mut self, route: &Route) {
        if self.limit() == 0 {
            return;
        }

		let delay = self.get_delay();

		if delay < 0 {
			// We're probably in the past.
			self.remaining = self.limit;

			return;
		}

        if self.remaining() == 0 {
            let delay = delay as u64;

            debug!(
                "Pre-emptive ratelimit on route {:?} for {:?}ms",
                route,
                delay,
            );

            thread::sleep(Duration::from_millis(delay));

            return;
        }

        self.remaining -= 1;
    }

    pub fn post_hook(&mut self, response: &Response, route: &Route) -> Result<bool> {
        if let Some(limit) = parse_header(&response.headers(), "x-ratelimit-limit")? {
            self.limit = limit;
        }

        if let Some(remaining) = parse_header(&response.headers(), "x-ratelimit-remaining")? {
            self.remaining = remaining;
        }

        if let Some(reset) = parse_header::<f64>(&response.headers(), "x-ratelimit-reset")? {
            self.reset = (reset * 1000f64) as i64;
        }

        if let Some(reset_after) = parse_header::<f64>(&response.headers(), "x-ratelimit-reset-after")? {
            self.reset_after = (reset_after * 1000f64) as i64;
        }

        Ok(if response.status() != StatusCode::TOO_MANY_REQUESTS {
            false
        } else if let Some(retry_after) = parse_header::<u64>(&response.headers(), "retry-after")? {
            debug!("Ratelimited on route {:?} for {:?}ms", route, retry_after);
            thread::sleep(Duration::from_millis(retry_after));

            true
        } else {
            false
        })
    }

    /// The total number of requests that can be made in a period of time.
    #[inline]
    pub fn limit(&self) -> i64 {
        self.limit
    }

    /// The number of requests remaining in the period of time.
    #[inline]
    pub fn remaining(&self) -> i64 {
        self.remaining
    }

    /// The absolute time in milliseconds when the interval resets.
    #[inline]
    pub fn reset(&self) -> i64 {
        self.reset
    }

    /// The total time in milliseconds when the interval resets.
    #[inline]
    pub fn reset_after(&self) -> i64 {
        self.reset_after
    }
}

impl Default for Ratelimit {
    fn default() -> Self {
        Self {
            limit: i64::MAX,
            remaining: i64::MAX,
            reset: i64::MAX,
            reset_after: i64::MAX,
        }
    }
}

/// Information about a request for the ratelimiter to perform.
///
/// This only contains the basic information needed by the ratelimiter to
/// perform a full cycle of making the request and returning the response.
///
/// Use the `From` implementations for making one of these.
pub struct RatelimitedRequest<'a> {
    req: Request<'a>,
}

impl<'a> From<Request<'a>> for RatelimitedRequest<'a> {
    fn from(req: Request<'a>) -> Self {
        Self { req }
    }
}

fn parse_header<T: FromStr>(headers: &HeaderMap, header: &str) -> Result<Option<T>> {
    let header = match headers.get(header) {
        Some(v) => v,
        None => return Ok(None),
    };

    let unicode = str::from_utf8(&header.as_bytes()).map_err(|_| {
        Error::from(HttpError::RateLimitUtf8)
    })?;

    let num = unicode.parse().map_err(|_| {
        Error::from(HttpError::RateLimitI64F64)
    })?;

    Ok(Some(num))
}

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        http::HttpError,
    };
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use std::{
        error::Error as StdError,
        result::Result as StdResult,
    };
    use super::parse_header;

    type Result<T> = StdResult<T, Box<dyn StdError>>;

    fn headers() -> HeaderMap {
        let pairs = &[
            (
                HeaderName::from_static("x-ratelimit-limit"),
                HeaderValue::from_static("5"),
            ),
            (
                HeaderName::from_static("x-ratelimit-remaining"),
                HeaderValue::from_static("4"),
            ),
            (
                HeaderName::from_static("x-ratelimit-reset"),
                HeaderValue::from_static("1560704880.423"),
            ),
            (
                HeaderName::from_static("x-bad-num"),
                HeaderValue::from_static("abc"),
            ),
            (
                HeaderName::from_static("x-bad-unicode"),
                HeaderValue::from_bytes(&[255, 255, 255, 255]).unwrap(),
            ),
        ];

        let mut map = HeaderMap::with_capacity(pairs.len());

        for (name, val) in pairs.into_iter() {
            map.insert(name, val.to_owned());
        }

        map
    }

    #[test]
    fn test_parse_header_good() -> Result<()> {
        let headers = headers();

        assert_eq!(parse_header::<i64>(&headers, "x-ratelimit-limit")?.unwrap(), 5);
        assert_eq!(
            parse_header::<i64>(&headers, "x-ratelimit-remaining")?.unwrap(),
            4,
        );
        assert_eq!(
            parse_header::<f64>(&headers, "x-ratelimit-reset")?.unwrap(),
            1_560_704_880.423,
        );

        Ok(())
    }

    #[test]
    fn test_parse_header_errors() -> Result<()> {
        let headers = headers();

        match parse_header::<i64>(&headers, "x-bad-num").unwrap_err() {
            Error::Http(x) => match *x {
                HttpError::RateLimitI64F64 => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        match parse_header::<i64>(&headers, "x-bad-unicode").unwrap_err() {
            Error::Http(http_err) => match *http_err {
                HttpError::RateLimitUtf8 => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        Ok(())
    }
}
