[![ci-badge][]][ci] [![docs-badge][]][docs] [![guild-badge][]][guild]

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

```rust,ignore
#[macro_use] extern crate serenity;

use serenity::client::Client;
use serenity::prelude::EventHandler;
use serenity::framework::standard::StandardFramework;
use std::env;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .cmd("ping", ping));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
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
serenity = "0.5"
```

and to the top of your `main.rs`:

```rust,ignore
#[macro_use] extern crate serenity;
```

Serenity only supports the _latest_ Stable, Beta, and Nightly.

# Features

Features can be enabled or disabled by configuring the library through
Cargo.toml:

```toml
[dependencies.serenity]
default-features = false
features = ["pick", "your", "feature", "names", "here"]
version = "0.5"
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

# Related Projects

- [Discord.net][library:Discord.net]
- [JDA][library:JDA]
- [disco][library:disco]
- [discordrb][library:discordrb]
- [discord.js][library:discord.js]
- [discord.py][library:discord.py]

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
[guild]: https://discord.gg/WBdGJCc
[guild-badge]: https://discordapp.com/api/guilds/381880193251409931/widget.png
[library:Discord.net]: https://github.com/RogueException/Discord.Net
[library:JDA]: https://github.com/DV8FromTheWorld/JDA
[library:disco]: https://github.com/b1naryth1ef/disco
[library:discordrb]: https://github.com/meew0/discordrb
[library:discord.js]: https://github.com/hydrabolt/discord.js
[library:discord.py]: https://github.com/Rapptz/discord.py
[logo]: https://zey.moe/assets/serenity_logo.png
