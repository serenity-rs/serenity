#[cfg(feature = "http")]
pub fn spawn_named<F, T>(_name: &str, future: F) -> tokio::task::JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    #[cfg(all(tokio_unstable, feature = "tokio_task_builder"))]
    let handle = tokio::task::Builder::new()
        .name(&*format!("serenity::{}", _name))
        .spawn(future)
        .expect("called outside tokio runtime");
    #[cfg(not(all(tokio_unstable, feature = "tokio_task_builder")))]
    let handle = tokio::spawn(future);
    handle
}
