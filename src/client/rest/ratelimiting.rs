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
//! the major parameter - `10` is equivilant in both URIs' format.
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
//! response, as the major parameter - `channel_id` - is equivilant for the two
//! requests (`10`).
//!
//!
//! With the examples out of the way: major parameters are why some variants
//! (i.e. all of the channel/guild variants) have an associated u64 as data.
//! This is the Id of the parameter, differentiating between different
//! ratelimits.

use hyper::client::{RequestBuilder, Response};
use hyper::header::Headers;
use hyper::status::StatusCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{str, thread};
use time;
use ::internal::prelude::*;

lazy_static! {
    static ref GLOBAL: Arc<Mutex<RateLimit>> = Arc::new(Mutex::new(RateLimit::default()));
    static ref ROUTES: Arc<Mutex<HashMap<Route, RateLimit>>> = Arc::new(Mutex::new(HashMap::default()));
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Route {
    ChannelsId(u64),
    ChannelsIdInvites(u64),
    ChannelsIdMessages(u64),
    ChannelsIdMessagesBulkDelete(u64),
    ChannelsIdMessagesId(u64),
    ChannelsIdMessagesIdAck(u64),
    ChannelsIdMessagesIdReactions(u64),
    ChannelsIdMessagesIdReactionsUserIdType(u64),
    ChannelsIdPermissionsOverwriteId(u64),
    ChannelsIdPins(u64),
    ChannelsIdPinsMessageId(u64),
    ChannelsIdTyping(u64),
    ChannelsIdWebhooks(u64),
    Gateway,
    GatewayBot,
    Guilds,
    GuildsId(u64),
    GuildsIdBans(u64),
    GuildsIdBansUserId(u64),
    GuildsIdChannels(u64),
    GuildsIdEmbed(u64),
    GuildsIdEmojis(u64),
    GuildsIdEmojisId(u64),
    GuildsIdIntegrations(u64),
    GuildsIdIntegrationsId(u64),
    GuildsIdIntegrationsIdSync(u64),
    GuildsIdInvites(u64),
    GuildsIdMembers(u64),
    GuildsIdMembersId(u64),
    GuildsIdMembersIdRolesId(u64),
    GuildsIdMembersMeNick(u64),
    GuildsIdPrune(u64),
    GuildsIdRegions(u64),
    GuildsIdRoles(u64),
    GuildsIdRolesId(u64),
    GuildsIdWebhooks(u64),
    InvitesCode,
    UsersId,
    UsersMe,
    UsersMeChannels,
    UsersMeConnections,
    UsersMeGuilds,
    UsersMeGuildsId,
    VoiceRegions,
    WebhooksId,
    None,
}

pub fn perform<'a, F>(route: Route, f: F) -> Result<Response>
    where F: Fn() -> RequestBuilder<'a> {

    loop {
        {
            // This will block if another thread already has the global
            // unlocked already (due to receiving an x-ratelimit-global).
            let mut _global = GLOBAL.lock().expect("global route lock poisoned");
        }

        // Perform pre-checking here:
        //
        // - get the route's relevant rate
        // - sleep if that route's already rate-limited until the end of the
        //   'reset' time;
        // - get the global rate;
        // - sleep if there is 0 remaining
        // - then, perform the request
        if route != Route::None {
            if let Some(route) = ROUTES.lock().expect("routes poisoned").get_mut(&route) {
                route.pre_hook();
            }
        }

        let response = super::retry(&f)?;

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
        if route != Route::None {
            let redo = if response.headers.get_raw("x-ratelimit-global").is_some() {
                let mut global = GLOBAL.lock().expect("global route lock poisoned");
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

        let current_time = time::get_time().sec;

        // The reset was in the past, so we're probably good.
        if current_time > self.reset {
            self.remaining = self.limit;

            return;
        }

        let diff = (self.reset - current_time) as u64;

        if self.remaining == 0 {
            let delay = (diff * 1000) + 500;

            debug!("Pre-emptive ratelimit for {:?}ms", delay);
            thread::sleep(Duration::from_millis(delay));

            return;
        }

        self.remaining -= 1;
    }

    pub fn post_hook(&mut self, response: &Response) -> Result<bool> {
        if let Some(limit) = get_header(&response.headers, "x-ratelimit-limit")? {
            self.limit = limit;
        }

        if let Some(remaining) = get_header(&response.headers, "x-ratelimit-remaining")? {
            self.remaining = remaining;
        }

        if let Some(reset) = get_header(&response.headers, "x-ratelimit-reset")? {
            self.reset = reset;
        }

        Ok(if response.status != StatusCode::TooManyRequests {
            false
        } else if let Some(retry_after) = get_header(&response.headers, "retry-after")? {
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
