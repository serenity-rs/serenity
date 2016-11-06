[![ci-badge][]][ci] [![license-badge][]][license] [![docs-badge][]][docs]

# serenity.rs

Serenity is an ergonomic and high-level Rust library for the Discord API.

View the [examples] on how to make and structure a bot.

Serenity supports both bot and user login via the use of [`Client::login_bot`]
and [`Client::login_user`].

You may also check your tokens prior to login via the use of
[`validate_token`].

Once logged in, you may add handlers to your client to dispatch [`Event`]s,
such as [`Client::on_message`]. This will cause your handler to be called
when a [`Event::MessageCreate`] is received. Each handler is given a
[`Context`], giving information about the event. See the
[client's module-level documentation].

The [`Connection`] is transparently handled by the library, removing
unnecessary complexity. Sharded connections are automatically handled for
you. See the [Connection's documentation][`Connection`] for more
information.

A [`State`] is also provided for you. This will be updated automatically for
you as data is received from the Discord API via events. When calling a
method on a [`Context`], the state will first be searched for relevant data
to avoid unnecessary HTTP requests to the Discord API. For more information,
see the [state's module-level documentation][state docs].

Note that - although this documentation will try to be as up-to-date and
accurate as possible - Discord hosts [official documentation][discord docs]. If
you need to be sure that some information piece is accurate, refer to their
docs.

# Dependencies

Serenity requires the following dependencies:

- openssl

# Example Bot

A basic ping-pong bot looks like:

```rust,no-run
extern crate serenity;

use serenity::Client;

fn main() {
    // Login with a bot token from the environment
    let client = Client::login_bot(env::var("DISCORD_TOKEN").expect("token"));
    client.with_framework(|f| f
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .on("ping", |_context, message| drop(message.reply("Pong!"))));

    let _ = client.start(); // start listening for events by starting a connection
}
```

[`Client::login_bot`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/struct.Client.html#method.login_bot
[`Client::login_user`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/struct.Client.html#method.login_user
[`Client::on_message`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/struct.Client.html#method.on_message
[`validate_token`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/fn.validate_token.html
[`Connection`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/struct.Connection.html
[`Context`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/struct.Context.html
[`Event`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/model/enum.Event.html
[`Event::MessageCreate`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/model/enum.Event.html#MessageCreate.v
[`State`]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/ext/state/struct.State.html
[ci]: https://travis-ci.org/zeyla/serenity.rs
[ci-badge]: https://travis-ci.org/zeyla/serenity.rs.svg?branch=master
[client's module-level documentation]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/client/index.html
[discord docs]: https://discordapp.com/developers/docs/intro
[docs]: https://docs.austinhellyer.me/serenity.rs/latest/serenity
[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg
[examples]: https://github.com/zeyla/serenity.rs/tree/master/examples
[license]: https://opensource.org/licenses/ISC
[license-badge]: https://img.shields.io/badge/license-ISC-blue.svg?style=flat-square
[state docs]: https://docs.austinhellyer.me/serenity.rs/latest/serenity/ext/state/index.html
