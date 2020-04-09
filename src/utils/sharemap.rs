//! A hashmap whose keys are defined by types.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// ShareMapKey is used to declare key types that are eligible for use
/// with [`ShareMap`].
///
/// [`ShareMap`]: struct.ShareMap.html
pub trait ShareMapKey: Any {
    /// Defines the value type that corresponds to this `ShareMapKey`.
    type Value: Send + Sync;
}

/// ShareMap is a simple abstraction around the standard library's [`HashMap`]
/// type, where types are its keys. This allows for statically-checked value
/// retrieval.
///
/// [`HashMap`]: std::collections::HashMap
pub struct ShareMap(HashMap<TypeId, Box<(dyn Any + Send + Sync)>>);

impl ShareMap {
    /// Creates a new instance of `ShareMap`.
    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a new value based on its [`ShareMapKey`].
    /// If the value has been already inserted, it will be overwritten
    /// with the new value.
    ///
    /// ```rust
    /// use serenity::utils::{ShareMap, ShareMapKey};
    ///
    /// struct Number;
    ///
    /// impl ShareMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = ShareMap::new();
    /// map.insert::<Number>(42);
    /// // Overwrite the value of `Number` with -42.
    /// map.insert::<Number>(-42);
    /// ```
    ///
    /// [`ShareMapKey`]: trait.ShareMapKey.html
    #[inline]
    pub fn insert<T>(&mut self, value: T::Value)
    where
        T: ShareMapKey
    {
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Retrieve a reference to a value based on its [`ShareMapKey`].
    /// Returns `None` if it couldn't be found.
    ///
    /// ```rust
    /// use serenity::utils::{ShareMap, ShareMapKey};
    ///
    /// struct Number;
    ///
    /// impl ShareMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = ShareMap::new();
    /// map.insert::<Number>(42);
    ///
    /// assert_eq!(*map.get::<Number>().unwrap(), 42);
    /// ```
    ///
    /// [`ShareMapKey`]: trait.ShareMapKey.html
    #[inline]
    pub fn get<T>(&self) -> Option<&T::Value>
    where
        T: ShareMapKey
    {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T::Value>())
    }

    /// Retrieve a mutable reference to a value based on its [`ShareMapKey`].
    /// Returns `None` if it couldn't be found.
    ///
    /// ```rust
    /// use serenity::utils::{ShareMap, ShareMapKey};
    ///
    /// struct Number;
    ///
    /// impl ShareMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = ShareMap::new();
    /// map.insert::<Number>(42);
    ///
    /// assert_eq!(*map.get::<Number>().unwrap(), 42);
    /// *map.get_mut::<Number>().unwrap() -= 42;
    /// assert_eq!(*map.get::<Number>().unwrap(), 0);
    /// ```
    ///
    /// [`ShareMapKey`]: trait.ShareMapKey.html
    #[inline]
    pub fn get_mut<T>(&mut self) -> Option<&mut T::Value>
    where
        T: ShareMapKey
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T::Value>())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Counter;

    impl ShareMapKey for Counter {
        type Value = u64;
    }

    #[test]
    fn sharemap_counter() {
        let mut map = ShareMap::new();

        map.insert::<Counter>(0);

        assert_eq!(*map.get::<Counter>().unwrap(), 0);

        for _ in 0..100 {
            *map.get_mut::<Counter>().unwrap() += 1;
        }

        assert_eq!(*map.get::<Counter>().unwrap(), 100);
    }
}
