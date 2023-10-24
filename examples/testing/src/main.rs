use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

mod model_type_sizes;

const IMAGE_URL: &str = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";
const IMAGE_URL_2: &str = "https://rustacean.net/assets/rustlogo.png";

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    let channel_id = msg.channel_id;
    let guild_id = msg.guild_id.unwrap();
    if let Some(_args) = msg.content.strip_prefix("testmessage ") {
        println!("command message: {msg:#?}");
    } else if msg.content == "globalcommand" {
        // Tests https://github.com/serenity-rs/serenity/issues/2259
        // Activate simd_json feature for this
        Command::create_global_command(
            &ctx,
            CreateCommand::new("ping").description("test command"),
        )
        .await?;
    } else if msg.content == "register" {
        guild_id
            .create_command(&ctx, CreateCommand::new("editattachments").description("test command"))
            .await?;
        guild_id
            .create_command(
                &ctx,
                CreateCommand::new("unifiedattachments1").description("test command"),
            )
            .await?;
        guild_id
            .create_command(
                &ctx,
                CreateCommand::new("unifiedattachments2").description("test command"),
            )
            .await?;
        guild_id
            .create_command(&ctx, CreateCommand::new("editembeds").description("test command"))
            .await?;
        guild_id
            .create_command(&ctx, CreateCommand::new("newselectmenu").description("test command"))
            .await?;
        guild_id
            .create_command(
                &ctx,
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
                CreateMessage::new().add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
            )
            .await?;
        // Pre-PR, this falsely triggered a MODEL_TYPE_CONVERT Discord error
        msg.edit(&ctx, EditMessage::new().add_existing_attachment(msg.attachments[0].id)).await?;
    } else if msg.content == "unifiedattachments" {
        let mut msg = channel_id.send_message(ctx, CreateMessage::new().content("works")).await?;
        msg.edit(ctx, EditMessage::new().content("works still")).await?;

        let mut msg = channel_id
            .send_message(
                ctx,
                CreateMessage::new().add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
            )
            .await?;
        msg.edit(
            ctx,
            EditMessage::new()
                .attachment(CreateAttachment::url(ctx, IMAGE_URL_2).await?)
                .add_existing_attachment(msg.attachments[0].id),
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
                        options: vec![
                            CreateSelectMenuOption::new("foo", "foo"),
                            CreateSelectMenuOption::new("bar", "bar"),
                        ],
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
                .await_component_interaction(&ctx.shard)
                .timeout(std::time::Duration::from_secs(10))
                .await;
            match button_press {
                Some(x) => x.defer(ctx).await?,
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
                ctx,
                EditAutoModRule::new().trigger(Trigger::Keyword {
                    strings: vec!["badword".into()],
                    regex_patterns: vec!["b[o0]{2,}b(ie)?s?".into()],
                }),
            )
            .await?;
        println!("new automod rules: {:?}", guild_id.automod_rules(ctx).await?);
    } else if let Some(user_id) = msg.content.strip_prefix("ban ") {
        // Test if banning without a reason actually works
        guild_id.ban(ctx, UserId(user_id.trim().parse().unwrap()), 0).await?;
    } else {
        return Ok(());
    }

    msg.react(&ctx, '✅').await?;
    Ok(())
}

async fn interaction(
    ctx: &Context,
    interaction: CommandInteraction,
) -> Result<(), serenity::Error> {
    if interaction.data.name == "editattachments" {
        // Respond with an image
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
                ),
            )
            .await?;

        // We need to know the attachments' IDs in order to not lose them in the subsequent edit
        let msg = interaction.get_response(ctx).await?;

        // Add another image
        let msg = interaction
            .edit_response(
                &ctx,
                EditInteractionResponse::new()
                    .new_attachment(CreateAttachment::url(ctx, IMAGE_URL_2).await?)
                    .keep_existing_attachment(msg.attachments[0].id),
            )
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Only keep the new image, removing the first image
        let _msg = interaction
            .edit_response(
                &ctx,
                EditInteractionResponse::new()
                    .clear_existing_attachments()
                    .keep_existing_attachment(msg.attachments[1].id),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments1" {
        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("works"),
                ),
            )
            .await?;

        interaction
            .edit_response(ctx, EditInteractionResponse::new().content("works still"))
            .await?;

        interaction
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new().content("still works still"),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments2" {
        interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
                ),
            )
            .await?;

        interaction
            .edit_response(
                ctx,
                EditInteractionResponse::new()
                    .new_attachment(CreateAttachment::url(ctx, IMAGE_URL_2).await?),
            )
            .await?;

        interaction
            .create_followup(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
            )
            .await?;
    } else if interaction.data.name == "editembeds" {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("hi")
                        .embed(CreateEmbed::new().description("hi")),
                ),
            )
            .await?;

        // Pre-PR, this falsely deleted the embed
        interaction.edit_response(&ctx, EditInteractionResponse::new()).await?;
    } else if interaction.data.name == "newselectmenu" {
        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .select_menu(CreateSelectMenu::new("0", CreateSelectMenuKind::String {
                            options: vec![
                                CreateSelectMenuOption::new("foo", "foo"),
                                CreateSelectMenuOption::new("bar", "bar"),
                            ],
                        }))
                        .select_menu(CreateSelectMenu::new("1", CreateSelectMenuKind::Mentionable))
                        .select_menu(CreateSelectMenu::new("2", CreateSelectMenuKind::Role))
                        .select_menu(CreateSelectMenu::new("3", CreateSelectMenuKind::User))
                        .select_menu(CreateSelectMenu::new("4", CreateSelectMenuKind::Channel {
                            channel_types: None,
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
    async fn message(&self, ctx: Context, msg: Message) {
        message(&ctx, msg).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, i: Interaction) {
        match i {
            Interaction::Command(i) => interaction(&ctx, i).await.unwrap(),
            Interaction::Component(i) => println!("{:#?}", i.data),
            Interaction::Autocomplete(i) => {
                i.create_response(
                    &ctx,
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

    async fn reaction_remove_emoji(&self, _ctx: Context, removed_reactions: Reaction) {
        println!("Got ReactionRemoveEmoji event: {removed_reactions:?}");
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    env_logger::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    Client::builder(token, intents).event_handler(Handler).await?.start().await
}
