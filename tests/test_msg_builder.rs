#![cfg(feature = "utils")]

extern crate serenity;

use serenity::utils::MessageBuilder;
use serenity::utils::ContentModifier::*;
use serenity::model::guild::Emoji;
use serenity::model::id::{EmojiId, UserId};

#[test]
fn code_blocks() {
    let mut builder = MessageBuilder::new();
    builder.push_codeblock("test", Some("rb"));
    let content = builder.build();
    assert_eq!(content, "```rb\ntest\n```");
}

#[test]
fn safe_content() {
    let mut builder = MessageBuilder::new();
    builder.push_safe("@everyone discord.gg/discord-api");
    let content = builder.build();
    assert_ne!(content, "@everyone discord.gg/discord-api");
}

#[test]
fn no_free_formatting() {
    let mut builder = MessageBuilder::new();
    builder.push_bold_safe("test**test");
    let content = builder.build();
    assert_ne!(content, "**test**test**");
}

#[test]
fn mentions() {
    let content_emoji = {
        let mut builder = MessageBuilder::new();
        builder.emoji(&Emoji {
            animated: false,
            id: EmojiId(32),
            name: "Rohrkatze".to_string(),
            managed: false,
            require_colons: true,
            roles: vec![],
        });
        builder.build()
    };

    let content_mentions = {
        let mut builder = MessageBuilder::new();
        builder.channel(1)
            .mention(&UserId(2))
            .role(3)
            .user(4);
        builder.build()
    };
    assert_eq!(content_mentions, "<#1><@2><@&3><@4>");
    assert_eq!(content_emoji, "<:Rohrkatze:32>");
}

#[test]
fn content() {
    let content = Bold + Italic + Code + "Fun!";

    assert_eq!(content.to_string(), "***`Fun!`***");
}

#[test]
fn message_content() {
    let mut builder = MessageBuilder::new();
    builder.push(Bold + Italic + Code + "Fun!");
    let message_content = builder.build();

    assert_eq!(message_content, "***`Fun!`***");
}

#[test]
fn message_content_safe() {
    let mut builder = MessageBuilder::new();
    builder.push_safe(Bold + Italic + "test**test");
    let message_content = builder.build();

    assert_eq!(message_content, "***test\\*\\*test***");
}
