use crate::manager::ClientVoiceManager;
use serenity::{
	client::{ClientBuilder, Context},
	prelude::{TypeMap, TypeMapKey},
};
use std::sync::Arc;

/// Key type used to store and retrieve access to the manager from the serenity client's
/// shared key-value store.
pub struct Songbird {}

impl TypeMapKey for Songbird {
    type Value = Arc<ClientVoiceManager>;
}

/// Installs a new songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register(client_builder: ClientBuilder) -> ClientBuilder {
	let voice = ClientVoiceManager::new(0, 0.into());
	register_with(client_builder, voice)
}

/// Installs a given songbird instance into the serenity client.
///
/// This should be called after any uses of `ClientBuilder::type_map`.
pub fn register_with(client_builder: ClientBuilder, voice: ClientVoiceManager) -> ClientBuilder {
	let voice = Arc::new(voice);

	client_builder.voice_manager_arc(voice.clone())
		.type_map_insert::<Songbird>(voice)
}

/// Retrieve the Songbird voice client from a serenity context's
/// shared key-value store.
pub async fn get(ctx: &Context) -> Option<Arc<ClientVoiceManager>> {
	let data = ctx.data.read().await;

	data.get::<Songbird>()
		.cloned()
}

/// Helper trait to add installation/creation methods to serenity's
/// `ClientBuilder`.
///
/// These install the client to receive gateway voice events, and
/// store an easily accessible reference to 
pub trait SerenityInit {
	fn register_songbird(self) -> Self;

	fn register_songbird_with(self, voice: ClientVoiceManager) -> Self;
}

impl SerenityInit for ClientBuilder<'_> {
	fn register_songbird(self) -> Self {
		register(self)
	}

	fn register_songbird_with(self, voice: ClientVoiceManager) -> Self {
		register_with(self, voice)
	}
}
