use std::sync::{Arc, RwLock};

pub trait RwLockExt<T> {
    fn with<Y, F: Fn(&T) -> Y>(&self, f: F) -> Y;
    fn with_mut<Y, F: FnMut(&mut T) -> Y>(&self, f: F) -> Y;
}

impl<T> RwLockExt<T> for Arc<RwLock<T>> {
    fn with<Y, F: Fn(&T) -> Y>(&self, f: F) -> Y {
        let r = self.read().unwrap();
        f(&r)
    }

    fn with_mut<Y, F: FnMut(&mut T) -> Y>(&self, mut f: F) -> Y {
        let mut w = self.write().unwrap();
        f(&mut w)
    }
}
