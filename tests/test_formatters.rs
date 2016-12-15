extern crate serenity;

use serenity::model::*;

#[test]
fn test_formatters() {
    assert_eq!(format!("{}", ChannelId(1)), "<#1>");
    assert_eq!(format!("{}", EmojiId(2)), "2");
    assert_eq!(format!("{}", GuildId(3)), "3");
    assert_eq!(format!("{}", RoleId(4)), "<@&4>");
    assert_eq!(format!("{}", UserId(5)), "<@5>");
}
