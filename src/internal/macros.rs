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

/// The `enum_number!` macro generates `From` implementations to convert between values and the
/// enum which can then be utilized by `serde` with `#[serde(from = "u8", into = "u8")]`.
///
/// When defining the enum like this:
/// ```ignore
/// enum_number! {
///     /// The `Foo` enum
///     #[derive(Clone, Copy, Deserialize, Serialize)]
///     #[serde(from = "u8", into = "u8")]
///     pub enum Foo {
///         /// First
///         Aah = 1,
///         /// Second
///         Bar = 2,
///         _ => Unknown(u8),
///     }
/// }
/// ```
///
/// Code like this will be generated:
///
/// ```
/// # use serde::{Deserialize, Serialize};
/// #
/// /// The `Foo` enum
/// #[derive(Clone, Copy, Deserialize, Serialize)]
/// #[serde(from = "u8", into = "u8")]
/// pub enum Foo {
///     /// First
///     Aah,
///     /// Second,
///     Bar,
///     /// Variant value is unknown.
///     Unknown(u8),
/// }
///
/// impl From<u8> for Foo {
///     fn from(value: u8) -> Self {
///         match value {
///             1 => Self::Aah,
///             2 => Self::Bar,
///             unknown => Self::Unknown(unknown),
///         }
///     }
/// }
///
/// impl From<Foo> for u8 {
///     fn from(value: Foo) -> Self {
///         match value {
///             Foo::Aah => 1,
///             Foo::Bar => 2,
///             Foo::Unknown(unknown) => unknown,
///         }
///     }
/// }
/// ```
macro_rules! enum_number {
    (
        $(#[$outer:meta])*
        $(#[<default> = $default:literal])?
        $vis:vis enum $Enum:ident {
            $(
                $(#[doc = $doc:literal])*
                $(#[cfg $($cfg:tt)*])?
                $Variant:ident = $value:literal,
            )*
            _ => Unknown($T:ty),
        }
    ) => {
        $(#[$outer])*
        $vis struct $Enum (pub $T);

        $(
            impl Default for $Enum {
                fn default() -> Self {
                    Self($default)
                }
            }
        )?

        #[allow(non_snake_case, non_upper_case_globals)]
        #[allow(clippy::allow_attributes, reason = "Does not always trigger due to macro")]
        impl $Enum {
            $(
                $(#[doc = $doc])*
                $(#[cfg $($cfg)*])?
                $vis const $Variant: Self = Self($value);
            )*

            /// Variant value is unknown.
            #[must_use]
            $vis const fn Unknown(val: $T) -> Self {
                Self(val)
            }
        }
    };
}

/// The macro forwards the generation to the `bitflags::bitflags!` macro and implements the default
/// (de)serialization for Discord's bitmask values.
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
    ) => {
        $(#[$outer])*
        #[repr(Rust, packed)]
        $vis struct $BitFlags($T);

        bitflags::bitflags! {
            impl $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }
        }

        bitflags!(__impl_serde $BitFlags: $T);
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
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::model::assert_json;

    #[test]
    fn enum_number() {
        enum_number! {
            #[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
            pub enum T {
                /// AAA
                A = 1,
                /// BBB
                B = 2,
                /// CCC
                C = 3,
                _ => Unknown(u8),
            }
        }

        assert_json(&T::A, json!(1));
        assert_json(&T::B, json!(2));
        assert_json(&T::C, json!(3));
        assert_json(&T::Unknown(123), json!(123));
    }
}
