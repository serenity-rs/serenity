#[cfg(all(any(feature = "http", feature = "gateway"),
    not(any(feature = "rustls_backend", feature = "native_tls_backend"))))]
compile_error!("You have the `http` or `gateway` feature enabled, \
    either the `rustls_backend` or `native_tls_backend` feature must be
    selected to let Serenity use `http` or `gateway`.\n\
    - `rustls_backend` uses Rustls, a pure Rust TLS-implemenation.\n\
    - `native_tls_backend` uses SChannel on Windows, Secure Transport on macOS, \
    and OpenSSL on other platforms.\n\
    If you are unsure, go with `rustls_backend`.");
#[cfg(all(feature = "voice", not(any(feature = "secretbox", feature = "sodiumoxide"))))]
compile_error!("You have the `voice` feature enabled, either the `secretbox` or `sodiumoxide` feature must be selected to let Serenity use `voice`.\n\
               - `secretbox` uses a pure rust secretbox implementation that will work on any system.\n\
               - `sodiumoxide` uses an optimised C-library.\n\
               If you are cross-compiling or compiling for Windows go with `secretbox`.");

fn main() {}

