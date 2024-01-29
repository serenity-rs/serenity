# Setting up the database

In order to compile the project, a database needs to be set-up. That's because SQLx accesses the
database at compile time to make sure your SQL queries are correct.

To set up the database, download the [SQLx CLI](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli)
and run `sqlx database setup`. This command will create the database and its tables by applying
the migration files in `migrations/`.

Most SQLx CLI commands require the `DATABASE_URL` environment variable to be set to the database
URL, for example `sqlite:database.sqlite` (where `sqlite:` is the protocol and `database.sqlite` the
actual filename). A convenient way to supply this information to SQLx is to create a `.env` file
which SQLx automatically detects and reads:

```rust
DATABASE_URL=sqlite:database.sqlite
```

# Running the example

```sh
# Note: due to a bug in SQLx (https://github.com/launchbadge/sqlx/issues/3099),
# you have to provide the full path to `DATABASE_URL` when compiling.
# Once the bug is fixed, you can omit `DATABASE_URL=...` and let SQLx read the `.env` file.
DATABASE_URL=sqlite:examples/e16_sqlite_database/database.sqlite DISCORD_TOKEN=... cargo run
```

Interact with the bot via `~todo list`, `~todo add` and `~todo remove`.

# What are migrations

In SQLx, migrations are SQL query files that update the database schema. Most SQLx project have at
least one migration file, often called `initial_migration`, which sets up tables initially.

If you need to modify the database schema in the future, call `sqlx migrate add "MIGRATION NAME"`
and write the migration queries into the newly created .sql file in `migrations/`.

# Make it easy to host your bot

Normally, users have to download and install SQLx CLI in order to compile your bot. Remember:
SQLx accesses the database at compile time. However, you can enable building in "offline mode":
https://github.com/launchbadge/sqlx/tree/master/sqlx-cli#enable-building-in-offline-mode-with-query.
That way, your bot will work out-of-the-box with `cargo run`.

Note that users still have to set `SQLX_OFFLINE` to `true` even if `sqlx-data.json` is present.

Tip: create a git pre-commit hook which executes `cargo sqlx prepare` for you before every commit.
See the `pre-commit` file for an example. Copy the file into `.git/hooks` to install the pre-commit
hook.

# Using SQLx

SQLx's GitHub repository explains a lot about SQLx, like the difference between `query!` and
`query_as!`. Please follow the links to learn more:

- SQLx: https://github.com/launchbadge/sqlx
- SQLx CLI: https://github.com/launchbadge/sqlx/tree/master/sqlx-cli
