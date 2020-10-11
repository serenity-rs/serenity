//! Compatability and convenience methods for working with [serenity].
//! Requires the `"serenity-rustls"` or `"serenity-native"` features.
//!
//! [serenity]: https://crates.io/crates/serenity/0.9.0-rc.2

use crate::manager::Songbird;
#[cfg(feature = "driver")]
use crate::tracks::TrackQueue;
use serenity::{
    client::{ClientBuilder, Context},
    prelude::TypeMapKey,
};
use std::sync::Arc;

/// Key type used to store and retrieve access to the manager from the serenity client's
/// shared key-value store.
pub struct SongbirdKey;

impl TypeMapKey for SongbirdKey {
    type Value = Arc<Songbird>;
}

/// Installs a new songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register(client_builder: ClientBuilder) -> ClientBuilder {
    let voice = Songbird::serenity();
    register_with(client_builder, voice)
}

/// Installs a given songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register_with(client_builder: ClientBuilder, voice: Arc<Songbird>) -> ClientBuilder {
    client_builder
        .voice_manager_arc(voice.clone())
        .type_map_insert::<SongbirdKey>(voice)
}

/// Retrieve the Songbird voice client from a serenity context's
/// shared key-value store.
pub async fn get(ctx: &Context) -> Option<Arc<Songbird>> {
    let data = ctx.data.read().await;

    data.get::<SongbirdKey>().cloned()
}

#[cfg(feature = "driver")]
pub struct SongbirdQueueKey;

#[cfg(feature = "driver")]
impl TypeMapKey for SongbirdQueueKey {
    type Value = Arc<TrackQueue>;
}

/// Helper trait to add installation/creation methods to serenity's
/// `ClientBuilder`.
///
/// These install the client to receive gateway voice events, and
/// store an easily accessible reference to songbir'd managers.
pub trait SerenityInit {
    fn register_songbird(self) -> Self;

    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self;

    #[cfg(feature = "driver")]
    fn register_trackqueue(self) -> Self;

    #[cfg(feature = "driver")]
    fn register_trackqueue_with(self, queue: TrackQueue) -> Self;
}

impl SerenityInit for ClientBuilder<'_> {
    fn register_songbird(self) -> Self {
        register(self)
    }

    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self {
        register_with(self, voice)
    }

    #[cfg(feature = "driver")]
    fn register_trackqueue(self) -> Self {
        unimplemented!()
    }

    #[cfg(feature = "driver")]
    fn register_trackqueue_with(self, queue: TrackQueue) -> Self {
        unimplemented!()
    }
}
