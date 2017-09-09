[![ci-badge][]][ci] [![docs-badge][]][docs]

# serenity

![serenity logo][logo]

Serenity is a Rust library for the Discord API.

View the [examples] on how to make and structure a bot.

Serenity supports bot login via the use of [`Client::new`].

You may also check your tokens prior to login via the use of
[`validate_token`].

Once logged in, you may add handlers to your client to dispatch [`Event`]s,
by implementing the handlers in a trait, such as [`EventHandler::on_message`]. This will cause your handler to be called
when a [`Event::MessageCreate`] is received. Each handler is given a
[`Context`], giving information about the event. See the
[client's module-level documentation].

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

```rust,no-run
#[macro_use] extern crate serenity;

use serenity::client::Client;
use std::env;

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"));
    client.with_framework(|f| f
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .on("ping", ping));

    // start listening for events by starting a single shard
    let _ = client.start();
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});
```

### Full Examples

Full examples, detailing and explaining usage of the basic functionality of the
library, can be found in the [`examples`] directory.

# Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
serenity = "0.3"
```

and to the top of your `main.rs`:

```rs
#[macro_use] extern crate serenity;
```

Serenity only supports the _latest_ Stable, Beta, and Nightly.

# Features

Features can be enabled or disabled by configuring the library through
Cargo.toml:

```toml
[dependencies.serenity]
git = "https://github.com/zeyla/serenity.git"
default-features = false
features = ["pick", "your", "feature", "names", "here"]
```

The following is a full list of features:

- **builder**: The builders used in conjunction with models' methods.
- **cache**: The cache will store information about guilds, channels, users, and
other data, to avoid performing REST requests. If you are low on RAM, do not
enable this;
- **client**: A manager for shards and event handlers, abstracting work away
handling shard events and updating the cache, if enabled.
- **framework**: Enables the framework, which is a utility to allow simple
command parsing, before/after command execution, prefix setting, and more;
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

# Dependencies

Serenity requires the following dependencies:

- openssl

### Voice

The following dependencies all require the **voice** feature to be enabled in
your Cargo.toml:

- libsodium (Arch: `community/libsodium`)
- opus (Arch: `extra/opus`)

Voice+ffmpeg:

- ffmpeg (Arch: `extra/ffmpeg`)

Voice+youtube-dl:

- youtube-dl (Arch: `community/youtube-dl`)

[`Cache`]: https://docs.rs/serenity/*/serenity/cache/struct.Cache.html
[`Client::new`]: https://docs.rs/serenity/*/serenity/client/struct.Client.html#method.new
[`EventHandler::on_message`]: https://docs.rs/serenity/*/serenity/client/struct.EventHandler.html#method.on_message
[`Context`]: https://docs.rs/serenity/*/serenity/client/struct.Context.html
[`Event`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html
[`Event::MessageCreate`]: https://docs.rs/serenity/*/serenity/model/event/enum.Event.html#variant.MessageCreatef
[`Shard`]: https://docs.rs/serenity/*/serenity/gateway/struct.Shard.html
[`examples`]: https://github.com/zeyla/serenity/blob/master/examples
[`rest`]: https://docs.rs/serenity/*/serenity/client/rest/index.html
[`validate_token`]: https://docs.rs/serenity/*/serenity/client/fn.validate_token.html
[cache docs]: https://docs.rs/serenity/*/serenity/cache/index.html
[ci]: https://travis-ci.org/zeyla/serenity
[ci-badge]: https://travis-ci.org/zeyla/serenity.svg?branch=master
[client's module-level documentation]: https://docs.rs/serenity/*/serenity/client/index.html
[discord docs]: https://discordapp.com/developers/docs/intro
[docs]: https://docs.rs/serenity
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg
[examples]: https://github.com/zeyla/serenity/tree/master/examples
[gateway docs]: https://docs.rs/serenity/*/serenity/gateway/index.html
[logo]: https://raw.githubusercontent.com/zeyla/serenity/master/logo.png
