//! Compatability and convenience methods for working with [serenity].
//! Requires the `"serenity-rustls"` or `"serenity-native"` features.
//!
//! [serenity]: https://crates.io/crates/serenity/0.9.0-rc.2

use crate::manager::Songbird;
use serenity::{
    client::{ClientBuilder, Context},
    prelude::TypeMapKey,
};
use std::sync::Arc;

/// Zero-size type used to retrieve the registered [`Songbird`] instance
/// from serenity's inner TypeMap.
///
/// [`Songbird`]: ../struct.Songbird.html
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

/// Helper trait to add installation/creation methods to serenity's
/// `ClientBuilder`.
///
/// These install the client to receive gateway voice events, and
/// store an easily accessible reference to Songbird's managers.
pub trait SerenityInit {
    /// Registers a new Songbird voice system with serenity, storing it for easy
    /// access via [`get`].
    ///
    /// [`get`]: fn.get.html
    fn register_songbird(self) -> Self;
    /// Registers a given Songbird voice system with serenity, as above.
    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self;
}

impl SerenityInit for ClientBuilder<'_> {
    fn register_songbird(self) -> Self {
        register(self)
    }

    fn register_songbird_with(self, voice: Arc<Songbird>) -> Self {
        register_with(self, voice)
    }
}
