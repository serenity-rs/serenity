//! This example will help us use an SQLite database with our bot.
//!
//! It is recommended that you read the README file first, as it contains important information
//! about this example.
use std::fmt::Write as _;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::event::FullEvent;
use serenity::model::prelude::*;
use serenity::prelude::*;

// Like in the previous parallel loops example, we place data we will want to access from the
// event handler in our Handler struct. In this example, it is our database connection pool.
struct Handler {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                let user_id = new_message.author.id.get() as i64;

                if let Some(task_description) = new_message.content.strip_prefix("~todo add") {
                    let task_description = task_description.trim();
                    // We perform our SQLite queries using the `sqlx::query!` macro.
                    //
                    // Here, we are inserting `task_description` into the `task` column and
                    // `user_id` into the `user_id` column of the `todo` table.
                    sqlx::query!(
                        "INSERT INTO todo (task, user_id) VALUES (?, ?)",
                        task_description,
                        user_id,
                    )
                    .execute(&self.database) // The query is executed here.
                    .await
                    .unwrap();

                    let response =
                        format!("Successfully added `{task_description}` to your todo list.");
                    new_message.channel_id.say(&ctx.http, response).await.unwrap();
                } else if let Some(task_index) = new_message.content.strip_prefix("~todo remove") {
                    let task_index = task_index.trim().parse::<i64>().unwrap() - 1;

                    // This SELECT query will select the `rowid` and `task` of the rows in the
                    // `todo` table where the value in the `user_id` column equals `user_id`, order
                    // the rows by `rowid`, skip the number of rows equal to `task_index`, and then
                    // return the next row.
                    let entry = sqlx::query!(
                        "SELECT rowid, task FROM todo WHERE user_id = ? ORDER BY rowid LIMIT 1 OFFSET ?",
                        user_id,
                        task_index,
                    )
                    .fetch_one(&self.database) // Only one row will be fetched.
                    .await
                    .unwrap();

                    // Delete every row in `todo` with a `rowid` that matches `entry.rowid`. This
                    // will only be one (the removed task) since `rowid` is a unique identifier.
                    sqlx::query!("DELETE FROM todo WHERE rowid = ?", entry.rowid)
                        .execute(&self.database)
                        .await
                        .unwrap();

                    let response = format!("Successfully completed `{}`!", entry.task);
                    new_message.channel_id.say(&ctx.http, response).await.unwrap();
                } else if new_message.content.trim() == "~todo list" {
                    // This SELECT query will select the `task` of the rows in the `todo` table
                    // where the value in the `user_id` column equals `user_id`, and then order the
                    // results by `rowid`.
                    let todos = sqlx::query!(
                        "SELECT task FROM todo WHERE user_id = ? ORDER BY rowid",
                        user_id
                    )
                    .fetch_all(&self.database) // All selected rows will be returned.
                    .await
                    .unwrap();

                    let mut response = format!("You have {} pending tasks:\n", todos.len());
                    for (i, todo) in todos.iter().enumerate() {
                        writeln!(response, "{}. {}", i + 1, todo.task).unwrap();
                    }

                    new_message.channel_id.say(&ctx.http, response).await.unwrap();
                }
            },
            FullEvent::Ready {
                data_about_bot, ..
            } => {
                println!("{} is connected!", data_about_bot.user.name);
            },
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() {
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a valid token in the environment");

    // Create a connection pool and immediately open a connection to the database, creating the
    // database file if necessary.
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    // Run migrations, which updates the database's schema to the latest version.
    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

    // Insert the connection pool into the event handler so that our bot can access the database
    // when events are dispatched.
    let handler = Arc::new(Handler {
        database,
    });

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(token, intents).event_handler(handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
