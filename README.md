[![ci-badge][]][ci] [![docs-badge][]][docs] [![guild-badge][]][guild] [![crates.io version]][crates.io link] [![rust-version-badge]][rust-version-link]

# serenity

![serenity logo][logo]

Serenity is a Rust library for the Discord API.

View the [examples] on how to make and structure a bot.

Serenity supports bot login via the use of [`Client::builder`].

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
use std::env;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(|c| c.prefix("~")); // set the bot's prefix to "~"

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
```

## Full Examples

Full examples, detailing and explaining usage of the basic functionality of the
library, can be found in the [`examples`] directory.

# Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
serenity = "0.11"
```

## MSRV Policy

Serenity's minimum supported Rust version (MSRV) is Rust 1.65.

We opt to keep MSRV stable on the `current` branch. This means it will remain
unchanged between minor releases. Occasionally, dependencies may violate SemVer
and update their own MSRV in a breaking way. As a result, pinning their
versions will become necessary to successfully build Serenity using an older
Rust release.

The `next` branch tracks the latest Rust release as its MSRV. This allows for
swift development as new languages features are stabilized, and reduces
technical debt in the long run. When a new major release is cut, the MSRV on
`current` will be updated to that of `next`, and we will commit to supporting
that MSRV until the following major release.

# Features

Features can be enabled or disabled by configuring the library through
Cargo.toml:

```toml
[dependencies.serenity]
default-features = false
features = ["pick", "your", "feature", "names", "here"]
version = "0.11"
```

The default features are: `builder`, `cache`, `client`, `framework`, `gateway`,
`http`, `model`, `standard_framework`, `utils`, and `rustls_backend`.

There are these alternative default features, they require to set `default-features = false`:

- **default_native_tls**: Uses `native_tls_backend` instead of the default `rustls_backend`.
- **default_no_backend**: Excludes the default backend, pick your own backend instead.

If you are unsure which to pick, use the default features by not setting `default-features = false`.

The following is a full list of features:

- **builder**: The builders used in conjunction with models' methods.
- **cache**: The cache will store information about guilds, channels, users, and
other data, to avoid performing REST requests. If you are low on RAM, do not
enable this.
- **collector**: A collector awaits events, such as receiving a message from a user or reactions on a message, and allows for responding to the events in a convenient fashion. Collectors can be configured to enforce certain criteria the events must meet.
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
- **time**: Use the `time` crate for Discord's timestamp fields. See `serenity::model::Timestamp`.
- **utils**: Utility functions for common use cases by users.
- **voice**: Enables registering a voice plugin to the client, which will handle actual voice connections from Discord.
[lavalink-rs][project:lavalink-rs] or [Songbird][project:songbird] are recommended voice plugins.
- **default_native_tls**: Default features but using `native_tls_backend`
instead of `rustls_backend`.
- **absolute_ratelimits**: Whether the library should use your system clock to avoid
ratelimits, or use the interval given by Discord that might be less efficient
due to latency in the network. If you turn this feature on, it is recommended to
synchronise your clock with an NTP server (such as Google's).
- **tokio_task_builder**: Enables tokio's `tracing` feature and uses `tokio::task::Builder` to spawn tasks with names if `RUSTFLAGS="--cfg tokio_unstable"` is set.
- **unstable_discord_api**: Enables features of the Discord API that do not have a stable interface. The features might not have official documentation or are subject to change.
- **simd_json**: Enables SIMD accelerated JSON parsing and rendering for API calls, use with `RUSTFLAGS="-C target-cpu=native"`
- **temp_cache**: Enables temporary caching in functions that retrieve data via the HTTP API.

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
    "chrono",
    "client",
    "framework",
    "gateway",
    "http",
    "model",
    "standard_framework",
    "utils",
    "rustls_backend",
]
version = "0.11"
```

# Dependencies

If you use the `native_tls_backend` and you are not developing on macOS or Windows, you will need:

- openssl

# Hosting

If you want a quick and easy way to host your bot, you can use [shuttle][project:shuttle],
a Rust-native cloud development platform that allows deploying Serenity bots for free.

# Projects extending Serenity

- [lavalink-rs][project:lavalink-rs]: An interface to [Lavalink][repo:lavalink] and [Andesite][repo:andesite], an audio sending node based on [Lavaplayer][repo:lavaplayer]
- [Songbird][project:songbird]: An async Rust library for the Discord voice API.
- [Poise][project:poise]: Experimental command framework, with advanced features like edit tracking, single function slash and prefix commands and flexible argument parsing.

[`Cache`]: https://docs.rs/serenity/*/serenity/cache/struct.Cache.html
[`Client::builder`]: https://docs.rs/serenity/*/serenity/client/struct.Client.html#method.builder
[`EventHandler::message`]: https://docs.rs/serenity/*/serenity/client/trait.EventHandler.html#method.message
[`Context`]: https://docs.rs/serenity/*/serenity/client/struct.Context.html
[`Event`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html
[`Event::MessageCreate`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html#variant.MessageCreate
[`Shard`]: https://docs.rs/serenity/*/serenity/gateway/struct.Shard.html
[`examples`]: https://github.com/serenity-rs/serenity/blob/current/examples
[`rest`]: https://docs.rs/serenity/*/serenity/client/rest/index.html
[`validate_token`]: https://docs.rs/serenity/*/serenity/utils/fn.validate_token.html
[cache docs]: https://docs.rs/serenity/*/serenity/cache/index.html
[ci]: https://github.com/serenity-rs/serenity/actions
[ci-badge]: https://img.shields.io/github/workflow/status/serenity-rs/serenity/CI?style=flat-square
[client's module-level documentation]: https://docs.rs/serenity/*/serenity/client/index.html
[crates.io link]: https://crates.io/crates/serenity
[crates.io version]: https://img.shields.io/crates/v/serenity.svg?style=flat-square
[discord docs]: https://discord.com/developers/docs/intro
[docs]: https://docs.rs/serenity
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square
[examples]: https://github.com/serenity-rs/serenity/tree/current/examples
[gateway docs]: https://docs.rs/serenity/*/serenity/gateway/index.html
[guild]: https://discord.gg/serenity-rs
[guild-badge]: https://img.shields.io/discord/381880193251409931.svg?style=flat-square&colorB=7289DA
[project:lavalink-rs]: https://gitlab.com/vicky5124/lavalink-rs/
[project:songbird]: https://github.com/serenity-rs/songbird
[project:poise]: https://github.com/kangalioo/poise
[project:shuttle]: https://github.com/shuttle-hq/shuttle
[repo:lavalink]: https://github.com/freyacodes/Lavalink
[repo:andesite]: https://github.com/natanbc/andesite
[repo:lavaplayer]: https://github.com/sedmelluq/lavaplayer
[logo]: https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png
[rust-version-badge]: https://img.shields.io/badge/rust-1.65.0+-93450a.svg?style=flat-square
[rust-version-link]: https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html
