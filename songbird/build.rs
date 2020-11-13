#[cfg(all(feature = "driver", not(any(feature = "rustls", feature = "native"))))]
compile_error!(
    "You have the `driver` feature enabled: \
    either the `rustls` or `native` feature must be
    selected to let Songbird's driver use websockets.\n\
    - `rustls` uses Rustls, a pure Rust TLS-implemenation.\n\
    - `native` uses SChannel on Windows, Secure Transport on macOS, \
    and OpenSSL on other platforms.\n\
    If you are unsure, go with `rustls`."
);

#[cfg(all(
    feature = "twilight",
    not(any(feature = "simd-zlib", feature = "stock-zlib"))
))]
compile_error!(
    "Twilight requires you to specify a zlib backend: \
    either the `simd-zlib` or `stock-zlib` feature must be
    selected.\n\
    If you are unsure, go with `stock-zlib`."
);

#[cfg(all(feature = "youtube-dl", feature = "youtube-dlc"))]
compile_error!(
    "Only youtube-dl or youtube-dlc can be selected \
    - `youtube-dl` is the standard downloading tool of youtube videos \
    - `youtube-dlc` is a fork of youtube-dl that aims to solve bugs with youtube-dl \
    If you are unsure, go with `youtube-dl`, however, if you encounter any errors with `youtube-dl`,
    such as songs ending immedietly when they are queued/played, try 'youtube-dlc'."
);

fn main() {}
