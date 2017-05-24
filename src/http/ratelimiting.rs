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
#![allow(zero_ptr)]

use hyper::client::{RequestBuilder, Response};
use hyper::header::Headers;
use hyper::status::StatusCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{str, thread};
use super::{HttpError, LightMethod};
use time;
use ::internal::prelude::*;

lazy_static! {
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
    pub static ref GLOBAL: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
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
    /// let routes = ROUTES.lock().unwrap();
    ///
    /// if let Some(route) = routes.get(&Route::ChannelsId(7)) {
    ///     println!("Reset time at: {}", route.reset);
    /// }
    /// ```
    ///
    /// [`RateLimit`]: struct.RateLimit.html
    /// [`Route`]: enum.Route.html
    pub static ref ROUTES: Arc<Mutex<HashMap<Route, RateLimit>>> = Arc::new(Mutex::new(HashMap::default()));
}

/// A representation of all routes registered within the library. These are safe
/// and memory-efficient representations of each path that functions exist for
/// in the [`http`] module.
///
/// [`http`]: ../index.html
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Route {
    /// Route for the `/channels/:channel_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsId(u64),
    /// Route for the `/channels/:channel_id/invites` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdInvites(u64),
    /// Route for the `/channels/:channel_id/messages` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdMessages(u64),
    /// Route for the `/channels/:channel_id/messages/bulk-delete` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdMessagesBulkDelete(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    // This route is a unique case. The ratelimit for message _deletions_ is
    // different than the overall route ratelimit.
    //
    // Refer to the docs on [Rate Limits] in the yellow warning section.
    //
    // Additionally, this needs to be a `LightMethod` from the parent module
    // and _not_ a `hyper` `Method` due to `hyper`'s not deriving `Copy`.
    //
    // [Rate Limits]: https://discordapp.com/developers/docs/topics/rate-limits
    ChannelsIdMessagesId(LightMethod, u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/ack` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdMessagesIdAck(u64),
    /// Route for the `/channels/:channel_id/messages/:message_id/reactions`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdMessagesIdReactions(u64),
    /// Route for the
    /// `/channels/:channel_id/messages/:message_id/reactions/:reaction/@me`
    /// path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdMessagesIdReactionsUserIdType(u64),
    /// Route for the `/channels/:channel_id/permissions/:target_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdPermissionsOverwriteId(u64),
    /// Route for the `/channels/:channel_id/pins` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdPins(u64),
    /// Route for the `/channels/:channel_id/pins/:message_id` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdPinsMessageId(u64),
    /// Route for the `/channels/:channel_id/typing` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdTyping(u64),
    /// Route for the `/channels/:channel_id/webhooks` path.
    ///
    /// The data is the relevant [`ChannelId`].
    ///
    /// [`ChannelId`]: ../../model/struct.ChannelId.html
    ChannelsIdWebhooks(u64),
    /// Route for the `/gateway` path.
    Gateway,
    /// Route for the `/gateway/bot` path.
    GatewayBot,
    /// Route for the `/guilds` path.
    Guilds,
    /// Route for the `/guilds/:guild_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsId(u64),
    /// Route for the `/guilds/:guild_id/bans` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdBans(u64),
    /// Route for the `/guilds/:guild_id/bans/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdBansUserId(u64),
    /// Route for the `/guilds/:guild_id/channels/:channel_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdChannels(u64),
    /// Route for the `/guilds/:guild_id/embed` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmbed(u64),
    /// Route for the `/guilds/:guild_id/emojis` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmojis(u64),
    /// Route for the `/guilds/:guild_id/emojis/:emoji_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdEmojisId(u64),
    /// Route for the `/guilds/:guild_id/integrations` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrations(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrationsId(u64),
    /// Route for the `/guilds/:guild_id/integrations/:integration_id/sync`
    /// path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdIntegrationsIdSync(u64),
    /// Route for the `/guilds/:guild_id/invites` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdInvites(u64),
    /// Route for the `/guilds/:guild_id/members` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembers(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersId(u64),
    /// Route for the `/guilds/:guild_id/members/:user_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/members/@me/nick` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdMembersMeNick(u64),
    /// Route for the `/guilds/:guild_id/prune` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdPrune(u64),
    /// Route for the `/guilds/:guild_id/regions` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRegions(u64),
    /// Route for the `/guilds/:guild_id/roles` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRoles(u64),
    /// Route for the `/guilds/:guild_id/roles/:role_id` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdRolesId(u64),
    /// Route for the `/guilds/:guild_id/webhooks` path.
    ///
    /// The data is the relevant [`GuildId`].
    ///
    /// [`GuildId`]: struct.GuildId.html
    GuildsIdWebhooks(u64),
    /// Route for the `/invites/:code` path.
    InvitesCode,
    /// Route for the `/users/:user_id` path.
    UsersId,
    /// Route for the `/users/@me` path.
    UsersMe,
    /// Route for the `/users/@me/channels` path.
    UsersMeChannels,
    /// Route for the `/users/@me/guilds` path.
    UsersMeGuilds,
    /// Route for the `/users/@me/guilds/:guild_id` path.
    UsersMeGuildsId,
    /// Route for the `/voice/regions` path.
    VoiceRegions,
    /// Route for the `/webhooks/:webhook_id` path.
    WebhooksId,
    /// Route where no ratelimit headers are in place (i.e. user account-only
    /// routes).
    ///
    /// This is a special case, in that if the route is `None` then pre- and
    /// post-hooks are not executed.
    None,
}

#[doc(hidden)]
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
        if route == Route::None {
            return Ok(response);
        } else {
            let redo = if response.headers.get_raw("x-ratelimit-global").is_some() {
                let _ = GLOBAL.lock().expect("global route lock poisoned");

                Ok(if let Some(retry_after) = parse_header(&response.headers, "retry-after")? {
                    debug!("Ratelimited: {:?}ms", retry_after);
                    thread::sleep(Duration::from_millis(retry_after as u64));

                    true
                } else {
                    false
                })
            } else {
                ROUTES.lock()
                    .expect("routes poisoned")
                    .entry(route)
                    .or_insert_with(RateLimit::default)
                    .post_hook(&response)
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
    #[doc(hidden)]
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

    #[doc(hidden)]
    pub fn post_hook(&mut self, response: &Response) -> Result<bool> {
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
            debug!("Ratelimited: {:?}ms", retry_after);
            thread::sleep(Duration::from_millis(retry_after as u64));

            true
        } else {
            false
        })
    }
}

fn parse_header(headers: &Headers, header: &str) -> Result<Option<i64>> {
    match headers.get_raw(header) {
        Some(header) => match str::from_utf8(&header[0]) {
            Ok(v) => match v.parse::<i64>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Err(Error::Http(HttpError::RateLimitI64)),
            },
            Err(_) => Err(Error::Http(HttpError::RateLimitUtf8)),
        },
        None => Ok(None),
    }
}
