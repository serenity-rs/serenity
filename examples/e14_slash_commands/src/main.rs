use std::env;

use serenity::async_trait;
use serenity::builder::{
    CreateApplicationCommand as CreateCommand,
    CreateApplicationCommandOption as CreateOption,
    CreateInteractionResponse,
    CreateInteractionResponseData,
};
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::{
    ResolvedOption,
    ResolvedValue,
};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "id" => {
                    if let Some(ResolvedOption {
                        value: ResolvedValue::User(user, _), ..
                    }) = command.data.options().get(0)
                    {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                },
                "attachmentinput" => {
                    if let Some(ResolvedOption {
                        value: ResolvedValue::Attachment(attachment),
                        ..
                    }) = command.data.options().get(0)
                    {
                        format!(
                            "Attachment name: {}, attachment size: {}",
                            attachment.filename, attachment.size
                        )
                    } else {
                        "Please provide a valid attachment".to_string()
                    }
                },
                _ => "not implemented :(".to_string(),
            };

            let data = CreateInteractionResponseData::new().content(content);
            let builder = CreateInteractionResponse::new()
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(data);
            if let Err(why) = command.create_interaction_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let c1 = CreateCommand::new("ping", "A ping command");

        let c2 = CreateCommand::new("id", "Get a user id").add_option(
            CreateOption::new(CommandOptionType::User, "id", "The user to lookup").required(true),
        );

        let c3 = CreateCommand::new("welcome", "Welcome a user")
            .name_localized("de", "begrüßen")
            .description_localized("de", "Einen Nutzer begrüßen")
            .add_option(
                CreateOption::new(CommandOptionType::User, "user", "The user to welcome")
                    .name_localized("de", "nutzer")
                    .description_localized("de", "Der zu begrüßende Nutzer")
                    .required(true),
            )
            .add_option(
                CreateOption::new(CommandOptionType::String, "message", "The message to send")
                    .name_localized("de", "nachricht")
                    .description_localized("de", "Die versendete Nachricht")
                    .required(true)
                    .add_string_choice_localized(
                        "Welcome to our cool server! Ask me if you need help",
                        "pizza",
                        [("de", "Willkommen auf unserem coolen Server! Frag mich, falls du Hilfe brauchst")],
                    )
                    .add_string_choice_localized(
                        "Hey, do you want a coffee?",
                        "coffee",
                        [("de", "Hey, willst du einen Kaffee?")]
                    )
                    .add_string_choice_localized(
                        "Welcome to the club, you're now a good person. Well, I hope.",
                        "club",
                        [("de", "Willkommen im Club, du bist jetzt ein guter Mensch. Naja, hoffentlich.")],
                    )
                    .add_string_choice_localized(
                        "I hope that you brought a controller to play together!",
                        "game",
                        [("de", "Ich hoffe du hast einen Controller zum Spielen mitgebracht!")],
                    ),
            );

        let c4 = CreateCommand::new("numberinput", "Test command for number input")
            .add_option(
                CreateOption::new(CommandOptionType::Integer, "int", "An integer fro 5 to 10")
                    .min_int_value(5)
                    .max_int_value(10)
                    .required(true),
            )
            .add_option(
                CreateOption::new(CommandOptionType::Number, "number", "A float from -3 to 234.5")
                    .min_number_value(-3.0)
                    .max_number_value(234.5)
                    .required(true),
            );

        let c5 = CreateCommand::new("attachmentinput", "Test command for attachment input")
            .add_option(
                CreateOption::new(CommandOptionType::Attachment, "attachment", "A file")
                    .required(true),
            );

        let commands = guild_id.set_application_commands(&ctx.http, vec![c1, c2, c3, c4, c5]).await;

        println!("I now have the following guild slash commands: {:#?}", commands);

        let guild_command = Command::create_global_application_command(
            &ctx.http,
            CreateCommand::new("wonderful_command", "An amazing command"),
        )
        .await;

        println!("I created the following global slash command: {:#?}", guild_command);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
