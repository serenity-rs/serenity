#[cfg(all(feature = "http", not(any(feature = "rustls_backend", feature = "native_tls_backend"))))]
compile_error!(
    "You have the `http` feature enabled; either the `rustls_backend` or `native_tls_backend` \
    feature must be enabled to let Serenity make requests over the network.\n\
    - `rustls_backend` uses Rustls, a pure Rust TLS-implemenation.\n\
    - `native_tls_backend` uses SChannel on Windows, Secure Transport on macOS, and OpenSSL on \
    other platforms.\n\
    If you are unsure, go with `rustls_backend`."
);

fn main() {
    println!("cargo:rustc-check-cfg=cfg(tokio_unstable, ignore_serenity_deprecated)");
}
