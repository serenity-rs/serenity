//! A fast hasher based on foldhash, specialized for integer keys, used for maps across the library.

pub struct Hasher<'a>(foldhash::fast::FoldHasher<'a>);
impl std::hash::Hasher for Hasher<'_> {
    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0.write_u8(i);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0.write_u16(i);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0.write_u32(i);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0.write_u64(i);
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.0.write_u128(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.0.write_usize(i);
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.0.write_i8(i);
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.0.write_i16(i);
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0.write_i32(i);
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.0.write_i64(i);
    }

    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.0.write_i128(i);
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.0.write_isize(i);
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
}

#[cfg(feature = "typesize")]
impl typesize::TypeSize for Hasher<'_> {}

#[derive(Clone, Default)]
pub struct BuildHasher(foldhash::fast::RandomState);
impl std::hash::BuildHasher for BuildHasher {
    type Hasher = Hasher<'static>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Hasher(self.0.build_hasher())
    }
}

pub type ExtractMap<K, V> = extract_map::ExtractMap<K, V, BuildHasher>;

#[cfg(feature = "typesize")]
impl typesize::TypeSize for BuildHasher {}
