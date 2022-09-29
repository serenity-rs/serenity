use serenity::builder::*;
use serenity::model::prelude::command::*;
use serenity::model::prelude::interaction::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    if let Some(_args) = msg.content.strip_prefix("testmessage ") {
        println!("command message: {:#?}", msg);
    } else if msg.content == "globalcommand" {
        // Tests https://github.com/serenity-rs/serenity/issues/2259
        // Activate simd_json feature for this
        Command::create_global_application_command(
            &ctx,
            CreateApplicationCommand::new("ping").description("test command"),
        )
        .await?;
    } else if msg.content == "register" {
        let guild_id = msg.guild_id.unwrap();
        guild_id
            .create_application_command(
                &ctx,
                CreateApplicationCommand::new("editembeds").description("test command"),
            )
            .await?;
    } else {
        return Ok(());
    }

    msg.react(&ctx, '✅').await?;
    Ok(())
}

async fn interaction(
    ctx: &Context,
    interaction: ApplicationCommandInteraction,
) -> Result<(), serenity::Error> {
    if interaction.data.name == "editembeds" {
        interaction
            .create_interaction_response(
                &ctx,
                CreateInteractionResponse::new().interaction_response_data(
                    CreateInteractionResponseData::new()
                        .content("hi")
                        .embed(CreateEmbed::new().description("hi")),
                ),
            )
            .await?;

        // Pre-PR, this falsely deleted the embed
        interaction
            .edit_original_interaction_response(&ctx, EditInteractionResponse::new())
            .await?;
    }

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
