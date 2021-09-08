use serenity::{async_trait, model::prelude::*, prelude::*};

struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let user_id = msg.author.id.0 as i64;

        if let Some(task_description) = msg.content.strip_prefix("~todo add") {
            let task_description = task_description.trim();
            sqlx::query!(
                "INSERT INTO todo (task, user_id) VALUES (?, ?)",
                task_description,
                user_id,
            )
            .execute(&self.database)
            .await
            .unwrap();

            let response = format!("Successfully added `{}` to your todo list", task_description);
            msg.channel_id.say(&ctx, response).await.unwrap();
        } else if let Some(task_index) = msg.content.strip_prefix("~todo remove") {
            let task_index = task_index.trim().parse::<i64>().unwrap() - 1;

            let entry = sqlx::query!(
                "SELECT rowid, task FROM todo WHERE user_id = ? ORDER BY rowid LIMIT 1 OFFSET ?",
                user_id,
                task_index,
            )
            .fetch_one(&self.database)
            .await
            .unwrap();

            sqlx::query!("DELETE FROM todo WHERE rowid = ?", entry.rowid)
                .execute(&self.database)
                .await
                .unwrap();

            let response = format!("Successfully completed `{}`!", entry.task);
            msg.channel_id.say(&ctx, response).await.unwrap();
        } else if msg.content.trim() == "~todo list" {
            let todos =
                sqlx::query!("SELECT task FROM todo WHERE user_id = ? ORDER BY rowid", user_id)
                    .fetch_all(&self.database)
                    .await
                    .unwrap();

            let mut response = format!("You have {} pending tasks:\n", todos.len());
            for (i, todo) in todos.iter().enumerate() {
                response += &format!("{}. {}\n", i + 1, todo.task);
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

    let mut client = Client::builder(&token).event_handler(bot).await.expect("Err creating client");
    client.start().await.unwrap();
}
