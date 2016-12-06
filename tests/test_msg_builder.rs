extern crate serenity;

use serenity::utils::MessageBuilder;
use serenity::model::Emoji;
use serenity::model::EmojiId;
use serenity::model::UserId;

#[test]
fn code_blocks() {
    let content = MessageBuilder::new()
        .push_codeblock("test", Some("rb"))
        .build();
    assert_eq!(content, "```rb\ntest\n```");
}


#[test]
fn safe_content() {
    let content = MessageBuilder::new()
        .push_safe("@everyone discord.gg/discord-api")
        .build();
    assert!(content != "@everyone discord.gg/discord-api");
}


#[test]
fn no_free_formatting() {
    let content = MessageBuilder::new()
        .push_bold_safe("test**test")
        .build();
    assert!(content != "**test**test**");
}

#[test]
fn mentions() {
    let content_emoji = MessageBuilder::new()
        .emoji(Emoji {
            id: EmojiId(32),
            name: "Rohrkatze".to_string(),
            managed: false,
            require_colons: true,
            roles: vec![]
        })
        .build();
    let content_mentions = MessageBuilder::new()
        .channel(1)
        .mention(UserId(2))
        .role(3)
        .user(4)
        .build();
    assert_eq!(content_mentions, "<#1><@2><@&3><@4>");
    assert_eq!(content_emoji, "<:Rohrkatze:32>");
}
