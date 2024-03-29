use std::borrow::Cow;

use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

mod model_type_sizes;

const IMAGE_URL: &str = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";
const IMAGE_URL_2: &str = "https://rustacean.net/assets/rustlogo.png";

async fn message(ctx: &Context, msg: &Message) -> Result<(), serenity::Error> {
    let channel_id = msg.channel_id;
    let guild_id = msg.guild_id.unwrap();
    if let Some(_args) = msg.content.strip_prefix("testmessage ") {
        println!("command message: {msg:#?}");
    } else if msg.content == "register" {
        guild_id
            .create_command(
                &ctx.http,
                CreateCommand::new("editattachments").description("test command"),
            )
            .await?;
        guild_id
            .create_command(
                &ctx.http,
                CreateCommand::new("unifiedattachments1").description("test command"),
            )
            .await?;
        guild_id
            .create_command(
                &ctx.http,
                CreateCommand::new("unifiedattachments2").description("test command"),
            )
            .await?;
        guild_id
            .create_command(&ctx.http, CreateCommand::new("editembeds").description("test command"))
            .await?;
        guild_id
            .create_command(
                &ctx.http,
                CreateCommand::new("newselectmenu").description("test command"),
            )
            .await?;
        guild_id
            .create_command(
                &ctx.http,
                CreateCommand::new("autocomplete").description("test command").add_option(
                    CreateCommandOption::new(CommandOptionType::String, "foo", "foo")
                        .set_autocomplete(true),
                ),
            )
            .await?;
    } else if msg.content == "edit" {
        let mut msg = channel_id
            .send_message(
                &ctx,
                CreateMessage::new()
                    .add_file(CreateAttachment::url(&ctx.http, IMAGE_URL, "testing.png").await?),
            )
            .await?;
        // Pre-PR, this falsely triggered a MODEL_TYPE_CONVERT Discord error
        msg.edit(&ctx, EditMessage::new().attachments(EditAttachments::keep_all(&msg))).await?;
    } else if msg.content == "unifiedattachments" {
        let mut msg = channel_id.send_message(ctx, CreateMessage::new().content("works")).await?;
        msg.edit(ctx, EditMessage::new().content("works still")).await?;

        let mut msg = channel_id
            .send_message(
                ctx,
                CreateMessage::new()
                    .add_file(CreateAttachment::url(&ctx.http, IMAGE_URL, "testing.png").await?),
            )
            .await?;
        msg.edit(
            ctx,
            EditMessage::new().attachments(
                EditAttachments::keep_all(&msg)
                    .add(CreateAttachment::url(&ctx.http, IMAGE_URL_2, "testing1.png").await?),
            ),
        )
        .await?;
    } else if msg.content == "ranking" {
        model_type_sizes::print_ranking();
    } else if msg.content == "auditlog" {
        // Test special characters in audit log reason
        msg.channel_id
            .edit(
                ctx,
                EditChannel::new().name("new-channel-name").audit_log_reason("hello\nworld\n🙂"),
            )
            .await?;
    } else if msg.content == "actionrow" {
        channel_id
            .send_message(
                ctx,
                CreateMessage::new()
                    .button(CreateButton::new("0").label("Foo"))
                    .button(CreateButton::new("1").emoji('🤗').style(ButtonStyle::Secondary))
                    .button(
                        CreateButton::new_link("https://google.com").emoji('🔍').label("Search"),
                    )
                    .select_menu(CreateSelectMenu::new("3", CreateSelectMenuKind::String {
                        options: Cow::Borrowed(&[
                            CreateSelectMenuOption::new("foo", "foo"),
                            CreateSelectMenuOption::new("bar", "bar"),
                        ]),
                    })),
            )
            .await?;
    } else if msg.content == "manybuttons" {
        let mut custom_id = msg.id.to_string();
        loop {
            let msg = channel_id
                .send_message(
                    ctx,
                    CreateMessage::new()
                        .button(CreateButton::new(custom_id.clone()).label(custom_id)),
                )
                .await?;
            let button_press = msg
                .await_component_interaction(ctx.shard.clone())
                .timeout(std::time::Duration::from_secs(10))
                .await;
            match button_press {
                Some(x) => x.defer(&ctx.http).await?,
                None => break,
            }

            custom_id = msg.id.to_string();
        }
    } else if msg.content == "reactionremoveemoji" {
        // Test new ReactionRemoveEmoji gateway event: https://github.com/serenity-rs/serenity/issues/2248
        msg.react(ctx, '👍').await?;
        msg.delete_reaction_emoji(ctx, '👍').await?;
    } else if msg.content == "testautomodregex" {
        guild_id
            .create_automod_rule(
                &ctx.http,
                EditAutoModRule::new().trigger(Trigger::Keyword {
                    strings: vec!["badword".into()],
                    regex_patterns: vec!["b[o0]{2,}b(ie)?s?".into()],
                    allow_list: vec!["bob".into()],
                }),
            )
            .await?;
        println!("new automod rules: {:?}", guild_id.automod_rules(&ctx.http).await?);
    } else if let Some(user_id) = msg.content.strip_prefix("ban ") {
        // Test if banning without a reason actually works
        let user_id: UserId = user_id.trim().parse().unwrap();
        guild_id.ban(&ctx.http, user_id, 0, None).await?;
    } else if msg.content == "createtags" {
        channel_id
            .edit(
                &ctx,
                EditChannel::new().available_tags(vec![
                    CreateForumTag::new("tag1 :)").emoji('👍'),
                    CreateForumTag::new("tag2 (:").moderated(true),
                ]),
            )
            .await?;
    } else if msg.content == "assigntags" {
        let forum_id = channel_id.to_channel(ctx).await?.guild().unwrap().parent_id.unwrap();
        let forum = forum_id.to_channel(ctx).await?.guild().unwrap();
        channel_id
            .edit_thread(
                &ctx.http,
                EditThread::new()
                    .applied_tags(forum.available_tags.iter().map(|t| t.id).collect::<Vec<_>>()),
            )
            .await?;
    } else if msg.content == "embedrace" {
        use serenity::futures::StreamExt;
        use tokio::time::Duration;

        let mut msg = channel_id
            .say(ctx, format!("https://codereview.stackexchange.com/questions/260653/very-slow-discord-bot-to-play-music{}", msg.id))
            .await?;

        let msg_id = msg.id;
        let mut message_updates = serenity::collector::collect(&ctx.shard, move |ev| match ev {
            Event::MessageUpdate(x) if x.id == msg_id => Some(()),
            _ => None,
        });
        let _ = tokio::time::timeout(Duration::from_millis(2000), message_updates.next()).await;
        msg.edit(&ctx, EditMessage::new().suppress_embeds(true)).await?;
    } else if msg.content == "voicemessage" {
        let audio_url =
            "https://upload.wikimedia.org/wikipedia/commons/8/81/Short_Silent%2C_Empty_Audio.ogg";
        // As of 2023-04-20, bots are still not allowed to sending voice messages
        let builder = CreateMessage::new()
            .flags(MessageFlags::IS_VOICE_MESSAGE)
            .add_file(CreateAttachment::url(&ctx.http, audio_url, "testing.ogg").await?);

        msg.author.dm(&ctx.http, builder).await?;
    } else if let Some(channel) = msg.content.strip_prefix("movetorootandback") {
        let mut channel =
            channel.trim().parse::<ChannelId>().unwrap().to_channel(ctx).await?.guild().unwrap();
        let parent_id = channel.parent_id.unwrap();
        channel.edit(ctx, EditChannel::new().category(None)).await?;
        channel.edit(ctx, EditChannel::new().category(Some(parent_id))).await?;
    } else if msg.content == "channelperms" {
        channel_id.say(ctx, format!("{:?}", msg.author_permissions(&ctx.cache))).await?;
    } else if let Some(forum_channel_id) = msg.content.strip_prefix("createforumpostin ") {
        forum_channel_id
            .parse::<ChannelId>()
            .unwrap()
            .create_forum_post(
                &ctx.http,
                CreateForumPost::new(
                    "a",
                    CreateMessage::new()
                        .add_file(CreateAttachment::bytes(b"Hallo welt!".as_slice(), "lul.txt")),
                ),
            )
            .await?;
    } else if let Some(forum_post_url) = msg.content.strip_prefix("deleteforumpost ") {
        let (_guild_id, channel_id, _message_id) =
            serenity::utils::parse_message_url(forum_post_url).unwrap();
        msg.channel_id.say(ctx, format!("Deleting <#{}> in 10 seconds...", channel_id)).await?;
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        channel_id.delete(&ctx.http, None).await?;
    } else {
        return Ok(());
    }

    msg.react(&ctx, '✅').await?;
    Ok(())
}

