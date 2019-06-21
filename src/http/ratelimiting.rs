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

use chrono::{DateTime, Utc};
use reqwest::{
    Response,
    header::HeaderMap,
    StatusCode,
};
use crate::internal::prelude::*;
use parking_lot::Mutex;
use std::{
    sync::Arc,
    time::Duration,
    str,
    thread,
    i64,
};
use super::{Http, HttpError, Request};
use log::debug;

/// Refer to [`offset`].
///
/// [`offset`]: fn.offset.html
static mut OFFSET: Option<i64> = None;

pub(super) fn perform(http: &Http, req: Request<'_>) -> Result<Response> {
    loop {
        // This will block if another thread is trying to send
        // an HTTP-request already (due to receiving an x-ratelimit-global).
        let _ = http.limiter.lock();

        // Destructure the tuple instead of retrieving the third value to
        // take advantage of the type system. If `RouteInfo::deconstruct`
        // returns a different number of tuple elements in the future, directly
        // accessing a certain index (e.g. `req.route.deconstruct().1`) would
        // mean this code would not indicate it might need to be updated for the
        // new tuple element amount.
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
        let bucket = Arc::clone(http.routes
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

        let response = http.retry(&req)?;

        // Check if an offset has been calculated yet to determine the time
        // difference from Discord can the client.
        //
        // Refer to the documentation for `OFFSET` for more information.
        //
        // This should probably only be a one-time check, although we may want
        // to choose to check this often in the future.
        if unsafe { OFFSET }.is_none() {
            calculate_offset(&response.headers().get("date").and_then(|d| Some(d.as_bytes())));
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
            let redo = if response.headers().get("x-ratelimit-global").is_some() {
                let _ = http.limiter.lock();

                Ok(
                    if let Some(retry_after) = parse_header(&response.headers(), "retry-after")? {
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
            let delay = diff * 1000;

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
        if let Some(limit) = parse_header(&response.headers(), "x-ratelimit-limit")? {
            self.limit = limit;
        }

        if let Some(remaining) = parse_header(&response.headers(), "x-ratelimit-remaining")? {
            self.remaining = remaining;
        }

        if let Some(reset) = parse_header(&response.headers(), "x-ratelimit-reset")? {
            self.reset = reset;
        }

        Ok(if response.status() != StatusCode::TOO_MANY_REQUESTS {
            false
        } else if let Some(retry_after) = parse_header(&response.headers(), "retry-after")? {
            debug!("Ratelimited on route {:?} for {:?}ms", route, retry_after);
            thread::sleep(Duration::from_millis(retry_after as u64));

            true
        } else {
            false
        })
    }
}

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
///
/// This will return `None` if an HTTP request hasn't been made, meaning that
/// no offset could have been calculated.
pub fn offset() -> Option<i64> {
    unsafe { OFFSET }
}

fn calculate_offset(header: &Option<&[u8]>) {
    // Get the current time as soon as possible.
    let now = Utc::now().timestamp();

    let header = header.and_then(|x| str::from_utf8(x).ok());

    if let Some(header) = header {
        // Replace the `GMT` timezone with an offset, and then parse it
        // into a chrono DateTime. If it parses correctly, calculate the
        // diff and then set it as the offset.
        let s = header.replace("GMT", "+0000");

        let parsed = DateTime::parse_from_str(&s, "%a, %d %b %Y %T %z");

        if let Ok(parsed) = parsed {
            let offset = parsed.timestamp();

            let diff = offset - now;

            unsafe {
                OFFSET = Some(diff);

                debug!("[ratelimiting] Set the ratelimit offset to {}", diff);
            }
        }
    }

}

fn parse_header(headers: &HeaderMap, header: &str) -> Result<Option<i64>> {
    let header = match headers.get(header) {
        Some(v) => v,
        None => return Ok(None),
    };

    let unicode = str::from_utf8(&header.as_bytes()).map_err(|_| {
        Error::from(HttpError::RateLimitUtf8)
    })?;

    let num = unicode.parse().map_err(|_| {
        Error::from(HttpError::RateLimitI64)
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
                HeaderValue::from_static("1560704880"),
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

        assert_eq!(parse_header(&headers, "x-ratelimit-limit")?.unwrap(), 5);
        assert_eq!(
            parse_header(&headers, "x-ratelimit-remaining")?.unwrap(),
            4,
        );
        assert_eq!(
            parse_header(&headers, "x-ratelimit-reset")?.unwrap(),
            1_560_704_880,
        );

        Ok(())
    }

    #[test]
    fn test_parse_header_errors() -> Result<()> {
        let headers = headers();

        match parse_header(&headers, "x-bad-num").unwrap_err() {
            Error::Http(x) => match *x {
                HttpError::RateLimitI64 => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        match parse_header(&headers, "x-bad-unicode").unwrap_err() {
            Error::Http(http_err) => match *http_err {
                HttpError::RateLimitUtf8 => assert!(true),
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        Ok(())
    }
}
