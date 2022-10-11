use serenity::model::prelude::interaction::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    if let Some(_args) = msg.content.strip_prefix("testmessage ") {
        println!("command message: {:#?}", msg);
    } else {
        return Ok(());
    }

    msg.react(&ctx, '✅').await?;
    Ok(())
}

async fn interaction(
    _ctx: &Context,
    _interaction: ApplicationCommandInteraction,
) -> Result<(), serenity::Error> {
    Ok(())
}

struct Handler;
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        message(&ctx, msg).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, i: Interaction) {
        if let Interaction::ApplicationCommand(i) = i {
            interaction(&ctx, i).await.unwrap();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    Client::builder(token, intents).event_handler(Handler).await?.start().await
}
