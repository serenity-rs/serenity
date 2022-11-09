use std::env;
use std::time::Duration;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{
    CreateButton,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    CreateMessage,
    CreateSelectMenu,
    CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;

fn sound_button(name: &str, emoji: ReactionType) -> CreateButton {
    // To add an emoji to buttons, use .emoji(). The method accepts anything ReactionType or
    // anything that can be converted to it. For a list of that, search Trait Implementations in the
    // docs for From<...>.
    CreateButton::new(name).emoji(emoji)
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "animal" {
            return;
        }

        // Ask the user for its favorite animal
        let m = msg
            .channel_id
            .send_message(
                &ctx,
                CreateMessage::new().content("Please select your favorite animal").select_menu(
                    CreateSelectMenu::new("animal_select", CreateSelectMenuKind::String {
                        options: vec![
                            CreateSelectMenuOption::new("🐈 meow", "Cat"),
                            CreateSelectMenuOption::new("🐕 woof", "Dog"),
                            CreateSelectMenuOption::new("🐎 neigh", "Horse"),
                            CreateSelectMenuOption::new("🦙 hoooooooonk", "Alpaca"),
                            CreateSelectMenuOption::new("🦀 crab rave", "Ferris"),
                        ],
                    })
                    .custom_id("animal_select")
                    .placeholder("No animal selected"),
                ),
            )
            .await
            .unwrap();

        // Wait for the user to make a selection
        // This uses a collector to wait for an incoming event without needing to listen for it
        // manually in the EventHandler.
        let interaction = match m
            .component_interaction_collector(&ctx.shard)
            .timeout(Duration::from_secs(60 * 3))
            .collect_single()
            .await
        {
            Some(x) => x,
            None => {
                m.reply(&ctx, "Timed out").await.unwrap();
                return;
            },
        };

        // data.values contains the selected value from each select menus. We only have one menu,
        // so we retrieve the first
        let animal = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect {
                values,
            } => &values[0],
            _ => panic!("unexpected interaction data kind"),
        };

        // Acknowledge the interaction and edit the message
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .content(format!("You chose: **{}**\nNow choose a sound!", animal))
                        .button(sound_button("meow", "🐈".parse().unwrap()))
                        .button(sound_button("woof", "🐕".parse().unwrap()))
                        .button(sound_button("neigh", "🐎".parse().unwrap()))
                        .button(sound_button("hoooooooonk", "🦙".parse().unwrap()))
                        .button(sound_button(
                            "crab rave",
                            // Custom emojis in Discord are represented with
                            // `<:EMOJI_NAME:EMOJI_ID>`. You can see this by
                            // posting an emoji in your server and putting a backslash
                            // before the emoji.
                            //
                            // Because ReactionType implements FromStr, we can use .parse()
                            // to convert the textual emoji representation to ReactionType
                            "<:ferris:381919740114763787>".parse().unwrap(),
                        )),
                ),
            )
            .await
            .unwrap();

        // Wait for multiple interactions
        let mut interaction_stream = m
            .component_interaction_collector(&ctx.shard)
            .timeout(Duration::from_secs(60 * 3))
            .collect_stream();

        while let Some(interaction) = interaction_stream.next().await {
            let sound = &interaction.data.custom_id;
            // Acknowledge the interaction and send a reply
            interaction
                .create_response(
                    &ctx,
                    // This time we dont edit the message but reply to it
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::default()
                            // Make the message hidden for other users by setting `ephemeral(true)`.
                            .ephemeral(true)
                            .content(format!("The **{}** says __{}__", animal, sound)),
                    ),
                )
                .await
                .unwrap();
        }

        // Delete the orig message or there will be dangling components (components that still
        // exist, but no collector is running so any user who presses them sees an error)
        m.delete(&ctx).await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
