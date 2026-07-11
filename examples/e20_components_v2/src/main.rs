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
        println!("Bot is ready! Sending Image Positioning tests...");
        let channel = ChannelId::new(CHANNEL_ID);

        // ================================================================
        // TEST 1: Section + Thumbnail = gambar di KANAN text
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x3498DB)
                        .add_text_display(CreateTextDisplay::new(
                            "## 1. Section + Thumbnail (Right-aligned)\n\
                             Thumbnail muncul di sebelah **kanan** text.\n\
                             Ini adalah cara utama untuk positioning image di Component V2.",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Text di kiri, gambar di kanan.\nThumbnail auto-align ke right side.\nUkuran thumbnail: 48x48 (small) atau 80x80 (large) tergantung Discord client.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                        .description("Serenity Logo"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new())
                        .add_section(
                            CreateSection::new(
                                "Section dengan 2 text lines + thumbnail di kanan.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://docs.rs/favicon.ico")
                                        .description("Docs"),
                                ),
                            )
                            .add_text(CreateTextDisplay::new(
                                "Baris kedua: description bisa panjang.",
                            ))
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("TEST 1 FAIL: {e}");
        } else {
            println!("TEST 1 OK: Section + Thumbnail (Right-aligned)");
        }

        // ================================================================
        // TEST 2: MediaGallery = gambar dalam GRID layout
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xE74C3C)
                        .add_text_display(CreateTextDisplay::new(
                            "## 2. MediaGallery (Grid Layout)\n\
                             Gambar ditampilkan dalam grid. 1 gambar = full width.\n\
                             2 gambar = side by side. 3+ = grid pattern.",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_with_description(
                                    "https://serenity.rs/favicon.ico",
                                    "1 gambar = full width",
                                ),
                        )
                        .add_text_display(CreateTextDisplay::new(
                            "↑ 1 gambar: full width di container",
                        ))
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_with_description(
                                    "https://serenity.rs/favicon.ico",
                                    "Gambar 1",
                                )
                                .add_item_with_description(
                                    "https://docs.rs/favicon.ico",
                                    "Gambar 2",
                                ),
                        )
                        .add_text_display(CreateTextDisplay::new(
                            "↑ 2 gambar: side by side (50/50)",
                        ))
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item("https://serenity.rs/favicon.ico")
                                .add_item("https://docs.rs/favicon.ico")
                                .add_item("https://crates.io/favicon.ico"),
                        )
                        .add_text_display(CreateTextDisplay::new(
                            "↑ 3 gambar: grid layout",
                        )),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("TEST 2 FAIL: {e}");
        } else {
            println!("TEST 2 OK: MediaGallery (Grid Layout)");
        }

        // ================================================================
        // TEST 3: Standalone Thumbnail = full width image (NEW!)
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x2ECC71)
                        .add_text_display(CreateTextDisplay::new(
                            "## 3. Standalone Thumbnail (Full-width, NEW!)\n\
                             Thumbnail bisa dipakai sebagai standalone component.\n\
                             Render sebagai gambar full-width di dalam container.",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_thumbnail(
                            CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                .description("Standalone thumbnail - full width"),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_thumbnail(
                            CreateThumbnail::new("https://docs.rs/favicon.ico")
                                .description("Thumbnail kedua - juga full width"),
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("TEST 3 FAIL: {e}");
        } else {
            println!("TEST 3 OK: Standalone Thumbnail (Full-width)");
        }

        // ================================================================
        // TEST 4: Section + Button = gambar diganti button (accessory)
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0xF39C12)
                        .add_text_display(CreateTextDisplay::new(
                            "## 4. Section + Button Accessory\n\
                             Button bisa juga jadi accessory di sebelah kanan text.",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Text di kiri, button di kanan.\nButton bisa Primary/Secondary/Success/Danger.",
                                CreateSectionAccessory::Button(
                                    CreateButton::new("section_action_btn")
                                        .label("Click Me!")
                                        .style(ButtonStyle::Primary),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_section(
                            CreateSection::new(
                                "Link button sebagai accessory.",
                                CreateSectionAccessory::Button(
                                    CreateButton::new_link("https://serenity.rs")
                                        .label("Visit Serenity"),
                                ),
                            )
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("TEST 4 FAIL: {e}");
        } else {
            println!("TEST 4 OK: Section + Button Accessory");
        }

        // ================================================================
        // TEST 5: Gabungan semua positioning dalam 1 container
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x9B59B6)
                        .add_text_display(CreateTextDisplay::new(
                            "## 5. Semua Image Positioning Gabungan\n\
                             Container ini memperlihatkan semua cara position image.",
                        ))
                        .add_separator(CreateSeparator::new())
                        // Section + Thumbnail (right-aligned)
                        .add_section(
                            CreateSection::new(
                                "Cara 1: Section + Thumbnail\nImage di KANAN text.",
                                CreateSectionAccessory::Thumbnail(
                                    CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                        .description("Right-aligned"),
                                ),
                            )
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        // Standalone Thumbnail (full-width)
                        .add_text_display(CreateTextDisplay::new(
                            "Cara 2: Standalone Thumbnail (full-width)",
                        ))
                        .add_thumbnail(
                            CreateThumbnail::new("https://docs.rs/favicon.ico")
                                .description("Full-width thumbnail"),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        // MediaGallery (grid)
                        .add_text_display(CreateTextDisplay::new(
                            "Cara 3: MediaGallery (grid layout)",
                        ))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item("https://serenity.rs/favicon.ico")
                                .add_item("https://docs.rs/favicon.ico"),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        // Section + Button (no image, but same position concept)
                        .add_text_display(CreateTextDisplay::new(
                            "Cara 4: Section + Button (same position concept)",
                        ))
                        .add_section(
                            CreateSection::new(
                                "Button accessory di posisi yang sama seperti thumbnail.",
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
            eprintln!("TEST 5 FAIL: {e}");
        } else {
            println!("TEST 5 OK: All Image Positioning Combined");
        }

        // ================================================================
        // TEST 6: Spoiler thumbnail + MediaGallery spoiler
        // ================================================================
        let msg = CreateMessage::new()
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components_v2(vec![
                CreateComponents::Container(
                    CreateContainer::new()
                        .accent_color(0x1ABC9C)
                        .add_text_display(CreateTextDisplay::new(
                            "## 6. Spoiler Images\n\
                             Gambar bisa di-spoiler. Klik untuk reveal.",
                        ))
                        .add_separator(CreateSeparator::new())
                        .add_thumbnail(
                            CreateThumbnail::new("https://serenity.rs/favicon.ico")
                                .spoiler(true)
                                .description("Spoiler thumbnail"),
                        )
                        .add_separator(CreateSeparator::new().divider(false))
                        .add_media_gallery(
                            CreateMediaGallery::new()
                                .add_item_full(
                                    "https://docs.rs/favicon.ico",
                                    "Spoiler gallery item",
                                    true,
                                ),
                        ),
                ),
            ]);
        if let Err(e) = channel.send_message(&ctx, msg).await {
            eprintln!("TEST 6 FAIL: {e}");
        } else {
            println!("TEST 6 OK: Spoiler Images");
        }

        println!("\n=== ALL IMAGE POSITIONING TESTS COMPLETED ===");
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
