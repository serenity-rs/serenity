use serenity::builder::*;
use serenity::model::prelude::command::*;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

mod model_type_sizes;

const IMAGE_URL: &str = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";
const IMAGE_URL_2: &str = "https://rustacean.net/assets/rustlogo.png";

async fn message(ctx: &Context, msg: Message) -> Result<(), serenity::Error> {
    let channel_id = msg.channel_id;
    if let Some(_args) = msg.content.strip_prefix("testmessage ") {
        println!("command message: {:#?}", msg);
    } else if msg.content == "globalcommand" {
        // Tests https://github.com/serenity-rs/serenity/issues/2259
        // Activate simd_json feature for this
        Command::create_global_application_command(
            &ctx,
            CreateCommand::new("ping").description("test command"),
        )
        .await?;
    } else if msg.content == "register" {
        let guild_id = msg.guild_id.unwrap();
        guild_id
            .create_application_command(
                &ctx,
                CreateCommand::new("editattachments").description("test command"),
            )
            .await?;
        guild_id
            .create_application_command(
                &ctx,
                CreateCommand::new("unifiedattachments1").description("test command"),
            )
            .await?;
        guild_id
            .create_application_command(
                &ctx,
                CreateCommand::new("unifiedattachments2").description("test command"),
            )
            .await?;
        guild_id
            .create_application_command(
                &ctx,
                CreateCommand::new("editembeds").description("test command"),
            )
            .await?;
        guild_id
            .create_application_command(
                &ctx,
                CreateCommand::new("newselectmenu").description("test command"),
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
                CreateMessage::new().components(vec![
                    CreateActionRow::Buttons(vec![
                        CreateButton::new("foo", ButtonStyle::Primary, "0"),
                        CreateButton::new("bar", ButtonStyle::Secondary, "1"),
                        CreateButton::new_link("baz", "https://google.com"),
                    ]),
                    // ONLY VALID IN MODALS
                    // CreateActionRow::InputText(CreateInputText::new(
                    //     InputTextStyle::Short,
                    //     "hi",
                    //     "2",
                    // )),
                    CreateActionRow::SelectMenu(CreateSelectMenu::new(
                        "3",
                        CreateSelectMenuKind::String {
                            options: vec![
                                CreateSelectMenuOption::new("foo", "foo"),
                                CreateSelectMenuOption::new("bar", "bar"),
                            ],
                        },
                    )),
                ]),
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
    interaction: CommandInteraction,
) -> Result<(), serenity::Error> {
    if interaction.data.name == "editattachments" {
        // Respond with an image
        interaction
            .create_interaction_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
                ),
            )
            .await?;

        // We need to know the attachments' IDs in order to not lose them in the subsequent edit
        let msg = interaction.get_interaction_response(ctx).await?;

        // Add another image
        let msg = interaction
            .edit_original_interaction_response(
                &ctx,
                EditInteractionResponse::new()
                    .new_attachment(CreateAttachment::url(ctx, IMAGE_URL_2).await?)
                    .keep_existing_attachment(msg.attachments[0].id),
            )
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Only keep the new image, removing the first image
        let _msg = interaction
            .edit_original_interaction_response(
                &ctx,
                EditInteractionResponse::new()
                    .clear_existing_attachments()
                    .keep_existing_attachment(msg.attachments[1].id),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments1" {
        interaction
            .create_interaction_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("works"),
                ),
            )
            .await?;

        interaction
            .edit_original_interaction_response(
                ctx,
                EditInteractionResponse::new().content("works still"),
            )
            .await?;

        interaction
            .create_followup_message(
                ctx,
                CreateInteractionResponseFollowup::new().content("still works still"),
            )
            .await?;
    } else if interaction.data.name == "unifiedattachments2" {
        interaction
            .create_interaction_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
                ),
            )
            .await?;

        interaction
            .edit_original_interaction_response(
                ctx,
                EditInteractionResponse::new()
                    .new_attachment(CreateAttachment::url(ctx, IMAGE_URL_2).await?),
            )
            .await?;

        interaction
            .create_followup_message(
                ctx,
                CreateInteractionResponseFollowup::new()
                    .add_file(CreateAttachment::url(ctx, IMAGE_URL).await?),
            )
            .await?;
    } else if interaction.data.name == "editembeds" {
        interaction
            .create_interaction_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("hi")
                        .embed(CreateEmbed::new().description("hi")),
                ),
            )
            .await?;

        // Pre-PR, this falsely deleted the embed
        interaction
            .edit_original_interaction_response(&ctx, EditInteractionResponse::new())
            .await?;
    } else if interaction.data.name == "newselectmenu" {
        interaction
            .create_interaction_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().components(
                        // Make one action row for each kind of select menu
                        vec![
                            CreateSelectMenuKind::String {
                                options: vec![
                                    CreateSelectMenuOption::new("foo", "foo"),
                                    CreateSelectMenuOption::new("bar", "bar"),
                                ],
                            },
                            CreateSelectMenuKind::Mentionable,
                            CreateSelectMenuKind::Role,
                            CreateSelectMenuKind::User,
                            CreateSelectMenuKind::Channel {
                                channel_types: None,
                            },
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, kind)| {
                            CreateActionRow::SelectMenu(CreateSelectMenu::new(i.to_string(), kind))
                        })
                        .collect(),
                    ),
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
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), serenity::Error> {
    env_logger::init();
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    Client::builder(token, intents).event_handler(Handler).await?.start().await
}
