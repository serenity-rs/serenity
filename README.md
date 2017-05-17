[![ci-badge][]][ci] [![crate-badge][]][crate] [![license-badge][]][license] [![docs-badge][]][docs] [![contribs-badge][]][contribs] [![dapi-badge][]][dapi]

# serenity

![serenity logo][logo]

Serenity is a Rust library for the Discord API.

View the [examples] on how to make and structure a bot.

Serenity supports bot login via the use of [`Client::login`].

You may also check your tokens prior to login via the use of
[`validate_token`].

Once logged in, you may add handlers to your client to dispatch [`Event`]s,
such as [`Client::on_message`]. This will cause your handler to be called
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
    let mut client = Client::login(&env::var("DISCORD_TOKEN").expect("token"));
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
serenity = "0.2"
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

- **cache**: The cache will store information about guilds, channels, users, and
other data, to avoid performing REST requests. If you are low on RAM, do not
enable this;
- **framework**: Enables the framework, which is a utility to allow simple
command parsing, before/after command execution, prefix setting, and more;
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

- [discord-rs][rel:discord-rs] (Rust)
- [eris][rel:eris] (JavaScript)
- [Discord.Net][rel:discord.net] (.NET)
- [discord.py][rel:discord.py] (Python)
- [discordrb][rel:discordrb] (Ruby)

[`Cache`]: https://serenity.zey.moe/serenity/ext/cache/struct.Cache.html
[`Client::login`]: https://serenity.zey.moe/serenity/client/struct.Client.html#method.login
[`Client::on_message`]: https://serenity.zey.moe/serenity/client/struct.Client.html#method.on_message
[`Shard`]: https://serenity.zey.moe/serenity/client/gateway/struct.Shard.html
[`Context`]: https://serenity.zey.moe/serenity/client/struct.Context.html
[`Event`]: https://serenity.zey.moe/serenity/model/enum.Event.html
[`Event::MessageCreate`]: https://serenity.zey.moe/serenity/model/enum.Event.html#variant.MessageCreate
[`examples`]: https://github.com/zeyla/serenity/blob/master/examples
[`rest`]: https://serenity.zey.moe/serenity/client/rest/index.html
[`validate_token`]: https://serenity.zey.moe/serenity/client/fn.validate_token.html
[cache docs]: https://serenity.zey.moe/serenity/ext/cache/index.html
[ci]: https://travis-ci.org/zeyla/serenity
[ci-badge]: https://travis-ci.org/zeyla/serenity.svg?branch=master
[contribs]: https://img.shields.io/github/contributors/zeyla/serenity.svg
[contribs-badge]: https://img.shields.io/github/contributors/zeyla/serenity.svg
[crate]: https://crates.io/crates/serenity
[crate-badge]: https://img.shields.io/crates/v/serenity.svg?maxAge=2592000
[client's module-level documentation]: https://serenity.zey.moe/serenity/client/index.html
[dapi]: https://discord.gg/PgQYQcc
[dapi-badge]: https://discordapp.com/api/guilds/81384788765712384/widget.png
[discord docs]: https://discordapp.com/developers/docs/intro
[docs]: https://serenity.zey.moe/
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg
[examples]: https://github.com/zeyla/serenity/tree/master/examples
[gateway docs]: https://serenity.zey.moe/serenity/client/gateway/index.html
[license]: https://opensource.org/licenses/ISC
[license-badge]: https://img.shields.io/badge/license-ISC-blue.svg
[logo]: https://raw.githubusercontent.com/zeyla/serenity/master/logo.png
[rel:discord-rs]: https://github.com/SpaceManiac/discord-rs
[rel:discord.net]: https://github.com/RogueException/Discord.Net
[rel:discord.py]: https://github.com/Rapptz/discord.py
[rel:discordrb]: https://github.com/meew0/discordrb
[rel:eris]: https://github.com/abalabahaha/eris
