// Most of this is
// shamelessly copied from https://github.com/hyperium/hyper/blob/master/src/header/internals/vec_map.rs

/// Like `HashMap` but solely uses a vector instead.
///
/// note: This is for internal use.
#[derive(Clone, Debug, Default)]
pub struct VecMap<K, V>(Vec<(K, V)>);

impl<K: PartialEq, V> VecMap<K, V> {
    pub fn new() -> Self {
        VecMap(Vec::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        VecMap(Vec::with_capacity(cap))
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.0.push((key, value));
    }

    pub fn remove<Q: ?Sized + PartialEq<K>>(&mut self, key: &Q) -> Option<V> {
        self.pos(key).map(|pos| self.0.remove(pos)).map(|entry| entry.1)
    }

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        match self.pos(&key) {
            Some(pos) => Entry::Occupied(OccupiedEntry {
                vec: &mut self.0,
                pos,
            }),
            None => Entry::Vacant(VacantEntry {
                vec: &mut self.0,
                key,
            })
        }
    }

    pub fn get<Q: PartialEq<K> + ?Sized>(&self, key: &Q) -> Option<&V> {
        self.iter().find(|entry| key == &entry.0).map(|entry| &entry.1)
    }

    #[inline]
    pub fn iter(&self) -> ::std::slice::Iter<(K, V)> {
        self.into_iter()
    }

    fn pos<Q: PartialEq<K> + ?Sized>(&self, key: &Q) -> Option<usize> {
        self.iter().position(|entry| key == &entry.0)
    }
}

impl<K, V> IntoIterator for VecMap<K, V> {
    type Item = (K, V);
    type IntoIter = ::std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = &'a (K, V);
    type IntoIter = ::std::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>)
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn or_insert(self, val: V) -> &'a mut V {
        use self::Entry::*;

        match self {
            Vacant(entry) => entry.insert(val),
            Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, val: F) -> &'a mut V {
        use self::Entry::*;

        match self {
            Vacant(entry) => entry.insert(val()),
            Occupied(entry) => entry.into_mut(),
        }
    }
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    vec: &'a mut Vec<(K, V)>,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn insert(self, val: V) -> &'a mut V {
        self.vec.push((self.key, val));
        let pos = self.vec.len() - 1;
        &mut self.vec[pos].1
    }
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    vec: &'a mut Vec<(K, V)>,
    pos: usize,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn into_mut(self) -> &'a mut V {
        &mut self.vec[self.pos].1
    }
}
