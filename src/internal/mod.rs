#[macro_use]
pub mod macros;

pub mod prelude;

#[cfg(feature = "gateway")]
pub mod ws_impl;

#[cfg(any(feature = "tokio", feature = "tokio_compat"))]
pub mod tokio;

#[cfg(feature = "transport_compression")]
mod inflater;

#[cfg(feature = "transport_compression")]
pub(crate) use inflater::Inflater;

pub fn is_false(b: &bool) -> bool {
    !b
}
