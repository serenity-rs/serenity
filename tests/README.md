### Running the Ignored Test Suite

Some tests are ignored by default, as they require authentication. These should
only be run when you need to actually test something that _requires_ hitting
Discord's REST API.

### Generic Tests

Provide the token by setting the environment variable `DISCORD_TOKEN`.

e.g.:

```sh
$ DISCORD_TOKEN=aaaaaaaaaaa TEST_USER=285951325479632862 cargo test -- --ignored
```

### Notes for Specific Tests

issues/69:

Provide a `TEST_USER`
