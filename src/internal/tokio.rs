use std::future::Future;

#[cfg(all(tokio_unstable, feature = "tokio_task_builder"))]
pub fn spawn_named<F, T>(name: &str, future: F) -> tokio::task::JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::Builder::new()
        .name(&*format!("serenity::{}", name))
        .spawn(future)
        .expect("called outside tokio runtime")
}

#[cfg(not(all(tokio_unstable, feature = "tokio_task_builder")))]
pub fn spawn_named<F, T>(_name: &str, future: F) -> tokio::task::JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(future)
}
