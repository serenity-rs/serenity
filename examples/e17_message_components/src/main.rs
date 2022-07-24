use std::error::Error as StdError;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};

use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::*;
use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::Message;
use serenity::prelude::*;

#[derive(Debug)]
enum Animal {
    Cat,
    Dog,
    Horse,
    Alpaca,
}

#[derive(Debug)]
struct ParseComponentError(String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl fmt::Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cat => write!(f, "Cat"),
            Self::Dog => write!(f, "Dog"),
            Self::Horse => write!(f, "Horse"),
            Self::Alpaca => write!(f, "Alpaca"),
        }
    }
}

impl FromStr for Animal {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Animal::Cat),
            "dog" => Ok(Animal::Dog),
            "horse" => Ok(Animal::Horse),
            "alpaca" => Ok(Animal::Alpaca),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

impl Animal {
    fn emoji(&self) -> char {
        match self {
            Self::Cat => '🐈',
            Self::Dog => '🐕',
            Self::Horse => '🐎',
            Self::Alpaca => '🦙',
        }
    }

    fn menu_option(&self) -> CreateSelectMenuOption {
        CreateSelectMenuOption::default()
            // This is what will be shown to the user
            .label(format!("{} {}", self.emoji(), self))
            // This is used to identify the selected value
            .value(self.to_string().to_ascii_lowercase())
    }

    fn select_menu() -> CreateSelectMenu {
        let options = CreateSelectMenuOptions::default()
            .add_option(Self::Cat.menu_option())
            .add_option(Self::Dog.menu_option())
            .add_option(Self::Horse.menu_option())
            .add_option(Self::Alpaca.menu_option());
        CreateSelectMenu::default()
            .custom_id("animal_select")
            .placeholder("No animal selected")
            .options(options)
    }

    fn action_row() -> CreateActionRow {
        // A select menu must be the only thing in an action row!
        CreateActionRow::default().add_select_menu(Self::select_menu())
    }
}

#[derive(Debug)]
enum Sound {
    Meow,
    Woof,
    Neigh,
    Honk,
}

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meow => write!(f, "meow"),
            Self::Woof => write!(f, "woof"),
            Self::Neigh => write!(f, "neigh"),
            Self::Honk => write!(f, "hoooooooonk"),
        }
    }
}

impl Sound {
    fn emoji(&self) -> char {
        match self {
            Self::Meow => Animal::Cat.emoji(),
            Self::Woof => Animal::Dog.emoji(),
            Self::Neigh => Animal::Horse.emoji(),
            Self::Honk => Animal::Alpaca.emoji(),
        }
    }

    fn button(&self) -> CreateButton {
        CreateButton::default()
            .custom_id(self.to_string().to_ascii_lowercase())
            .emoji(self.emoji())
            .label(self.to_string())
            .style(ButtonStyle::Primary)
    }

    fn action_row() -> CreateActionRow {
        // We can add up to 5 buttons per action row
        CreateActionRow::default()
            .add_button(Sound::Meow.button())
            .add_button(Sound::Woof.button())
            .add_button(Sound::Neigh.button())
            .add_button(Sound::Honk.button())
    }
}

impl FromStr for Sound {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meow" => Ok(Sound::Meow),
            "woof" => Ok(Sound::Woof),
            "neigh" => Ok(Sound::Neigh),
            "hoooooooonk" => Ok(Sound::Honk),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "animal" {
            return;
        }

        // Ask the user for its favorite animal
        let components = CreateComponents::default().add_action_row(Animal::action_row());
        let builder = CreateMessage::default()
            .content("Please select your favorite animal")
            .components(components);
        let m = msg.channel_id.send_message(&ctx, builder).await.unwrap();

        // Wait for the user to make a selection
        let mci = match m
            .component_interaction_collector(&ctx.shard)
            .timeout(Duration::from_secs(60 * 3))
            .collect_single()
            .await
        {
            Some(ci) => ci,
            None => {
                m.reply(&ctx, "Timed out").await.unwrap();
                return;
            },
        };

        // data.custom_id contains the id of the component (here "animal_select")
        // and should be used to identify if a message has multiple components.
        // data.values contains the selected values from the menu
        let animal = Animal::from_str(mci.data.values.get(0).unwrap()).unwrap();

        // Acknowledge the interaction and edit the message
        let components = CreateComponents::default().add_action_row(Sound::action_row());
        let data = CreateInteractionResponseData::default()
            .content(format!("You chose: **{}**\nNow choose a sound!", animal))
            .components(components);
        let builder = CreateInteractionResponse::default()
            .kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(data);
        mci.create_interaction_response(&ctx, builder).await.unwrap();

        // Wait for multiple interactions

        let mut cib = m
            .component_interaction_collector(&ctx.shard)
            .timeout(Duration::from_secs(60 * 3))
            .build();

        while let Some(mci) = cib.next().await {
            let sound = Sound::from_str(&mci.data.custom_id).unwrap();
            // Acknowledge the interaction and send a reply
            let data = CreateInteractionResponseData::default()
                .content(format!("The **{}** says __{}__", animal, sound))
                // Make the message hidden for other users by setting `ephemeral(true)`.
                .ephemeral(true);
            let builder = CreateInteractionResponse::default()
                // This time we don't edit the message, but reply to it.
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(data);
            mci.create_interaction_response(&ctx, builder).await.unwrap();
        }

        // Delete the orig message or there will be dangling components
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
