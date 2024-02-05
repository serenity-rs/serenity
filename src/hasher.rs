pub struct Hasher(rustc_hash::FxHasher);
impl std::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
}

#[derive(Clone, Default)]
pub struct BuildHasher();
impl std::hash::BuildHasher for BuildHasher {
    type Hasher = Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        Hasher(rustc_hash::FxHasher::default())
    }
}

#[allow(clippy::disallowed_types)]
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasher>;
