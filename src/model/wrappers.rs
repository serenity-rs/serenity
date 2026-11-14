//! Collection type aliases used across the library.
//!
//! Maps and sets that the library fills itself (the cache, deserialized models, ...) use
//! [`BuildHasher`], a fast hasher specialized for integer keys. Collections that users construct
//! and hand to the library (e.g. localizations in builders) use [`UserProvidedHashMap`], which has
//! the standard library's default hasher so that `HashMap::from([...])` and `.collect()` keep
//! working.
//!
//! The standard library collection types are banned via `clippy::disallowed_types`, so that the
//! right alias gets picked deliberately. This module is the only place allowed to name them.

pub use crate::internal::hasher::BuildHasher;

/// A [`HashMap`](std::collections::HashMap) that is provided by the user, using the default
/// hasher.
#[expect(clippy::disallowed_types, reason = "this is the sanctioned alias for the std type")]
pub type UserProvidedHashMap<K, V> = std::collections::HashMap<K, V>;

/// A [`HashMap`](std::collections::HashMap) using the library's fast hasher.
#[expect(clippy::disallowed_types, reason = "this is the sanctioned alias for the std type")]
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasher>;

/// A [`HashSet`](std::collections::HashSet) using the library's fast hasher.
#[expect(clippy::disallowed_types, reason = "this is the sanctioned alias for the std type")]
pub type HashSet<T> = std::collections::HashSet<T, BuildHasher>;

/// An [`ExtractMap`](extract_map::ExtractMap) using the library's fast hasher.
pub type ExtractMap<K, V> = extract_map::ExtractMap<K, V, BuildHasher>;
