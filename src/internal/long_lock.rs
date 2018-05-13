use owning_ref::OwningHandle;
use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

pub type LongLock<T> = OwningHandle<Arc<Mutex<T>>, MutexGuard<'static, T>>;

pub fn long_lock<T>(resource: Arc<Mutex<T>>) -> LongLock<T> {
	OwningHandle::new_with_fn(resource, |resource_ptr| 
        unsafe {
            resource_ptr.as_ref()
                .expect("Guaranteed to be a real object.")
                .lock()
        }
    )
}