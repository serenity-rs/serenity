extern crate serenity;

use serenity::model::*;

#[test]
fn test_formatters() {
    assert_eq!(ChannelId(1).to_string(), "1");
    assert_eq!(EmojiId(2).to_string(), "2");
    assert_eq!(GuildId(3).to_string(), "3");
    assert_eq!(RoleId(4).to_string(), "4");
    assert_eq!(UserId(5).to_string(), "5");
}

#[cfg(feature = "utils")]
#[test]
fn test_mention() {
    use serenity::utils::Colour;
    use std::sync::{Arc, RwLock};

    let channel = Channel::Guild(Arc::new(RwLock::new(GuildChannel {
        bitrate: None,
        category_id: None,
        guild_id: GuildId(1),
        kind: ChannelType::Text,
        id: ChannelId(4),
        last_message_id: None,
        last_pin_timestamp: None,
        name: "a".to_owned(),
        permission_overwrites: vec![],
        position: 1,
        topic: None,
        user_limit: None,
        nsfw: false,
    })));
    let emoji = Emoji {
        id: EmojiId(5),
        name: "a".to_owned(),
        managed: true,
        require_colons: true,
        roles: vec![],
    };
    let role = Role {
        id: RoleId(2),
        colour: Colour::rosewater(),
        hoist: false,
        managed: false,
        mentionable: false,
        name: "fake role".to_owned(),
        permissions: Permissions::empty(),
        position: 1,
    };
    let user = User {
        id: UserId(6),
        avatar: None,
        bot: false,
        discriminator: 4132,
        name: "fake".to_owned(),
    };
    let member = Member {
        deaf: false,
        guild_id: GuildId(2),
        joined_at: None,
        mute: false,
        nick: None,
        roles: vec![],
        user: Arc::new(RwLock::new(user.clone())),
    };

    assert_eq!(ChannelId(1).mention(), "<#1>");
    assert_eq!(channel.mention(), "<#4>");
    assert_eq!(emoji.mention(), "<:a:5>");
    assert_eq!(member.mention(), "<@6>");
    assert_eq!(role.mention(), "<@&2>");
    assert_eq!(role.id.mention(), "<@&2>");
    assert_eq!(user.mention(), "<@6>");
    assert_eq!(user.id.mention(), "<@6>");
}
