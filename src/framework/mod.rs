//! The framework is a customizable method of separating commands.
//!
//! This is used in combination with [`ClientBuilder::framework`].
//!
//! [`ClientBuilder::framework`]: crate::gateway::client::ClientBuilder::framework

use async_trait::async_trait;

use crate::gateway::client::{Client, Context, FullEvent};

/// A trait for defining your own framework for serenity to use.
///
/// Should you implement this trait, or define a `message` handler, depends on you. However, using
/// this will benefit you by abstracting the [`EventHandler`] away.
///
/// [`EventHandler`]: crate::gateway::client::EventHandler
#[async_trait]
pub trait Framework: Send + Sync {
    /// Called directly after the `Client` is created.
    async fn init(&mut self, client: &Client) {
        let _: &Client = client;
    }
    /// Called on every incoming event.
    async fn dispatch(&self, ctx: &Context, event: &FullEvent);
}

#[async_trait]
impl<F> Framework for Box<F>
where
    F: Framework + ?Sized,
{
    async fn init(&mut self, client: &Client) {
        (**self).init(client).await;
    }
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        (**self).dispatch(ctx, event).await;
    }
}

#[async_trait]
impl<F> Framework for &mut F
where
    F: Framework + ?Sized,
{
    async fn init(&mut self, client: &Client) {
        (**self).init(client).await;
    }
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        (**self).dispatch(ctx, event).await;
    }
}
