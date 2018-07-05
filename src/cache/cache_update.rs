use super::Cache;

/// Trait used for updating the cache with a type.
///
/// This may be implemented on a type and used to update the cache via
/// [`Cache::update`].
///
/// # Examples
///
/// Creating a custom struct implementation to update the cache with:
///
/// ```rust
/// use serenity::{
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
/// impl CacheUpdate for DatabaseUserUpdate {
///     // A copy of the old user's data, if it existed in the cache.
///     type Output = User;
///
///     fn update(&mut self, cache: &mut Cache) -> Option<Self::Output> {
///         // If an entry for the user already exists, update its fields.
///         match cache.users.entry(self.user_id) {
///             Entry::Occupied(entry) => {
///                 let user = entry.get();
///                 let mut writer = user.write();
///                 let old = writer.clone();
///
///                 writer.bot = self.user_is_bot;
///                 writer.discriminator = self.user_discriminator;
///                 writer.id = self.user_id;
///
///                 if writer.avatar != self.user_avatar {
///                     writer.avatar = self.user_avatar.clone();
///                 }
///
///                 if writer.name != self.user_name {
///                     writer.name = self.user_name.clone();
///                 }
///
///                 // Return the old copy for the user's sake.
///                 Some(old)
///             },
///             Entry::Vacant(entry) => {
///                 entry.insert(Arc::new(RwLock::new(User {
///                     id: self.user_id,
///                     avatar: self.user_avatar.clone(),
///                     bot: self.user_is_bot,
///                     discriminator: self.user_discriminator,
///                     name: self.user_name.clone(),
///                 })));
///
///                 // There was no old copy, so return None.
///                 None
///             },
///         }
///     }
/// }
///
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
/// cache.update(&mut update_message);
/// ```
///
/// [`Cache::update`]: struct.Cache.html#method.update
pub trait CacheUpdate {
    /// The return type of an update.
    ///
    /// If there is nothing to return, specify this type as an unit (`()`).
    type Output;

    /// Updates the cache with the implementation.
    fn update(&mut self, &mut Cache) -> Option<Self::Output>;
}
