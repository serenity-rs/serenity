# Serenity Examples

The examples listed in each directory demonstrate different use cases of the library, and
increasingly show more advanced or in-depth code.

All examples have documentation and try to explain any new concepts. Examples should be completed
in order, so as not to miss any documentation.

If you are looking for voice examples, they can be found in the [songbird] and [lavalink-rs]
repositories.

To provide a token for the examples, you need to set the `DISCORD_TOKEN` environmental variable to
the Bot token. If you prefer not to use environment tokens, you can hardcode your token instead.
> **Tip:** A valid token starts with M, N, or O, and includes two periods.

## Running Examples

To run an example, you have various options:

1. Use [cargo-make]:
    - Install cargo-make: `cargo install --force cargo-make`
    - Clone the repository: `git clone https://github.com/serenity-rs/serenity.git`
    - CD into the Serenity folder: `cd serenity`
    - Run `cargo make <number>`, where `<number>` is the number of the example you wish to run:

      | No. | Example Name | Description |
      | --- | --- | --- |
      1 | [Basic Ping Bot] | A bare-minimum Serenity application. |
      2 | [Transparent Guild Sharding] | How to use sharding and shared cache. |
      3 | [Model Methods] | Simple usage of wrapper methods on model types. |
      4 | [Message Builder] | A demonstration of the message builder utility, to generate messages safely. |
      5 | [Simple Bot Structure] | An example showing the recommended file structure, with slash commands. |
      6 | [Env Logging] | How to use the [`tracing`] crate with Serenity. |
      7 | [Shard Manager] | How to get started with using the shard manager. |
      8 | [Create Message Builder] | How to send embeds and files. |
      9 | [Collectors] | How to use the collectors feature to wait for messages and reactions. |
      10 | [Gateway Intents] | How to use intents to limit the events the bot will receive. |
      11 | [Global Data] | How to use the client data to share data between commands and events safely. |
      12 | [Parallel Loops] | How to run tasks in a loop with context access.<br/>Additionally, shows how to send a message to a specific channel. |
      13 | [SQLite Database] | How to run an embedded SQLite database alongside the bot using SQLx. |
      14 | [Message Components] | How to structure and use buttons and select menus. |
      15 | [Webhook] | How to construct and call a webhook. |
      16 | [Interactions Endpoint] | How to use HTTP-based interactions instead of the Gateway. |

2. Manually running:
    - Clone the repository: `git clone https://github.com/serenity-rs/serenity.git`
    - Run the example of choice, selected using the `-p` flag: `cargo run --release -p e01_basic_ping_bot `

3. Copy-paste:
    - Copy the contents of the example into your local binary project (created via
      `cargo new test-project --bin`)
    - Ensure that your project's `Cargo.toml` file includes the crates from the `[dependencies]`
      section of the example's `Cargo.toml`
    - Run the example using `cargo run`

### Questions

If you have any questions, feel free to submit an issue or ask in [the Serenity server] on Discord.

### Contributing

If you add a new example, also add it to the following files:

- `.github/workflows/ci.yml`
- `Makefile.toml`
- `examples/README.md`

[songbird]: https://github.com/serenity-rs/songbird/tree/current/examples/serenity
[lavalink-rs]: https://gitlab.com/vicky5124/lavalink-rs/-/tree/master/examples
[cargo-make]: https://crates.io/crates/cargo-make
[Basic Ping Bot]: /examples/e01_basic_ping_bot/
[Transparent Guild Sharding]: /examples/e02_transparent_guild_sharding/
[Model Methods]: /examples/e03_model_methods/
[Message Builder]: /examples/e04_message_builder/
[Simple Bot Structure]: /examples/e05_sample_bot_structure/
[Env Logging]: /examples/e06_env_logging/
[`tracing`]: https://crates.io/crates/tracing
[Shard Manager]: /examples/e07_shard_manager/
[Create Message Builder]: /examples/e08_create_message_builder/
[Collectors]: /examples/e09_collectors/
[Gateway Intents]: /examples/e10_gateway_intents/
[Global Data]: /examples/e11_global_data/
[Parallel Loops]: /examples/e12_parallel_loops/
[SQLite Database]: /examples/e13_sqlite_database/
[Message Components]: /examples/e14_message_components/
[Webhook]: /examples/e15_webhook/
[Interactions Endpoint]: /examples/e16_interactions_endpoint/
[the Serenity server]: https://discord.gg/serenity-rs