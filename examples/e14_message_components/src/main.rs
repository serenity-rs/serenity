//! This example demonstrates how to use message components.
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

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
use serenity::collector::CollectComponentInteractions;
use serenity::futures::StreamExt;
use serenity::model::event::FullEvent;
use serenity::model::prelude::*;
use serenity::prelude::*;

// This is a helper function to reduce boilerplate when creating buttons.
fn sound_button(name: &str, emoji: ReactionType) -> CreateButton<'_> {
    // To add an emoji to a button, use `.emoji()`. The method accepts a `ReactionType` or any
    // other type that can be converted to it. For a list of such types, go to the `ReactionType`
    // documentation and look for `From<...>` under "Trait Implementations".
    CreateButton::new(name).emoji(emoji)
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message {
                new_message, ..
            } => {
                if new_message.content != "animal" {
                    return;
                }

                // We want to ask the user for their favorite animal. We use the `CreateMessage`
                // builder to create a message with a select menu and then we send the message.
                let m = new_message
                    .channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new()
                            .content("Please select your favorite animal.")
                            // The `select_menu()` convenience method allows us to attach a select
                            // menu without needing to manually create an action row component.
                            .select_menu(
                                CreateSelectMenu::new(
                                    "animal_select",
                                    CreateSelectMenuKind::String {
                                        options: Cow::Borrowed(&[
                                            CreateSelectMenuOption::new("🐈 meow", "Cat"),
                                            CreateSelectMenuOption::new("🐕 woof", "Dog"),
                                            CreateSelectMenuOption::new("🐎 neigh", "Horse"),
                                            CreateSelectMenuOption::new("🦙 hoooooooonk", "Alpaca"),
                                            CreateSelectMenuOption::new("🦀 crab rave", "Ferris"),
                                        ]),
                                    },
                                )
                                .custom_id("animal_select")
                                .placeholder("No animal selected"),
                            ),
                    )
                    .await
                    .unwrap();

                // Wait for the user to make a selection. This uses a collector to wait for an
                // interaction without needing to listen for it manually in the event handler.
                let interaction = match m
                    .id
                    .collect_component_interactions(ctx)
                    .timeout(Duration::from_mins(3))
                    .await
                {
                    Some(x) => x,
                    None => {
                        m.reply(&ctx.http, "Timed out").await.unwrap();
                        return;
                    },
                };

                // `data.values` contains the text of the selected options. Only one selection is
                // allowed by default, so take the first (and only) value.
                let animal = match &interaction.data.kind {
                    ComponentInteractionDataKind::StringSelect {
                        values,
                    } => &values[0],
                    _ => panic!("unexpected interaction data kind"),
                };

                // Acknowledge the interaction and edit the message.
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::default()
                                .content(format!("You chose: **{animal}**\nNow choose a sound!"))
                                .button(sound_button("meow", "🐈".parse().unwrap()))
                                .button(sound_button("woof", "🐕".parse().unwrap()))
                                .button(sound_button("neigh", "🐎".parse().unwrap()))
                                .button(sound_button("hoooooooonk", "🦙".parse().unwrap()))
                                .button(sound_button(
                                    "crab rave",
                                    // Custom emojis in Discord are represented with
                                    // `<:EMOJI_NAME:EMOJI_ID>`. You can verify this by selecting
                                    // a custom emoji in Discord and putting a backslash before it
                                    // in the chat box.
                                    //
                                    // Because ReactionType implements FromStr, we can use .parse()
                                    // to convert the textual emoji representation to ReactionType.
                                    "<:ferris:381919740114763787>".parse().unwrap(),
                                )),
                        ),
                    )
                    .await
                    .unwrap();

                // Wait for multiple interactions.
                let mut interaction_stream =
                    m.id.collect_component_interactions(ctx)
                        .timeout(Duration::from_mins(3))
                        .stream();

                while let Some(interaction) = interaction_stream.next().await {
                    // The `custom_id` is the `name` that we defined for our sounds using the
                    // `sound_button()` helper function.
                    let sound = &interaction.data.custom_id;
                    // Acknowledge the interaction and send a reply.
                    interaction
                        .create_response(
                            &ctx.http,
                            // This time we reply instead of editing our message.
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::default()
                                    // Hide the message from other users by setting
                                    // `ephemeral(true)`.
                                    .ephemeral(true)
                                    .content(format!("The **{animal}** says __{sound}__")),
                            ),
                        )
                        .await
                        .unwrap();
                }

                // Delete the original message or there will be dangling components, i.e., the
                // components are still visible, but since no collector is running, any user who
                // interacts with them will see an error.
                m.delete(&ctx.http, None).await.unwrap()
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
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(Handler))
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
