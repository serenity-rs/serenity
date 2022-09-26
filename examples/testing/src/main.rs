use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

const IMAGE_URL: &str = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    if msg.content == "edit" {
        let mut msg = msg
            .channel_id
            .send_message(
                &ctx,
                CreateMessage::new().add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
            )
            .await?;
        // Pre-PR, this falsely triggered a MODEL_TYPE_CONVERT Discord error
        msg.edit(&ctx, EditMessage::new().add_existing_attachment(msg.attachments[0].id)).await?;
    } else {
        return Ok(());
    }

    msg.react(&ctx, '✅').await?;
    Ok(())
}

struct Handler;
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        message(&ctx, msg).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    Client::builder(token, intents).event_handler(Handler).await?.start().await
}
