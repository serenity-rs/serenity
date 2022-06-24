use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as FutContext, Poll};

use futures::Future;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;
use tokio::time::Sleep;

pub struct Collector<Item> {
    pub(super) receiver: Pin<Box<Receiver<Arc<Item>>>>,
    pub(super) timeout: Option<Pin<Box<Sleep>>>,
}

impl<Item> Collector<Item> {
    /// Stops collecting, this will implicitly be done once the
    /// collector drops.
    /// In case the drop does not appear until later, it is preferred to
    /// stop the collector early.
    pub fn stop(self) {}
}

impl<Item> futures::stream::Stream for Collector<Item> {
    type Item = Arc<Item>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut FutContext<'_>) -> Poll<Option<Self::Item>> {
        if let Some(timeout) = &mut self.timeout {
            match timeout.as_mut().poll(ctx) {
                Poll::Ready(_) => {
                    return Poll::Ready(None);
                },
                Poll::Pending => (),
            }
        }

        self.receiver.as_mut().poll_recv(ctx)
    }
}
