# Serenity Examples

The examples listed in each directory demonstrate different use cases of the
library, and increasingly show more advanced or in-depth code.

All examples have documentation for new concepts, and try to explain any new
concepts. Examples should be completed in order, so as not to miss any
documentation.

To provide a token for them to use, you need to set the `DISCORD_TOKEN`
environmental variable to the Bot token.\
If you don't like environment tokens, you can hardcode your token in instead.\
TIP: A valid token starts with M, N or O and has 2 dots.

### Running Examples

To run an example, you have various options:

1. [cargo-make](https://lib.rs/crates/cargo-make)
- Clone the repository: `git clone https://github.com/serenity-rs/serenity.git`
- CD into the serenity folder: `cd serenity`
- Run `cargo make 1`, where 1 is the number of the example you wish to run; these are:
```
 1 => Basic Ping Bot: A bare minimum serenity application.
 2 => Transparent Guild Sharding: How to use sharding and shared cache.
 3 => Structure Utilities: Simple usage of the utils feature.
 4 => Message Builder: A demonstration of the message builder utility, to generate messages safely.
 5 => Command Framework: The main example, where it's demonstrated how to use serenity's command framework,
      along with mosts of it's utilities.
      This example also shows how to share data between events and commands, using `Context.data`
 6 => Voice: Simple example on playing back audio with serenity, along with FFMPEG and Youtube-DL.
 7 => Simple Bot Stucture: An example showing the recommended file structure to use.
 8 => Env Logging: How to use the tracing crate along with serenity.
 9 => Shard Manager: How to get started with using the shard manager.
10 => Voice Recieve: How to recieve voice audio packets.
      WARNING: This example *may* not work, PR #806 is aiming to fix this.
11 => Create Message Builder: How to send embeds and files.
12 => Collectors: How to use the collectors feature to wait for messages and reactions.
13 => Gateway Intents: How to use intents to limit the events the bot will recieve.
14 => Global Data: How to use the client data to share data between commands and events safely.
15 => Parallel Loops: How to run tasks in a loop with context access.
      Additionally, show how to send a message to a specific channel.
```

2. Manualy running:
- Clone the repository: `git clone https://github.com/serenity-rs/serenity.git`
- CD into the examples folder: `cd serenity/examples`
- CD into the example of choice: `cd e01_basic_ping_bot`
- Run the example: `cargo run --release`

3. Copy Paste:
- Copy the contents of the example into your local binary project\
(created via `cargo new test-project --bin`)\
and ensuring that the contents of the `Cargo.toml` file
contains that of the example's `[dependencies]` section,\
and _then_ executing `cargo run`.

### Questions

If you have any questions, feel free to submit an issue with what can be
clarified.

### Contributing

If you add a new example also add it to the following files:
- `azure-build-examples.yml`
- `Makefile.toml`
- `examples/README.md`
