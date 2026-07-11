use std::env;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{
    CreateActionRow,
    CreateButton,
    CreateComponents,
    CreateContainer,
    CreateMediaGallery,
    CreateMessage,
    CreateSection,
    CreateSectionAccessory,
    CreateSelectMenu,
    CreateSelectMenuKind,
    CreateSelectMenuOption,
    CreateSeparator,
    CreateTextDisplay,
    CreateThumbnail,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

const CHANNEL_ID: u64 = 1457222385991417869;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        println!("Bot is ready! Sending Component V2 tests...");
        let channel = ChannelId::new(CHANNEL_ID);

        // Section + Thumbnail (right-aligned image)
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x3498DB)
                        .add_text_display(CreateTextDisplay::new(
                            "## Section + Thumbnail\nImage appears to the right of text.",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Text on the left, image on the right.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                        .description("Serenity Logo"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new())
                        .add_section(
                            CreateSection::new(
                                "Section with 2 text lines + thumbnail.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://docs.rs/favicon.ico")
                                        .description("Docs"),
                                ),
                            )
                            .add_text(CreateTextDisplay::new(
                                "Second line: description can be long.",
                            ))
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Section+Thumbnail FAIL: {e}");
        } else {
            println!("OK: Section + Thumbnail");
        }

        // MediaGallery (grid layout)
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xE74C3C)
                        .add_text_display(CreateTextDisplay::new(
                            "## MediaGallery\n1 image = full width, 2 = side by side, 3+ = grid.",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_with_description(
                                    "https://serenity.rs/favicon.ico",
                                    "1 image: full width",
                                ),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_with_description(
                                    "https://serenity.rs/favicon.ico",
                                    "Image 1",
                                )
                                .add_item_with_description(
                                    "https://docs.rs/favicon.ico",
                                    "Image 2",
                                ),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item("https://serenity.rs/favicon.ico")
                                .add_item("https://docs.rs/favicon.ico")
                                .add_item("https://crates.io/favicon.ico"),
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("MediaGallery FAIL: {e}");
        } else {
            println!("OK: MediaGallery");
        }

        // Container with multiple section types
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x9B59B6)
                        .add_text_display(CreateTextDisplay::new(
                            "## All Image Positioning",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_section(
                            CreateSection::new(
                                "Section + Thumbnail (right-aligned)",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                        .description("Right-aligned"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_text_display(CreateTextDisplay::new(
                            "MediaGallery (grid layout)",
                        ))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item("https://serenity.rs/favicon.ico")
                                .add_item("https://docs.rs/favicon.ico"),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_text_display(CreateTextDisplay::new(
                            "Section + Button (same position concept)",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Button accessory in the same position as thumbnail.",
                                CreateSectionAccessory::Button(
                                    CreateButton::new("combo_btn")
                                        .label("Action")
                                        .style(ButtonStyle::Success),
                                ),
                            )
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("All Positioning FAIL: {e}");
        } else {
            println!("OK: All Image Positioning");
        }

        // Container with all button styles
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xF39C12)
                        .add_text_display(CreateTextDisplay::new(
                            "## Button Variants",
                        ))
                        .add_action_row(CreateActionRow::Buttons(vec![
                            CreateButton::new("btn_primary").label("Primary").style(ButtonStyle::Primary),
                            CreateButton::new("btn_secondary").label("Secondary").style(ButtonStyle::Secondary),
                            CreateButton::new("btn_success").label("Success").style(ButtonStyle::Success),
                            CreateButton::new("btn_danger").label("Danger").style(ButtonStyle::Danger),
                            CreateButton::new_link("https://serenity.rs").label("Link"),
                        ]))
                        .add_action_row(CreateActionRow::Buttons(vec![
                            CreateButton::new("btn_disabled").label("Disabled").disabled(true),
                            CreateButton::new("btn_emoji").label("Rocket").emoji('🚀'),
                        ])),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Button Variants FAIL: {e}");
        } else {
            println!("OK: Button Variants");
        }

        // All select menu types
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x1ABC9C)
                        .add_text_display(CreateTextDisplay::new(
                            "## All Select Menu Types",
                        ))
                        .add_action_row(CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "string_select",
                                CreateSelectMenuKind::String {
                                    options: vec![
                                        CreateSelectMenuOption::new("🍎 Apple", "apple"),
                                        CreateSelectMenuOption::new("🍊 Orange", "orange"),
                                        CreateSelectMenuOption::new("🍋 Lemon", "lemon"),
                                    ],
                                },
                            )
                            .placeholder("String Select...")
                            .min_values(1)
                            .max_values(2),
                        ))
                        .add_action_row(CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "user_select",
                                CreateSelectMenuKind::User { default_users: None },
                            )
                            .placeholder("User Select..."),
                        ))
                        .add_action_row(CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "role_select",
                                CreateSelectMenuKind::Role { default_roles: None },
                            )
                            .placeholder("Role Select..."),
                        ))
                        .add_action_row(CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "mentionable_select",
                                CreateSelectMenuKind::Mentionable {
                                    default_users: None,
                                    default_roles: None,
                                },
                            )
                            .placeholder("Mentionable Select..."),
                        ))
                        .add_action_row(CreateActionRow::SelectMenu(
                            CreateSelectMenu::new(
                                "channel_select",
                                CreateSelectMenuKind::Channel {
                                    channel_types: Some(vec![ChannelType::Text, ChannelType::Voice]),
                                    default_channels: None,
                                },
                            )
                            .placeholder("Channel Select..."),
                        )),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Select Menus FAIL: {e}");
        } else {
            println!("OK: All Select Menu Types");
        }

        // Spoiler images
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x1ABC9C)
                        .add_text_display(CreateTextDisplay::new(
                            "## Spoiler Images\nClick to reveal.",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_section(CreateSection::new(
                            "Spoiler thumbnail as Section accessory:",
                            CreateSectionAccessory::Thumbnail(
                                CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                    .spoiler(true)
                                    .description("Spoiler thumbnail"),
                            ),
                        ))
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_full(
                                    "https://docs.rs/favicon.ico",
                                    "Spoiler gallery",
                                    true,
                                ),
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Spoilers FAIL: {e}");
        } else {
            println!("OK: Spoiler Images");
        }

        // Multiple colored containers
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new().accent_color(0xFF0000).add_text_display(
                        CreateTextDisplay::new("Red Container"),
                    ),
                ),
                CreateComponents::Container(
                    CreateContainer::new().accent_color(0x00FF00).add_text_display(
                        CreateTextDisplay::new("Green Container"),
                    ),
                ),
                CreateComponents::Container(
                    CreateContainer::new().accent_color(0x0000FF).add_text_display(
                        CreateTextDisplay::new("Blue Container"),
                    ),
                ),
                CreateComponents::Container(
                    CreateContainer::new().accent_color(0xFFAA00).add_text_display(
                        CreateTextDisplay::new("Orange Container"),
                    ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Colored Containers FAIL: {e}");
        } else {
            println!("OK: Colored Containers");
        }

        // Spoiler container
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::TextDisplay(CreateTextDisplay::new(
                    "## Spoiler Container\nClick to reveal.",
                )),
                CreateComponents::Container(
                    CreateContainer::new()
                        .spoiler(true)
                        .accent_color(0x9B59B6)
                        .add_text_display(CreateTextDisplay::new(
                            "This content is spoilered!",
                        ))
                        .add_action_row(CreateActionRow::Buttons(vec![
                            CreateButton::new("spoiler_btn")
                                .label("Secret Button")
                                .style(ButtonStyle::Primary),
                        ])),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Spoiler Container FAIL: {e}");
        } else {
            println!("OK: Spoiler Container");
        }

        // Section variants
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xE74C3C)
                        .add_text_display(CreateTextDisplay::new(
                            "## Section Variants",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Link button as accessory.",
                                CreateSectionAccessory::Button(
                                    CreateButton::new_link("https://serenity.rs")
                                        .label("Visit Serenity"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_section(
                            CreateSection::new(
                                "Thumbnail without description.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://docs.rs/favicon.ico"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_section(
                            CreateSection::new(
                                "Spoilered thumbnail.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                        .spoiler(true)
                                        .description("Spoiler"),
                                ),
                            )
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Section Variants FAIL: {e}");
        } else {
            println!("OK: Section Variants");
        }

        // Section with multi-text
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xFF6B6B)
                        .add_section(
                            CreateSection::new_with_components(
                                vec![
                                    CreateTextDisplay::new("Line 1: first text"),
                                    CreateTextDisplay::new("Line 2: second text"),
                                    CreateTextDisplay::new("Line 3: third text (max 3)"),
                                ],
                                CreateSectionAccessory::Button(
                                    CreateButton::new("multi_text_btn")
                                        .label("Action")
                                        .style(ButtonStyle::Success),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new())
                        .add_text_display(CreateTextDisplay::new(
                            "Section above has 3 text lines + 1 button.",
                        )),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Multi-Text FAIL: {e}");
        } else {
            println!("OK: Section Multi-Text");
        }

        // Separator variants
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x2ECC71)
                        .add_text_display(CreateTextDisplay::new(
                            "## Separator Variants",
                        ))
                        .add_text_display(CreateTextDisplay::new("Small spacing (default)"))
                        .add_separator(CreateSeparator::new())
                        .add_text_display(CreateTextDisplay::new("Large spacing"))
                        .add_separator(CreateSeparator::new().spacing(2))
                        .add_text_display(CreateTextDisplay::new("No divider"))
                        .add_separator(CreateSeparator::new().divider(false)),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("Separator Variants FAIL: {e}");
        } else {
            println!("OK: Separator Variants");
        }

        println!("\n=== ALL TESTS COMPLETED ===");
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
