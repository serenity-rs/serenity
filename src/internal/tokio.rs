pub fn spawn_named<T>(
    name: &str,
    future: impl std::future::Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T>
where
    T: Send + 'static,
{
    #[cfg(tokio_unstable)]
    return tokio::task::Builder::new().name(&*format!("serenity::{}", name)).spawn(future);

    #[cfg(not(tokio_unstable))]
    tokio::spawn(future)
}
