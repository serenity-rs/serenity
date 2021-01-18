use std::convert::TryFrom;
use std::str::FromStr;

use serenity::model::channel::ReactionType;
use serenity::model::id::EmojiId;

#[test]
fn str_to_reaction_type() {
    let emoji_str = "<:customemoji:600404340292059257>";
    let reaction = ReactionType::try_from(emoji_str).unwrap();
    let reaction2 = ReactionType::Custom {
        animated: false,
        id: EmojiId(600404340292059257),
        name: Some("customemoji".to_string()),
    };
    assert_eq!(reaction, reaction2);
}

#[test]
fn str_to_reaction_type_animated() {
    let emoji_str = "<a:customemoji2:600409340292059257>";
    let reaction = ReactionType::try_from(emoji_str).unwrap();
    let reaction2 = ReactionType::Custom {
        animated: true,
        id: EmojiId(600409340292059257),
        name: Some("customemoji2".to_string()),
    };
    assert_eq!(reaction, reaction2);
}

#[test]
fn string_to_reaction_type() {
    let emoji_string = "<:customemoji:600404340292059257>".to_string();
    let reaction = ReactionType::try_from(emoji_string).unwrap();
    let reaction2 = ReactionType::Custom {
        animated: false,
        id: EmojiId(600404340292059257),
        name: Some("customemoji".to_string()),
    };
    assert_eq!(reaction, reaction2);
}

#[test]
fn string_to_reaction_type_empty() {
    let emoji_string = "".to_string();
    ReactionType::try_from(emoji_string).unwrap_err();
}

#[test]
fn str_to_reaction_type_empty() {
    let emoji_str = "";
    ReactionType::try_from(emoji_str).unwrap_err();
}

#[test]
fn str_to_reaction_type_mangled() {
    let emoji_str = "<a:custom:emoji2:600409340292059257>";
    ReactionType::try_from(emoji_str).unwrap_err();
}

#[test]
fn str_to_reaction_type_mangled_2() {
    let emoji_str = "<a:customemoji2:600409340292059257>Trail";
    ReactionType::try_from(emoji_str).unwrap_err();
}

#[test]
fn str_to_reaction_type_mangled_3() {
    let emoji_str = "<somestuff:1234>";
    ReactionType::try_from(emoji_str).unwrap_err();
}

#[test]
fn str_to_reaction_type_mangled_4() {
    let emoji_str = "<:somestuff:1234";
    ReactionType::try_from(emoji_str).unwrap_err();
}

#[test]
fn str_fromstr() {
    let emoji_str = "<:somestuff:1234";
    ReactionType::from_str(emoji_str).unwrap_err();
}
