use std::sync::Arc;

use super::Context;
use super::event_handler::{EventHandler, RawEventHandler};
#[cfg(feature = "framework")]
use crate::framework::Framework;
#[cfg(feature = "voice")]
use crate::gateway::VoiceGatewayManager;
use crate::internal::tokio::spawn_named;
use crate::model::event::{Event, FullEvent};

pub struct EventDispatcher {
    pub context: Context,
    pub event_handler: Option<Arc<dyn EventHandler>>,
    pub raw_event_handler: Option<Arc<dyn RawEventHandler>>,
    #[cfg(feature = "framework")]
    pub framework: Option<Arc<dyn Framework>>,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<Arc<dyn VoiceGatewayManager + 'static>>,
}

impl EventDispatcher {
    pub async fn dispatch(&self, mut event: Box<Event>) {
        #[cfg(feature = "voice")]
        {
            if let Some(voice_manager) = &self.voice_manager {
                match &*event {
                    Event::Ready(_) => {
                        voice_manager
                            .register_shard(self.context.shard_id.0, self.context.shard.clone())
                            .await;
                    },
                    Event::VoiceServerUpdate(event) => {
                        voice_manager
                            .server_update(event.guild_id, event.endpoint.as_deref(), &event.token)
                            .await;
                    },
                    Event::VoiceStateUpdate(event) => {
                        if let Some(guild_id) = event.voice_state.guild_id {
                            voice_manager.state_update(guild_id, &event.voice_state).await;
                        }
                    },
                    _ => {},
                }
            }
        }

        if let Some(handler) = self.event_handler.as_ref() {
            if let Some(new_event) = handler.filter_event(&self.context, event) {
                event = new_event;
            } else {
                return;
            }
        }

        if let Some(handler) = self.raw_event_handler.as_ref() {
            if let Some(new_event) = handler.filter_event(&self.context, event) {
                event = new_event;
            } else {
                return;
            }
        }

        #[cfg(feature = "collector")]
        self.context.collectors.write().retain(|callback| (callback.0)(&event));

        if let Some(raw_handler) = &self.raw_event_handler {
            raw_handler.raw_event(self.context.clone(), &event).await;
        }

        let mut extra_event = None;
        let full_event = FullEvent::from_event(
            event,
            &mut extra_event,
            #[cfg(feature = "cache")]
            &self.context.cache,
        );

        #[cfg(feature = "framework")]
        let framework = self.framework.clone();
        let event_handler = self.event_handler.clone();
        let context = self.context.clone();

        spawn_named("dispatch::user", async move {
            #[cfg(feature = "framework")]
            tokio::join!(
                dispatch_framework(&context, framework, &full_event, extra_event.as_ref()),
                dispatch_event_handler(&context, event_handler, &full_event, extra_event.as_ref())
            );

            #[cfg(not(feature = "framework"))]
            dispatch_event_handler(&context, event_handler, &full_event, extra_event.as_ref())
                .await;
        });
    }
}

#[cfg(feature = "framework")]
async fn dispatch_framework(
    context: &Context,
    framework: Option<Arc<dyn Framework>>,
    full_event: &FullEvent,
    extra_event: Option<&FullEvent>,
) {
    if let Some(framework) = framework {
        if let Some(extra_event) = extra_event {
            framework.dispatch(context, extra_event).await;
        }

        framework.dispatch(context, full_event).await;
    }
}

async fn dispatch_event_handler(
    context: &Context,
    event_handler: Option<Arc<dyn EventHandler>>,
    full_event: &FullEvent,
    extra_event: Option<&FullEvent>,
) {
    if let Some(handler) = event_handler {
        if let Some(extra_event) = extra_event {
            handler.dispatch(context, extra_event).await;
        }

        handler.dispatch(context, full_event).await;
    }
}
