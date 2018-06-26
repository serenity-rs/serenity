use owning_ref::OwningHandle;
use parking_lot::{Mutex, MutexGuard};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

type LongLockInner<T> = OwningHandle<Arc<Mutex<T>>, MutexGuard<'static, T>>;
pub struct LongLock<T: 'static> {
    inner: LongLockInner<T>,
}

unsafe impl<T> Send for LongLock<T> {

}

impl<T> LongLock<T>{
    pub fn new(resource: Arc<Mutex<T>>) -> LongLock<T> {
    	let inner = OwningHandle::new_with_fn(resource, |resource_ptr| 
            unsafe {
                resource_ptr.as_ref()
                    .expect("Guaranteed to be a real object.")
                    .lock()
            }
        );

        LongLock {inner}
    }
}

impl<T> Drop for LongLock<T> {
    fn drop(&mut self) {
        println!("killing the LongLock");
    }
}

impl<T> Deref for LongLock<T> {
    type Target = LongLockInner<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for LongLock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
