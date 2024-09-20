use super::Cache;

/// Trait used for updating the cache with a type.
///
/// This may be implemented on a type and used to update the cache via [`Cache::update`].
///
/// **Info**: You may not access the fields of the cache, as they are public for the crate only.
pub trait CacheUpdate {
    /// The return type of an update.
    ///
    /// If there is nothing to return, specify this type to be the unit `()`.
    type Output;

    /// Updates the cache with the implementation.
    fn update(&mut self, _: &Cache) -> Option<Self::Output>;
}
