pub mod prelude {
    pub use serenity_core::secrets::Token;
    pub use small_fixed_array::{FixedArray, FixedString, TruncatingInto};
    pub use to_arraystring::ToArrayString;

    pub use crate::error::{Error, Result};
}

pub mod tokio {
    use std::future::Future;

    pub fn spawn_named<F, T>(_name: &str, future: F) -> tokio::task::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        #[cfg(all(tokio_unstable, feature = "tokio_task_builder"))]
        let handle = tokio::task::Builder::new()
            .name(&*format!("serenity::{}", _name))
            .spawn(future)
            .expect("called outside tokio runtime");
        #[cfg(not(all(tokio_unstable, feature = "tokio_task_builder")))]
        let handle = tokio::spawn(future);
        handle
    }
}

#[macro_use]
pub mod macros {
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
}
