# Serenity Examples

The examples listed in each directory demonstrate different use cases of the
library, and increasingly show more advanced or in-depth code.

All examples have documentation for new concepts, and try to explain any new
concepts. Examples should be completed in order, so as not to miss any
documentation.

### Running Examples

To run an example, you have the option of either:

1. cloning this repository, `cd`ing into the example's directory, and then
running `cargo run` to run the example; or
2. copying the contents of the example into your local binary project
(created via `cargo new test-project --bin`) and ensuring that the contents of
the `Cargo.toml` file contains that of the example's `[dependencies]` section,
and _then_ executing `cargo run`.

Note that all examples - by default - require an environment token of
`DISCORD_TOKEN` to be set. If you don't like environment tokens, you can
hardcode your token in.

### Questions

If you have any questions, feel free to submit an issue with what can be
clarified.

### Contributing

If you add a new example also add it to the file `azure-build-examples.yml`.
