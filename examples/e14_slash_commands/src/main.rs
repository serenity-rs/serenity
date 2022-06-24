use std::env;

use serenity::async_trait;
use serenity::builder::*;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
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
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let CommandDataOptionValue::User(user, _member) = options {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                },
                "attachmentinput" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected attachment option")
                        .resolved
                        .as_ref()
                        .expect("Expected attachment object");

                    if let CommandDataOptionValue::Attachment(attachment) = options {
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

            let data = CreateInteractionResponseData::default().content(content);
            if let Err(why) = command
                .create_interaction_response()
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(data)
                .execute(&ctx.http)
                .await
            {
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

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command.name("id").description("Get a user id").create_option(|option| {
                        option
                            .name("id")
                            .description("The user to lookup")
                            .kind(CommandOptionType::User)
                            .required(true)
                    })
                })
                .create_application_command(|command| {
                    command
                        .name("welcome")
                        .name_localized("de", "begrüßen")
                        .description("Welcome a user")
                        .description_localized("de", "Einen Nutzer begrüßen")
                        .create_option(|option| {
                            option
                                .name("user")
                                .name_localized("de", "nutzer")
                                .description("The user to welcome")
                                .description_localized("de", "Der zu begrüßende Nutzer")
                                .kind(CommandOptionType::User)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("message")
                                .name_localized("de", "nachricht")
                                .description("The message to send")
                                .description_localized("de", "Die versendete Nachricht")
                                .kind(CommandOptionType::String)
                                .required(true)
                                .add_string_choice_localized(
                                    "Welcome to our cool server! Ask me if you need help",
                                    "pizza",
                                    [("de", "Willkommen auf unserem coolen Server! Frag mich, falls du Hilfe brauchst")]
                                )
                                .add_string_choice_localized(
                                    "Hey, do you want a coffee?",
                                    "coffee",
                                    [("de", "Hey, willst du einen Kaffee?")],
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
                                )
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("numberinput")
                        .description("Test command for number input")
                        .create_option(|option| {
                            option
                                .name("int")
                                .description("An integer from 5 to 10")
                                .kind(CommandOptionType::Integer)
                                .min_int_value(5)
                                .max_int_value(10)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("number")
                                .description("A float from -3.3 to 234.5")
                                .kind(CommandOptionType::Number)
                                .min_number_value(-3.3)
                                .max_number_value(234.5)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("attachmentinput")
                        .description("Test command for attachment input")
                        .create_option(|option| {
                            option
                                .name("attachment")
                                .description("A file")
                                .kind(CommandOptionType::Attachment)
                                .required(true)
                        })
                })
        })
        .await;

        println!("I now have the following guild slash commands: {:#?}", commands);

        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            command.name("wonderful_command").description("An amazing command")
        })
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
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
