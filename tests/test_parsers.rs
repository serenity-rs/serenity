extern crate serenity;

use serenity::utils::*;

#[test]
fn invite_parser() {
	assert_eq!(parse_invite("https://discord.gg/abc"), "abc");
	assert_eq!(parse_invite("http://discord.gg/abc"), "abc");
	assert_eq!(parse_invite("discord.gg/abc"), "abc");
}

#[test]
fn username_parser() {
	assert_eq!(parse_username("<@12345>").unwrap(), 12345);
	assert_eq!(parse_username("<@!12345>").unwrap(), 12345);
}

#[test]
fn role_parser() {
	assert_eq!(parse_role("<@&12345>").unwrap(), 12345);
}

#[test]
fn channel_parser() {
	assert_eq!(parse_channel("<#12345>").unwrap(), 12345);
}

#[test]
fn emoji_parser() {
	let emoji = parse_emoji("<:name:12345>").unwrap();
	assert_eq!(emoji.name, "name");
	assert_eq!(emoji.id, 12345);
}

#[test]
fn quote_parser() {
	let parsed = parse_quotes("a \"b c\" d\"e f\"  g");
	assert_eq!(parsed, vec!["a", "b c", "d", "e f", "g"]);
}
