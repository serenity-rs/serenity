[![ci-badge][]][ci] [![docs-badge][]][docs] [![guild-badge][]][guild] [![crates.io version]][crates.io link] [![rust 1.39.0+ badge]][rust 1.39.0+ link]

# serenity

![serenity logo][logo]

Serenity is a Rust library for the Discord API.

View the [examples] on how to make and structure a bot.

Serenity supports bot login via the use of [`Client::new`].

You may also check your tokens prior to login via the use of
[`validate_token`].

Once logged in, you may add handlers to your client to dispatch [`Event`]s,
by implementing the handlers in a trait, such as [`EventHandler::message`].
This will cause your handler to be called when a [`Event::MessageCreate`] is
received. Each handler is given a [`Context`], giving information about the
event. See the [client's module-level documentation].

The [`Shard`] is transparently handled by the library, removing
unnecessary complexity. Sharded connections are automatically handled for
you. See the [gateway's documentation][gateway docs] for more information.

A [`Cache`] is also provided for you. This will be updated automatically for
you as data is received from the Discord API via events. When calling a
method on a [`Context`], the cache will first be searched for relevant data
to avoid unnecessary HTTP requests to the Discord API. For more information,
see the [cache's module-level documentation][cache docs].

Note that - although this documentation will try to be as up-to-date and
accurate as possible - Discord hosts [official documentation][discord docs]. If
you need to be sure that some information piece is accurate, refer to their
docs.

# Example Bot

A basic ping-pong bot looks like:

```rust,ignore
use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::prelude::{EventHandler, Context};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

#[group]
#[commands(ping)]
struct General;

use std::env;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!")?;

    Ok(())
}

```

### Full Examples

Full examples, detailing and explaining usage of the basic functionality of the
library, can be found in the [`examples`] directory.

# Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
serenity = "0.8"
```

Serenity supports a minimum of Rust 1.37.

# Features

Features can be enabled or disabled by configuring the library through
Cargo.toml:

```toml
[dependencies.serenity]
default-features = false
features = ["pick", "your", "feature", "names", "here"]
version = "0.8"
```

The default features are: `builder`, `cache`, `client`, `framework`, `gateway`,
`http`, `model`, `standard_framework`, `utils`, and `rustls_backend`.

The following is a full list of features:

- **builder**: The builders used in conjunction with models' methods.
- **cache**: The cache will store information about guilds, channels, users, and
other data, to avoid performing REST requests. If you are low on RAM, do not
enable this.
- **client**: A manager for shards and event handlers, abstracting away the
work of handling shard events and updating the cache, if enabled.
- **framework**: Enables the framework, which is a utility to allow simple
command parsing, before/after command execution, prefix setting, and more.
- **gateway**: A Shard, used as a higher-level interface for communicating with
the Discord gateway over a WebSocket client.
- **http**: Functions providing a wrapper over Discord's REST API at a low
enough level that optional parameters can be provided at will via a JsonMap.
- **model**: Method implementations for models, acting as helper methods over
the HTTP functions.
- **standard_framework**: A standard, default implementation of the Framework
- **utils**: Utility functions for common use cases by users.
- **voice**: Enables compilation of voice support, so that voice channels can be
connected to and audio can be sent/received.
- **default_native_tls**: Default features but using `native_tls_backend`
instead of `rustls_backend`.
- **absolute_ratelimits**: Whether the library should use your system clock to avoid
ratelimits, or use the interval given by Discord that might be less efficient
due to latency in the network. If you turn this feature on, it is recommended to
synchronise your clock with an NTP server (such as Google's).

Serenity offers two TLS-backends, `rustls_backend` by default, you need to pick
one if you do not use the default features:

- **rustls_backend**: Uses Rustls for all platforms, a pure Rust
TLS implementation.
- **native_tls_backend**: Uses SChannel on Windows, Secure Transport on macOS,
and OpenSSL on other platforms.


If you want all of the default features except for `cache` for example, you can
list all but that:

```toml
[dependencies.serenity]
default-features = false
features = [
    "builder",
    "client",
    "framework",
    "gateway",
    "http",
    "model",
    "standard_framework",
    "utils",
    "rustls_backend",
]
version = "0.8"
```

# Dependencies

If you use the `native_tls_backend` and you are not developing on macOS or Windows, you will need:

- openssl

If you want to use `voice`, Serenity will attempt to build these for you:

- libsodium (Arch: `community/libsodium`)
- opus (Arch: `extra/opus`)

In case the automated building fails, you may report it to us, but installing should fix it.

Voice + ffmpeg:

- ffmpeg (Arch: `extra/ffmpeg`)

Voice + youtube-dl:

- youtube-dl (Arch: `community/youtube-dl`)

# Projects extending Serenity

- [serenity-lavalink][project:serenity-lavalink]: An interface to [Lavalink][repo:lavalink], an audio sending node based on [Lavaplayer][repo:lavaplayer]

[`Cache`]: https://docs.rs/serenity/*/serenity/cache/struct.Cache.html
[`Client::new`]: https://docs.rs/serenity/*/serenity/client/struct.Client.html#method.new
[`EventHandler::message`]: https://docs.rs/serenity/*/serenity/client/trait.EventHandler.html#method.message
[`Context`]: https://docs.rs/serenity/*/serenity/client/struct.Context.html
[`Event`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html
[`Event::MessageCreate`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html#variant.MessageCreatef
[`Shard`]: https://docs.rs/serenity/*/serenity/gateway/struct.Shard.html
[`examples`]: https://github.com/serenity-rs/serenity/blob/current/examples
[`rest`]: https://docs.rs/serenity/*/serenity/client/rest/index.html
[`validate_token`]: https://docs.rs/serenity/*/serenity/client/fn.validate_token.html
[cache docs]: https://docs.rs/serenity/*/serenity/cache/index.html
[ci]: https://dev.azure.com/serenity-org/serenity/_build?definitionId=1
[ci-badge]: https://img.shields.io/azure-devops/build/serenity-org/1ce9579e-03bc-499f-9302-4180a2dfec6f/1/next.svg?style=flat-square
[client's module-level documentation]: https://docs.rs/serenity/*/serenity/client/index.html
[crates.io link]: https://crates.io/crates/serenity
[crates.io version]: https://img.shields.io/crates/v/serenity.svg?style=flat-square
[discord docs]: https://discord.com/developers/docs/intro
[docs]: https://docs.rs/serenity
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square
[examples]: https://github.com/serenity-rs/serenity/tree/current/examples
[gateway docs]: https://docs.rs/serenity/*/serenity/gateway/index.html
[guild]: https://discord.gg/9X7vCus
[guild-badge]: https://img.shields.io/discord/381880193251409931.svg?style=flat-square&colorB=7289DA
[project:serenity-lavalink]: https://gitlab.com/nitsuga5124/serenity-lavalink/
[repo:lavalink]: https://github.com/Frederikam/Lavalink
[repo:lavaplayer]: https://github.com/sedmelluq/lavaplayer
[logo]: https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png
[rust 1.39.0+ badge]: https://img.shields.io/badge/rust-1.39.0+-93450a.svg?style=flat-square
[rust 1.39.0+ link]: https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html
