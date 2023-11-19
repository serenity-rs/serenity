//! Wrappers around library types for easier use.

use std::hash::Hash;

use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::DashMap;

#[derive(Debug)]
/// A wrapper around Option<DashMap<K, V>> to ease disabling specific cache fields.
pub(crate) struct MaybeMap<K: Eq + Hash, V>(pub(super) Option<DashMap<K, V, BuildHasher>>);
impl<K: Eq + Hash, V> MaybeMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V, BuildHasher>> {
        Option::iter(&self.0).flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V, BuildHasher>> {
        self.0.as_ref()?.get(k)
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<'_, K, V, BuildHasher>> {
        self.0.as_ref()?.get_mut(k)
    }

    pub fn contains(&self, k: &K) -> bool {
        self.0.as_ref().is_some_and(|m| m.contains_key(k))
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

    pub fn shrink_to_fit(&self) {
        if let Some(map) = self.0.as_ref() {
            map.shrink_to_fit();
        }
    }

    pub(crate) fn as_read_only(&self) -> ReadOnlyMapRef<'_, K, V> {
        ReadOnlyMapRef(self.0.as_ref())
    }
}

#[derive(Clone, Copy, Debug)]
/// A wrapper around a reference to a MaybeMap, allowing for public inspection of the underlying
/// map without allowing mutation of internal cache fields, which could cause issues.
pub struct ReadOnlyMapRef<'a, K: Eq + Hash, V>(Option<&'a DashMap<K, V, BuildHasher>>);
impl<'a, K: Eq + Hash, V> ReadOnlyMapRef<'a, K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V, BuildHasher>> {
        self.0.into_iter().flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V, BuildHasher>> {
        self.0?.get(k)
    }

    pub fn len(&self) -> usize {
        self.0.map_or(0, DashMap::len)
    }
}

pub struct Hasher(fxhash::FxHasher);
impl std::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
}
#[derive(Clone, Default)]
pub struct BuildHasher(fxhash::FxBuildHasher);
impl std::hash::BuildHasher for BuildHasher {
    type Hasher = Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        Hasher(self.0.build_hasher())
    }
}
