use async_trait::async_trait;

use super::Cache;

/// Trait used for updating the cache with a type.
///
/// This may be implemented on a type and used to update the cache via
/// [`Cache::update`].
///
/// **Info**:
/// You may not access the fields of the cache, as they are public for the
/// crate only.
///
/// # Examples
///
/// Creating a custom struct implementation to update the cache with:
///
/// ```rust,ignore
/// use serenity::{
///     json::json,    
///     cache::{Cache, CacheUpdate},
///     model::{
///         id::UserId,
///         user::User,
///     },
///     prelude::RwLock,
/// };
/// use std::{
///     collections::hash_map::Entry,
///     sync::Arc,
/// };
///
/// // For example, an update to the user's record in the database was
/// // published to a pubsub channel.
/// struct DatabaseUserUpdate {
///     user_avatar: Option<String>,
///     user_discriminator: u16,
///     user_id: UserId,
///     user_is_bot: bool,
///     user_name: String,
/// }
///
/// #[serenity::async_trait]
/// impl CacheUpdate for DatabaseUserUpdate {
///     // A copy of the old user's data, if it existed in the cache.
///     type Output = User;
///
///     async fn update(&mut self, cache: &Cache) -> Option<Self::Output> {
///         // If an entry for the user already exists, update its fields.
///         match cache.users.entry(self.user_id) {
///             Entry::Occupied(entry) => {
///                 let user = entry.get();
///                 let old_user = user.clone();
///
///                 user.bot = self.user_is_bot;
///                 user.discriminator = self.user_discriminator;
///                 user.id = self.user_id;
///
///                 if user.avatar != self.user_avatar {
///                     user.avatar = self.user_avatar.clone();
///                 }
///
///                 if user.name != self.user_name {
///                     user.name = self.user_name.clone();
///                 }
///
///                 // Return the old copy for the user's sake.
///                 Some(old_user)
///             },
///             Entry::Vacant(entry) => {
///                 // We can convert a `serde_json::Value` to a User for test
///                 // purposes.
///                 let user = serde_json::from_value::<User>(json!({
///                     "id": self.user_id,
///                     "avatar": self.user_avatar.clone(),
///                     "bot": self.user_is_bot,
///                     "discriminator": self.user_discriminator,
///                     "username": self.user_name.clone(),
///                 })).expect("Error making user");
///
///                 entry.insert(user);
///
///                 // There was no old copy, so return None.
///                 None
///             },
///         }
///     }
/// }
///
/// # async fn run() {
/// // Create an instance of the cache.
/// let mut cache = Cache::new();
///
/// // This is a sample pubsub message that you might receive from your
/// // database.
/// let mut update_message = DatabaseUserUpdate {
///     user_avatar: None,
///     user_discriminator: 6082,
///     user_id: UserId(379740138303127564),
///     user_is_bot: true,
///     user_name: "TofuBot".to_owned(),
/// };
///
/// // Update the cache with the message.
/// cache.update(&mut update_message).await;
/// # }
/// ```
#[async_trait]
pub trait CacheUpdate {
    /// The return type of an update.
    ///
    /// If there is nothing to return, specify this type as an unit (`()`).
    type Output;

    /// Updates the cache with the implementation.
    async fn update(&mut self, _: &Cache) -> Option<Self::Output>;
}
