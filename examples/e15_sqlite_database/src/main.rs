// It is recommended that you read the README file, it is very important to this example.
// This example will help us to use a sqlite database with our bot.
use std::fmt::Write as _;

use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let user_id = msg.author.id.get() as i64;

        if let Some(task_description) = msg.content.strip_prefix("~todo add") {
            let task_description = task_description.trim();
            // That's how we are going to use a sqlite command.
            // We are inserting into the todo table, our task_description in task column and our
            // user_id in user_Id column.
            sqlx::query!(
                "INSERT INTO todo (task, user_id) VALUES (?, ?)",
                task_description,
                user_id,
            )
            .execute(&self.database) // < Where the command will be executed
            .await
            .unwrap();

            let response = format!("Successfully added `{task_description}` to your todo list");
            msg.channel_id.say(&ctx, response).await.unwrap();
        } else if let Some(task_index) = msg.content.strip_prefix("~todo remove") {
            let task_index = task_index.trim().parse::<i64>().unwrap() - 1;

            // "SELECT" will return the rowid of the todo rows where the user_Id column = user_id.
            let entry = sqlx::query!(
                "SELECT rowid, task FROM todo WHERE user_id = ? ORDER BY rowid LIMIT 1 OFFSET ?",
                user_id,
                task_index,
            )
            .fetch_one(&self.database) // < Just one data will be sent to entry
            .await
            .unwrap();

            // Every todo row with rowid column = entry.rowid will be deleted.
            sqlx::query!("DELETE FROM todo WHERE rowid = ?", entry.rowid)
                .execute(&self.database)
                .await
                .unwrap();

            let response = format!("Successfully completed `{}`!", entry.task);
            msg.channel_id.say(&ctx, response).await.unwrap();
        } else if msg.content.trim() == "~todo list" {
            // "SELECT" will return the task of all rows where user_Id column = user_id in todo.
            let todos = sqlx::query!("SELECT task FROM todo WHERE user_id = ? ORDER BY rowid", user_id)
                    .fetch_all(&self.database) // < All matched data will be sent to todos
                    .await
                    .unwrap();

            let mut response = format!("You have {} pending tasks:\n", todos.len());
            for (i, todo) in todos.iter().enumerate() {
                writeln!(response, "{}. {}", i + 1, todo.task).unwrap();
            }

            msg.channel_id.say(&ctx, response).await.unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Initiate a connection to the database file, creating the file if required.
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

    let bot = Bot {
        database,
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(&token, intents).event_handler(bot).await.expect("Err creating client");
    client.start().await.unwrap();
}
