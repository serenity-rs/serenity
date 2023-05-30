use std::env;
use std::time::Duration;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{CreateButton, CreateChannelSelect, SelectChannelType};
use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::application::component::ButtonStyle;
use serenity::model::prelude::*;
use serenity::prelude::*;

fn sound_button(name: &str, emoji: ReactionType) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(name);
    // To add an emoji to buttons, use .emoji(). The method accepts anything ReactionType or
    // anything that can be converted to it. For a list of that, search Trait Implementations in the
    // docs for From<...>.
    b.emoji(emoji);
    b.label(name);
    b.style(ButtonStyle::Primary);
    b
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let select_ids = &interaction.as_message_component().unwrap().data.values;
        let custom_id_found_for_form: &str =
            &interaction.as_message_component().unwrap().data.custom_id.as_ref();

        let mut temp = String::from(format!(
            "Interaction with component with \"custom_id\": {:?}. Selections: {:?}",
            custom_id_found_for_form, select_ids
        ));

        match custom_id_found_for_form {
            "user_sel" => {
                let mut users: Vec<UserId> = Vec::new();

                for user in select_ids {
                    let user: u64 = user.parse().unwrap();

                    let current_selection = UserId::from(user);

                    users.push(current_selection);
                }

                temp.push_str(&format!("{:?}", users));
            },

            "role_sel" => {
                let mut roles_selected: Vec<RoleId> = Vec::new();

                for role in select_ids {
                    let role: u64 = role.parse().unwrap();

                    let current_selection = RoleId::from(role);

                    roles_selected.push(current_selection);
                }

                temp.push_str(&format!("{:?}", roles_selected));
            },

            "mention_sel" => {
                let mut mentions: Vec<Mention> = Vec::new();

                for mention in select_ids {
                    let mention: u64 = mention.parse().unwrap();

                    // Mention select seems partially unsupported by serenity-rs. It can be converted
                    // into the correct type, but the type itself does not look like it currently supports
                    // detecting whether a @role or @user was mentioned.
                    //
                    // This detection will likely need to be done manually.

                    let current_selection = RoleId::from(mention);
                    let current_selection = Mention::from(current_selection);

                    mentions.push(current_selection);
                }

                temp.push_str(&format!("{:?}", mentions));
            },

            "channel_sel" => {
                let mut channels: Vec<ChannelId> = Vec::new();

                for channel in select_ids {
                    let channel: u64 = channel.parse().unwrap();

                    let current_selection = ChannelId::from(channel);

                    channels.push(current_selection);
                }

                temp.push_str(&format!("{:?}", channels));
            },

            _ => {},
        }

        let resp_res = interaction
            .into_message_component()
            .unwrap()
            .create_interaction_response(ctx, |interaction_resp| {
                interaction_resp.interaction_response_data(|resp_data| resp_data.content(temp))
            })
            .await;

        match resp_res {
            Ok(_) => {},
            Err(e) => {
                println!("error responding to interaction: {:?}", e)
            },
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!mention_select" {
            let select_sent = msg
                .channel_id
                .send_message(&ctx.http, |message| {
                    message.reference_message(&msg).components(|comp| {
                        comp.create_action_row(|row| {
                            row.create_mentionable_select(|mention_select| {
                                mention_select.custom_id("mention_sel");

                                mention_select
                            });

                            row
                        })
                    });

                    return message;
                })
                .await;

            match select_sent {
                Ok(_resp) => {
                    // println!("Ok: {:?}", resp);
                },
                Err(resp) => {
                    println!("Error: {:?}", resp);
                },
            }
        }

        if msg.content == "!user_select" {
            let select_sent = msg
                .channel_id
                .send_message(&ctx.http, |message| {
                    message.reference_message(&msg).components(|comp| {
                        comp.create_action_row(|row| {
                            row.create_user_select(|user_select| {
                                user_select.custom_id("user_sel");

                                user_select
                            });

                            row
                        })
                    });

                    return message;
                })
                .await;

            match select_sent {
                Ok(_resp) => {},
                Err(resp) => {
                    println!("Error: {:?}", resp);
                },
            }
        }

        if msg.content == "!role_select" {
            let select_sent = msg
                .channel_id
                .send_message(&ctx.http, |message| {
                    message.reference_message(&msg).components(|comp| {
                        comp.create_action_row(|row| {
                            row.create_role_select(|role_select| {
                                role_select.custom_id("role_sel").min_values(0).max_values(2);

                                role_select
                            });

                            row
                        })
                    });

                    return message;
                })
                .await;

            match select_sent {
                Ok(_resp) => {
                    // println!("Ok: {:?}", resp);
                },
                Err(resp) => {
                    println!("Error: {:?}", resp);
                },
            }
        }

        if msg.content == "!channel_select" {
            let select_sent = msg
                .channel_id
                .send_message(&ctx.http, |message| {
                    message.reference_message(&msg).components(|comp| {
                        comp.create_action_row(|row| {
                            row.create_channel_select(
                                |channel_select: &mut CreateChannelSelect| {
                                    channel_select.custom_id("channel_sel");

                                    let mut only_use: Vec<SelectChannelType> = Vec::new();

                                    only_use.push(SelectChannelType::GuildVoice);

                                    // Only use voice channels in the channel selection
                                    channel_select.channel_types(only_use);

                                    channel_select
                                },
                            );

                            row
                        })
                    });

                    return message;
                })
                .await;

            match select_sent {
                Ok(_resp) => {
                    // println!("Ok: {:?}", resp);
                },
                Err(resp) => {
                    println!("Error: {:?}", resp);
                },
            }
        }

        if msg.content != "animal" {
            return;
        }

        // Ask the user for its favorite animal
        let m = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.content("Please select your favorite animal").components(|c| {
                    c.create_action_row(|row| {
                        // An action row can only contain one select menu!
                        row.create_select_menu(|menu| {
                            menu.custom_id("animal_select");
                            menu.placeholder("No animal selected");
                            menu.options(|f| {
                                f.create_option(|o| o.label("🐈 meow").value("Cat"));
                                f.create_option(|o| o.label("🐕 woof").value("Dog"));
                                f.create_option(|o| o.label("🐎 neigh").value("Horse"));
                                f.create_option(|o| o.label("🦙 hoooooooonk").value("Alpaca"));
                                f.create_option(|o| o.label("🦀 crab rave").value("Ferris"))
                            })
                        })
                    })
                })
            })
            .await
            .unwrap();

        // Wait for the user to make a selection
        // This uses a collector to wait for an incoming event without needing to listen for it
        // manually in the EventHandler.
        let interaction =
            match m.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 3)).await {
                Some(x) => x,
                None => {
                    m.reply(&ctx, "Timed out").await.unwrap();
                    return;
                },
            };

        // data.values contains the selected value from each select menus. We only have one menu,
        // so we retrieve the first
        let animal = &interaction.data.values[0];

        // Acknowledge the interaction and edit the message
        interaction
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
                    d.content(format!("You chose: **{}**\nNow choose a sound!", animal)).components(
                        |c| {
                            c.create_action_row(|r| {
                                // add_XXX methods are an alternative to create_XXX methods
                                r.add_button(sound_button("meow", "🐈".parse().unwrap()));
                                r.add_button(sound_button("woof", "🐕".parse().unwrap()));
                                r.add_button(sound_button("neigh", "🐎".parse().unwrap()));
                                r.add_button(sound_button("hoooooooonk", "🦙".parse().unwrap()));
                                r.add_button(sound_button(
                                    "crab rave",
                                    // Custom emojis in Discord are represented with
                                    // `<:EMOJI_NAME:EMOJI_ID>`. You can see this by
                                    // posting an emoji in your server and putting a backslash
                                    // before the emoji.
                                    //
                                    // Because ReactionType implements FromStr, we can use .parse()
                                    // to convert the textual emoji representation to ReactionType
                                    "<:ferris:381919740114763787>".parse().unwrap(),
                                ))
                            })
                        },
                    )
                })
            })
            .await
            .unwrap();

        // Wait for multiple interactions
        let mut interaction_stream =
            m.await_component_interactions(&ctx).timeout(Duration::from_secs(60 * 3)).build();

        while let Some(interaction) = interaction_stream.next().await {
            let sound = &interaction.data.custom_id;
            // Acknowledge the interaction and send a reply
            interaction
                .create_interaction_response(&ctx, |r| {
                    // This time we dont edit the message but reply to it
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            // Make the message hidden for other users by setting `ephemeral(true)`.
                            d.ephemeral(true)
                                .content(format!("The **{}** says __{}__", animal, sound))
                        })
                })
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
