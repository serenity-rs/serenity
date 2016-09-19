use hyper::client::{RequestBuilder, Response};
use hyper::header::Headers;
use hyper::status::StatusCode;
use std::collections::HashMap;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use super::http;
use time;
use ::prelude::*;

lazy_static! {
    static ref GLOBAL: Arc<Mutex<RateLimit>> = Arc::new(Mutex::new(RateLimit::default()));
    static ref ROUTES: Arc<Mutex<HashMap<Route, RateLimit>>> = Arc::new(Mutex::new(HashMap::default()));
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Route {
    ChannelsId,
    ChannelsIdInvites,
    ChannelsIdMessages,
    ChannelsIdMessagesBulkDelete,
    ChannelsIdMessagesId,
    ChannelsIdPermissionsOverwriteId,
    ChannelsIdPins,
    ChannelsIdPinsMessageId,
    ChannelsIdTyping,
    Gateway,
    Global,
    Guilds,
    GuildsId,
    GuildsIdBans,
    GuildsIdBansUserId,
    GuildsIdChannels,
    GuildsIdEmbed,
    GuildsIdEmojis,
    GuildsIdEmojisId,
    GuildsIdIntegrations,
    GuildsIdIntegrationsId,
    GuildsIdIntegrationsIdSync,
    GuildsIdInvites,
    GuildsIdMembers,
    GuildsIdMembersId,
    GuildsIdPrune,
    GuildsIdRegions,
    GuildsIdRoles,
    GuildsIdRolesId,
    InvitesCode,
    Users,
    UsersId,
    UsersMe,
    UsersMeChannels,
    USersMeConnections,
    UsersMeGuilds,
    UsersMeGuildsId,
    VoiceRegions,
    None,
}

pub fn perform<'a, F>(route: Route, f: F) -> Result<Response>
    where F: Fn() -> RequestBuilder<'a> {
    // Keeping the global lock poisoned here for the duration of the function
    // will ensure that requests are synchronous, which will further ensure
    // that 429s are _never_ hit.
    //
    // This would otherwise cause the potential for 429s to be hit while
    // requests are open.
    let mut global = GLOBAL.lock().expect("global route lock poisoned");

    loop {
        // Perform pre-checking here:
        //
        // - get the route's relevant rate
        // - sleep if that route's already rate-limited until the end of the
        //   'reset' time;
        // - get the global rate;
        // - sleep if there is 0 remaining
        // - then, perform the request
        global.pre_hook();

        if let Some(route) = ROUTES.lock().expect("routes poisoned").get_mut(&route) {
            route.pre_hook();
        }

        let response = try!(http::retry(&f));

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

        let redo = if response.headers.get_raw("x-ratelimit-global").is_some() {
            global.post_hook(&response)
        } else {
            ROUTES.lock()
                .expect("routes poisoned")
                .entry(route)
                .or_insert_with(RateLimit::default)
                .post_hook(&response)
        };

        if redo.unwrap_or(false) {
            continue;
        }

        return Ok(response);
    }
}

#[derive(Clone, Debug, Default)]
pub struct RateLimit {
    limit: i64,
    remaining: i64,
    reset: i64,
}

impl RateLimit {
    pub fn pre_hook(&mut self) {
        if self.limit == 0 {
            return;
        }

        let diff = (self.reset - time::get_time().sec) as u64;

        if self.remaining == 0 {
            let delay = (diff * 1000) + 500;

            debug!("Pre-emptive ratelimit for {:?}ms", delay);
            thread::sleep(Duration::from_millis(delay));

            return;
        }

        self.remaining -= 1;
    }

    pub fn post_hook(&mut self, response: &Response) -> Result<bool> {
        if let Some(limit) = try!(get_header(&response.headers, "x-ratelimit-limit")) {
            self.limit = limit;
        }

        if let Some(remaining) = try!(get_header(&response.headers, "x-ratelimit-remaining")) {
            self.remaining = remaining;
        }

        if let Some(reset) = try!(get_header(&response.headers, "x-ratelimit-reset")) {
            self.reset = reset;
        }

        Ok(if response.status != StatusCode::TooManyRequests {
            false
        } else if let Some(retry_after) = try!(get_header(&response.headers, "retry-after")) {
            debug!("Ratelimited: {:?}ms", retry_after);
            thread::sleep(Duration::from_millis(retry_after as u64));

            true
        } else {
            false
        })
    }
}

fn get_header(headers: &Headers, header: &str) -> Result<Option<i64>> {
    match headers.get_raw(header) {
        Some(header) => match str::from_utf8(&header[0]) {
            Ok(v) => match v.parse::<i64>() {
                Ok(v) => Ok(Some(v)),
                Err(_why) => Err(Error::Client(ClientError::RateLimitI64)),
            },
            Err(_why) => Err(Error::Client(ClientError::RateLimitUtf8)),
        },
        None => Ok(None),
    }
}
