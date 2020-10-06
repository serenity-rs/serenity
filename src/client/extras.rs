use super::{EventHandler, RawEventHandler};
use crate::client::bridge::gateway::GatewayIntents;

use std::fmt;
use std::sync::Arc;

#[cfg(feature = "cache")]
use std::time::Duration;

#[cfg(feature = "framework")]
use crate::framework::Framework;

/// A builder to extra things for altering the [`Client`].
///
/// [`Client`]: ../struct.Client.html
#[derive(Clone)]
pub struct Extras {
    pub(crate) event_handler: Option<Arc<dyn EventHandler>>,
    pub(crate) raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub(crate) framework: Arc<Option<Box<dyn Framework + Send + Sync + 'static>>>,
    #[cfg(feature = "cache")]
    pub(crate) timeout: Option<Duration>,
    pub(crate) guild_subscriptions: bool,
    pub(crate) intents: Option<GatewayIntents>,
}

impl Extras {
    /// Set the handler for managing discord events.
    pub fn event_handler<H>(&mut self, handler: H) -> &mut Self
    where
        H: EventHandler + 'static,
    {
        self.event_handler = Some(Arc::new(handler));
        self
    }

    /// Set the handler for raw events.
    ///
    /// If you have set the specialised [`event_handler`], all events
    /// will be cloned for use to the raw event handler.
    ///
    /// [`event_handler`]: #method.event_handler
    pub fn raw_event_handler<H>(&mut self, handler: H) -> &mut Self
    where
        H: RawEventHandler + 'static,
    {
        self.raw_event_handler = Some(Arc::new(handler));
        self
    }

    /// Set the framework.
    #[cfg(feature = "framework")]
    pub fn framework<F: Framework + Send + Sync + 'static>(&mut self, framework: F) -> &mut Self {
        self.framework = Arc::new(Some(Box::new(framework)));
        self
    }

    /// Set the duration the library is permitted to update the cache
    /// before giving up acquiring a write-lock.
    ///
    /// This can be useful for avoiding deadlocks, but it also may invalidate your cache.
    #[cfg(feature = "cache")]
    pub fn cache_update_timeout(&mut self, duration: Duration) -> &mut Self {
        self.timeout = Some(duration);
        self
    }

    /// Set whether the library should subscribe for listening to presence and typing events.
    ///
    /// By default, this is `true`.
    pub fn guild_subscriptions(&mut self, guild_subscriptions: bool) -> &mut Self {
        self.guild_subscriptions = guild_subscriptions;
        self
    }

    /// Set what Discord gateway events shall be received.
    ///
    /// By default, no intents are being used and all events are received.
    pub fn intents(&mut self, intents: GatewayIntents) -> &mut Self {
        self.intents = Some(intents);
        self
    }
}

impl Default for Extras {
    fn default() -> Self {
        Extras {
            event_handler: None,
            raw_event_handler: None,
            #[cfg(feature = "framework")]
            framework: Arc::new(None),
            #[cfg(feature = "cache")]
            timeout: None,
            guild_subscriptions: true,
            intents: None,
        }
    }
}

impl fmt::Debug for Extras {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        struct EventHandler;

        #[derive(Debug)]
        struct RawEventHandler;

        let mut ds = f.debug_struct("Extras");

        ds.field("event_handler", &EventHandler);
        ds.field("raw_event_handler", &RawEventHandler);
        #[cfg(feature = "cache")]
        ds.field("cache_update_timeout", &self.timeout);
        ds.field("intents", &self.intents);

        ds.finish()
    }
}
