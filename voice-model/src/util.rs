pub(crate) mod json_safe_u64 {
    use core::fmt::{Formatter, Result as FmtResult};

    use serde::de::{Deserializer, Error, Visitor};
    use serde::ser::Serializer;

    struct U64Visitor;

    impl<'de> Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
            formatter.write_str("a u64 represented by a string or number")
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse::<u64>().map_err(E::custom)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(U64Visitor)
    }

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(value)
    }
}
