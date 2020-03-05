use tokio::sync::RwLock;
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait RwLockExt<T: Send + Sync> {
    async fn with<Y, F: Fn(&T) -> Y + Send>(&self, f: F) -> Y;
    async fn with_mut<Y, F: FnMut(&mut T) -> Y + Send>(&self, f: F) -> Y;
}

#[async_trait]
impl<T: Send + Sync> RwLockExt<T> for Arc<RwLock<T>> {
    async fn with<Y, F: Fn(&T) -> Y + Send>(&self, f: F) -> Y {
        let r = self.read().await;
        f(&r)
    }

    async fn with_mut<Y, F: FnMut(&mut T) -> Y + Send>(&self, mut f: F) -> Y {
        let mut w = self.write().await;
        f(&mut w)
    }
}
