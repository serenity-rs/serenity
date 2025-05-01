//! Wrappers around library types for easier use.

use std::hash::Hash;
#[cfg(feature = "temp_cache")]
use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::multiple::RefMulti;
use dashmap::mapref::one::{Ref, RefMut};
#[cfg(feature = "typesize")]
use typesize::TypeSize;

#[derive(Debug)]
/// A wrapper around Option<DashMap<K, V>> to ease disabling specific cache fields.
pub(crate) struct MaybeMap<K: Eq + Hash, V>(pub(crate) Option<DashMap<K, V, BuildHasher>>);
impl<K: Eq + Hash, V> MaybeMap<K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V>> {
        Option::iter(&self.0).flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V>> {
        self.0.as_ref()?.get(k)
    }

    pub fn get_mut(&self, k: &K) -> Option<RefMut<'_, K, V>> {
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
        self.0.as_ref().map_or(0, DashMap::len)
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

#[cfg(feature = "typesize")]
impl<K: Eq + Hash + TypeSize, V: TypeSize> TypeSize for MaybeMap<K, V> {
    fn extra_size(&self) -> usize {
        self.0.as_ref().map(DashMap::extra_size).unwrap_or_default()
    }

    typesize::if_typesize_details! {
        fn get_collection_item_count(&self) -> Option<usize> {
            self.0.as_ref().and_then(DashMap::get_collection_item_count)
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// A wrapper around a reference to a MaybeMap, allowing for public inspection of the underlying
/// map without allowing mutation of internal cache fields, which could cause issues.
pub struct ReadOnlyMapRef<'a, K: Eq + Hash, V>(Option<&'a DashMap<K, V, BuildHasher>>);
impl<K: Eq + Hash, V> ReadOnlyMapRef<'_, K, V> {
    pub fn iter(&self) -> impl Iterator<Item = RefMulti<'_, K, V>> {
        self.0.into_iter().flat_map(DashMap::iter)
    }

    pub fn get(&self, k: &K) -> Option<Ref<'_, K, V>> {
        self.0?.get(k)
    }

    pub fn len(&self) -> usize {
        self.0.map_or(0, DashMap::len)
    }
}

pub struct Hasher(foldhash::fast::FoldHasher);
impl std::hash::Hasher for Hasher {
    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.0.write_u8(i);
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.0.write_u16(i);
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.0.write_u32(i);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0.write_u64(i);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.0.write_u128(i);
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.0.write_usize(i);
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) {
        self.0.write_i8(i);
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) {
        self.0.write_i16(i);
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) {
        self.0.write_i32(i);
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) {
        self.0.write_i64(i);
    }

    #[inline(always)]
    fn write_i128(&mut self, i: i128) {
        self.0.write_i128(i);
    }

    #[inline(always)]
    fn write_isize(&mut self, i: isize) {
        self.0.write_isize(i);
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
    
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

#[cfg(feature = "typesize")]
impl typesize::TypeSize for Hasher {}

#[derive(Clone, Default)]
pub struct BuildHasher(foldhash::fast::RandomState);
impl std::hash::BuildHasher for BuildHasher {
    type Hasher = Hasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        Hasher(self.0.build_hasher())
    }
}

pub type ExtractMap<K, V> = extract_map::ExtractMap<K, V, BuildHasher>;

#[cfg(feature = "typesize")]
impl typesize::TypeSize for BuildHasher {}

/// Wrapper around `SizableArc<T, Owned>`` with support for disabling typesize.
///
/// This denotes an Arc where T's size should be considered when calling `TypeSize::get_size`
#[derive(Debug)]
#[cfg(feature = "temp_cache")]
pub(crate) struct MaybeOwnedArc<T>(
    #[cfg(feature = "typesize")] typesize::ptr::SizableArc<T, typesize::ptr::Owned>,
    #[cfg(not(feature = "typesize"))] Arc<T>,
);

#[cfg(feature = "temp_cache")]
impl<T> MaybeOwnedArc<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self(Arc::new(inner).into())
    }

    pub(crate) fn get_inner(self) -> Arc<T> {
        #[cfg(feature = "typesize")]
        let inner = self.0.0;
        #[cfg(not(feature = "typesize"))]
        let inner = self.0;

        inner
    }
}

#[cfg(all(feature = "typesize", feature = "temp_cache"))]
impl<T: typesize::TypeSize> typesize::TypeSize for MaybeOwnedArc<T> {
    fn extra_size(&self) -> usize {
        self.0.extra_size()
    }

    fn get_collection_item_count(&self) -> Option<usize> {
        self.0.get_collection_item_count()
    }
}

#[cfg(feature = "temp_cache")]
impl<T> std::ops::Deref for MaybeOwnedArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "temp_cache")]
impl<T> Clone for MaybeOwnedArc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone().into())
    }
}
