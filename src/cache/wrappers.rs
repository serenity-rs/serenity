//! Wrappers around library types for easier use.

use std::hash::Hash;

use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;
use fxhash::FxBuildHasher;

#[derive(Debug)]
/// A wrapper around Option<DashMap<K, V>> to ease disabling specific cache fields.
pub(crate) struct MaybeMap<K: Eq + Hash, V>(pub(super) Option<DashMap<K, V, FxBuildHasher>>);
impl<K: Eq + Hash, V> MaybeMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V, FxBuildHasher>> {
        Option::iter(&self.0).flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V, FxBuildHasher>> {
        self.0.as_ref()?.get(k)
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<'_, K, V, FxBuildHasher>> {
        self.0.as_ref()?.get_mut(k)
    }

    pub fn insert(&self, k: K, v: V) -> Option<V> {
        self.0.as_ref()?.insert(k, v)
    }

    pub fn remove(&self, k: &K) -> Option<(K, V)> {
        self.0.as_ref()?.remove(k)
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |map| map.len())
    }

    pub(crate) fn as_read_only(&self) -> ReadOnlyMapRef<'_, K, V> {
        ReadOnlyMapRef(self.0.as_ref())
    }
}

#[derive(Debug)]
/// A wrapper around a reference to a MaybeMap, allowing for public inspection of the underlying map
/// without allowing mutation of internal cache fields, which could cause issues.
pub struct ReadOnlyMapRef<'a, K: Eq + Hash, V>(Option<&'a DashMap<K, V, FxBuildHasher>>);
impl<'a, K: Eq + Hash, V> ReadOnlyMapRef<'a, K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V, FxBuildHasher>> {
        self.0.into_iter().flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V, FxBuildHasher>> {
        self.0?.get(k)
    }

    pub fn len(&self) -> usize {
        self.0.map_or(0, DashMap::len)
    }
}
