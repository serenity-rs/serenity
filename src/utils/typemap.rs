//! A hashmap whose keys are defined by types.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::hash_map::{
    Entry as HashMapEntry,
    OccupiedEntry as HashMapOccupiedEntry,
    VacantEntry as HashMapVacantEntry,
};
use std::marker::PhantomData;

/// TypeMapKey is used to declare key types that are eligible for use
/// with [`TypeMap`].
///
/// [`TypeMap`]: struct.TypeMap.html
pub trait TypeMapKey: Any {
    /// Defines the value type that corresponds to this `TypeMapKey`.
    type Value: Send + Sync;
}

/// TypeMap is a simple abstraction around the standard library's [`HashMap`]
/// type, where types are its keys. This allows for statically-checked value
/// retrieval.
///
/// [`HashMap`]: std::collections::HashMap
pub struct TypeMap(HashMap<TypeId, Box<(dyn Any + Send + Sync)>>);

impl TypeMap {
    /// Creates a new instance of `TypeMap`.
    #[inline]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a new value based on its [`TypeMapKey`].
    /// If the value has been already inserted, it will be overwritten
    /// with the new value.
    ///
    /// ```rust
    /// use serenity::utils::{TypeMap, TypeMapKey};
    ///
    /// struct Number;
    ///
    /// impl TypeMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = TypeMap::new();
    /// map.insert::<Number>(42);
    /// // Overwrite the value of `Number` with -42.
    /// map.insert::<Number>(-42);
    /// ```
    ///
    /// [`TypeMapKey`]: trait.TypeMapKey.html
    #[inline]
    pub fn insert<T>(&mut self, value: T::Value)
    where
        T: TypeMapKey
    {
        self.0.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Retrieve the entry based on its [`TypeMapKey`]
    ///
    /// [`TypeMapKey`]: trait.TypeMapKey.html
    #[inline]
    pub fn entry<T>(&mut self) -> Entry<'_, T>
    where
        T: TypeMapKey
    {
        match self.0.entry(TypeId::of::<T>()) {
            HashMapEntry::Occupied(entry) => Entry::Occupied(OccupiedEntry {
                entry,
                _marker: PhantomData,
            }),
            HashMapEntry::Vacant(entry) => Entry::Vacant(VacantEntry {
                entry,
                _marker: PhantomData,
            })
        }
    }

    /// Retrieve a reference to a value based on its [`TypeMapKey`].
    /// Returns `None` if it couldn't be found.
    ///
    /// ```rust
    /// use serenity::utils::{TypeMap, TypeMapKey};
    ///
    /// struct Number;
    ///
    /// impl TypeMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = TypeMap::new();
    /// map.insert::<Number>(42);
    ///
    /// assert_eq!(*map.get::<Number>().unwrap(), 42);
    /// ```
    ///
    /// [`TypeMapKey`]: trait.TypeMapKey.html
    #[inline]
    pub fn get<T>(&self) -> Option<&T::Value>
    where
        T: TypeMapKey
    {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T::Value>())
    }

    /// Retrieve a mutable reference to a value based on its [`TypeMapKey`].
    /// Returns `None` if it couldn't be found.
    ///
    /// ```rust
    /// use serenity::utils::{TypeMap, TypeMapKey};
    ///
    /// struct Number;
    ///
    /// impl TypeMapKey for Number {
    ///     type Value = i32;
    /// }
    ///
    /// let mut map = TypeMap::new();
    /// map.insert::<Number>(42);
    ///
    /// assert_eq!(*map.get::<Number>().unwrap(), 42);
    /// *map.get_mut::<Number>().unwrap() -= 42;
    /// assert_eq!(*map.get::<Number>().unwrap(), 0);
    /// ```
    ///
    /// [`TypeMapKey`]: trait.TypeMapKey.html
    #[inline]
    pub fn get_mut<T>(&mut self) -> Option<&mut T::Value>
    where
        T: TypeMapKey
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|b| b.downcast_mut::<T::Value>())
    }
}

/// A view into a single entry in the [`TypeMap`],
/// which may either be vacant or occupied.
///
/// This heavily mirrors the official [`Entry`] API in the standard library,
/// but not all of it is provided due to implementation restrictions. Please
/// refer to its documentations.
///
/// [`TypeMap`]: struct.TypeMap.html
/// [`Entry`]: std::collections::hash_map::Entry
pub enum Entry<'a, K>
where
    K: TypeMapKey,
{
    Occupied(OccupiedEntry<'a, K>),
    Vacant(VacantEntry<'a, K>),
}

impl<'a, K> Entry<'a, K>
where
    K: TypeMapKey,
{
    #[inline]
    pub fn or_insert(self, value: K::Value) -> &'a mut K::Value {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(value),
        }
    }

    #[inline]
    pub fn or_insert_with<F>(self, f: F) -> &'a mut K::Value
    where
        F: FnOnce() -> K::Value
    {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(f()),
        }
    }

    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut K::Value)
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            },
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K> Entry<'a, K>
where
    K: TypeMapKey,
    K::Value: Default
{
    #[inline]
    pub fn or_default(self) -> &'a mut K::Value {
        self.or_insert_with(<K::Value as Default>::default)
    }
}

pub struct OccupiedEntry<'a, K>
where
    K: TypeMapKey,
{
    entry: HashMapOccupiedEntry<'a, TypeId, Box<(dyn Any + Send + Sync)>>,
    _marker: PhantomData<&'a K::Value>,
}

impl<'a, K> OccupiedEntry<'a, K>
where
    K: TypeMapKey,
{
    #[inline]
    pub fn get(&self) -> &K::Value {
        self.entry.get().downcast_ref().unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut K::Value {
        self.entry.get_mut().downcast_mut().unwrap()
    }

    #[inline]
    pub fn into_mut(self) -> &'a mut K::Value {
        self.entry.into_mut().downcast_mut().unwrap()
    }

    #[inline]
    pub fn insert(&mut self, value: K::Value) {
        self.entry.insert(Box::new(value));
    }

    #[inline]
    pub fn remove(self) {
        self.entry.remove();
    }
}

pub struct VacantEntry<'a, K>
where
    K: TypeMapKey,
{
    entry: HashMapVacantEntry<'a, TypeId, Box<(dyn Any + Send + Sync)>>,
    _marker: PhantomData<&'a K::Value>,
}

impl<'a, K> VacantEntry<'a, K>
where
    K: TypeMapKey,
{
    #[inline]
    pub fn insert(self, value: K::Value) -> &'a mut K::Value {
        self.entry.insert(Box::new(value)).downcast_mut().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Counter;

    impl TypeMapKey for Counter {
        type Value = u64;
    }

    #[test]
    fn typemap_counter() {
        let mut map = TypeMap::new();

        map.insert::<Counter>(0);

        assert_eq!(*map.get::<Counter>().unwrap(), 0);

        for _ in 0..100 {
            *map.get_mut::<Counter>().unwrap() += 1;
        }

        assert_eq!(*map.get::<Counter>().unwrap(), 100);
    }

    #[test]
    fn typemap_entry() {
        let mut map = TypeMap::new();

        assert_eq!(map.get::<Counter>(), None);
        *map.entry::<Counter>().or_insert(0) += 42;
        assert_eq!(*map.get::<Counter>().unwrap(), 42);
    }
}