async fn interaction(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<(), serenity::Error> {
    if interaction.data.name == "editattachments" {
        // Respond with an image
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().add_file(
                        CreateAttachment::url(&ctx.http, IMAGE_URL, "testing.png").await?,
                    ),
                ),
            )
            .await?;

        // We need to know the attachments' IDs in order to not lose them in the subsequent edit
        let msg = interaction.get_response(&ctx.http).await?;

        // Add another image
        let msg = interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().attachments(
                    EditAttachments::keep_all(&msg)
                        .add(CreateAttachment::url(&ctx.http, IMAGE_URL_2, "testing1.png").await?),
                ),
            )
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Only keep the new image, removing the first image
        let _msg = interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new()
                    .attachments(EditAttachments::new().keep(msg.attachments[1].id)),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments1" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("works"),
                ),
            )
            .await?;

        interaction
            .edit_response(&ctx.http, EditInteractionResponse::new().content("works still"))
            .await?;

        interaction
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new().content("still works still"),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments2" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().add_file(
                        CreateAttachment::url(&ctx.http, IMAGE_URL, "testing.png").await?,
                    ),
                ),
            )
            .await?;

        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().new_attachment(
                    CreateAttachment::url(&ctx.http, IMAGE_URL_2, "testing1.png").await?,
                ),
            )
            .await?;

        interaction
            .create_followup(
                &ctx.http,
                CreateInteractionResponseFollowup::new()
                    .add_file(CreateAttachment::url(&ctx.http, IMAGE_URL, "testing.png").await?),
            )
            .await?;
    } else if interaction.data.name == "editembeds" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("hi")
                        .embed(CreateEmbed::new().description("hi")),
                ),
            )
            .await?;

        // Pre-PR, this falsely deleted the embed
        interaction.edit_response(&ctx.http, EditInteractionResponse::new()).await?;
    } else if interaction.data.name == "newselectmenu" {
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .select_menu(CreateSelectMenu::new("0", CreateSelectMenuKind::String {
                            options: Cow::Borrowed(&[
                                CreateSelectMenuOption::new("foo", "foo"),
                                CreateSelectMenuOption::new("bar", "bar"),
                            ]),
                        }))
                        .select_menu(CreateSelectMenu::new(
                            "1",
                            CreateSelectMenuKind::Mentionable {
                                default_users: None,
                                default_roles: None,
                            },
                        ))
                        .select_menu(CreateSelectMenu::new("2", CreateSelectMenuKind::Role {
                            default_roles: None,
                        }))
                        .select_menu(CreateSelectMenu::new("3", CreateSelectMenuKind::User {
                            default_users: None,
                        }))
                        .select_menu(CreateSelectMenu::new("4", CreateSelectMenuKind::Channel {
                            channel_types: None,
                            default_channels: None,
                        })),
                ),
            )
            .await?;
    }

    Ok(())
}

struct Handler;
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: &Context, msg: &Message) {
        message(ctx, msg).await.unwrap();
    }

    async fn interaction_create(&self, ctx: &Context, i: &Interaction) {
        match i {
            Interaction::Command(i) => interaction(ctx, i).await.unwrap(),
            Interaction::Component(i) => println!("{:#?}", i.data),
            Interaction::Autocomplete(i) => {
                i.create_response(
                    &ctx.http,
                    CreateInteractionResponse::Autocomplete(
                        CreateAutocompleteResponse::new()
                            .add_string_choice("suggestion", "suggestion"),
                    ),
                )
                .await
                .unwrap();
            },
            _ => {},
        }
    }

    async fn reaction_remove_emoji(&self, _ctx: &Context, removed_reactions: &Reaction) {
        println!("Got ReactionRemoveEmoji event: {removed_reactions:?}");
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "--print-sizes" {
            model_type_sizes::print_ranking();
            return Ok(());
        }
    }

    env_logger::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    Client::builder(&token, intents).event_handler(Handler).await?.start().await
}
