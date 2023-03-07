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
        concat!("https://discord.com/api/v10", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

#[cfg(feature = "http")]
macro_rules! status {
    ($e:expr) => {
        concat!("https://status.discord.com/api/v2", $e)
    };
}

// Enable check for cache
#[cfg(all(feature = "cache", feature = "client"))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {{
        $enabled
    }};
}

#[cfg(all(not(feature = "cache"), feature = "client"))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {{
        $disabled
    }};
}

macro_rules! enum_number {
    ($name:ident { $($(#[$attr:meta])? $variant:ident $(,)? )* }) => {
        impl $name {
            #[inline]
            #[must_use]
            pub fn num(&self) -> u64 {
                *self as u64
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where S: serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where D: serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>)
                        -> std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> std::result::Result<$name, E>
                        where E: serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $(#[$attr])? v if v == $name::$variant as u64 => Ok($name::$variant), )*
                            _ => {
                                tracing::warn!("Unknown {} value: {}", stringify!($name), value);

                                Ok($name::Unknown)
                            }
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

/// The macro forwards the generation to the `bitflags::bitflags!` macro and implements
/// the default (de)serialization for Discord's bitmask values.
///
/// The flags are created with `T::from_bits_truncate` for the deserialized integer value.
///
/// Use the `bitflags::bitflags! macro directly if a different serde implementation is required.
macro_rules! bitflags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        bitflags::bitflags! {
            $(#[$outer])*
            $vis struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }
        }

        bitflags!(__impl_serde $BitFlags: $T);

        bitflags! {
            $($t)*
        }
    };
    (__impl_serde $BitFlags:ident: $T:tt) => {
        impl<'de> serde::de::Deserialize<'de> for $BitFlags {
            fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                Ok(Self::from_bits_truncate(<$T>::deserialize(deserializer)?))
            }
        }

        impl serde::ser::Serialize for $BitFlags {
            fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
                self.bits().serialize(serializer)
            }
        }
    };
    () => {};
}
