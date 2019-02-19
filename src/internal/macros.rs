//! A set of macros for easily working with internals.

#[cfg(any(feature = "model", feature = "utils"))]
macro_rules! cdn {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(cdn!($e), $($rest)*)
    };
}

#[cfg(feature = "http")]
macro_rules! api {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

#[cfg(feature = "http")]
macro_rules! status {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}

// Enable/disable check for cache
#[cfg(any(all(feature = "cache", any(feature = "client", feature = "model"))))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(any(all(not(feature = "cache"), any(feature = "client", feature = "model"))))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

macro_rules! enum_number {
    ($name:ident { $($variant:ident, )* }) => {
        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter<'_>)
                        -> ::std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> ::std::result::Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( v if v == $name::$variant as u64 => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}
