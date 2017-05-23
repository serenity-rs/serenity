extern crate serde;
extern crate serde_json;
extern crate serenity;

use serde::de::Deserialize;
use serde_json::Value;
use serenity::model::event::*;
use serenity::model::*;
use std::fs::File;

macro_rules! p {
    ($s:ident, $filename:expr) => {
        let f = File::open(concat!("./tests/resources/", $filename, ".json")).unwrap();
        let v = serde_json::from_reader::<File, Value>(f).unwrap();
        let _ = $s::deserialize(v).unwrap();
    };
}

#[test]
fn channel_create() {
    p!(ChannelCreateEvent, "channel_create_1");
}

#[test]
fn channel_delete() {
    p!(ChannelDeleteEvent, "channel_delete_1");
}

#[test]
fn channel_pins_update() {
    p!(ChannelPinsUpdateEvent, "channel_pins_update_1");
}

#[test]
fn channel_update() {
    p!(ChannelUpdateEvent, "channel_update_1");
}

// A game with null type.
#[test]
fn game() {
    p!(Game, "game_1");
}

#[test]
fn guild_ban_add() {
    p!(GuildBanAddEvent, "guild_ban_add_1");
}

#[test]
fn guild_ban_remove() {
    p!(GuildBanRemoveEvent, "guild_ban_remove_1");
}

// The Discord API general channel over REST.
#[test]
fn guild_channel_1_rest() {
    p!(GuildChannel, "guild_channel_rest_1");
}

// A Discord API GUILD_CREATE.
#[test]
fn guild_create() {
    p!(GuildCreateEvent, "guild_create_1");
}

#[test]
fn guild_delete() {
    p!(GuildDeleteEvent, "guild_delete_1");
}

#[test]
fn guild_emojis_update() {
    p!(GuildEmojisUpdateEvent, "guild_emojis_update_1");
}

#[test]
fn guild_member_add() {
    p!(GuildMemberAddEvent, "guild_member_add_1");
}

#[test]
fn guild_member_remove() {
    p!(GuildMemberRemoveEvent, "guild_member_remove_1");
}

#[test]
fn guild_member_update() {
    p!(GuildMemberUpdateEvent, "guild_member_update_1");
}

#[test]
fn guild_role_create() {
    p!(GuildRoleCreateEvent, "guild_role_create_1");
}

#[test]
fn guild_role_delete() {
    p!(GuildRoleDeleteEvent, "guild_role_delete_1");
}

#[test]
fn guild_role_update() {
    p!(GuildRoleUpdateEvent, "guild_role_update_1");
}

#[test]
fn guild_update() {
    p!(GuildUpdateEvent, "guild_update_1");
}

#[test]
fn message_create() {
    p!(MessageCreateEvent, "message_create_1");
}

#[test]
fn message_update() {
    p!(MessageUpdateEvent, "message_update_1");
}

#[test]
fn message_reaction_add() {
    p!(ReactionAddEvent, "message_reaction_add_1");
    p!(ReactionAddEvent, "message_reaction_add_2");
}

#[test]
fn message_reaction_remove() {
    p!(ReactionRemoveEvent, "message_reaction_remove_1");
    p!(ReactionRemoveEvent, "message_reaction_remove_2");
}

#[test]
fn message_reaction_remove_all() {
    p!(ReactionRemoveAllEvent, "message_reaction_remove_all_1");
}

#[test]
fn ready() {
    p!(ReadyEvent, "ready_1");
}

#[test]
fn typing_start() {
    p!(TypingStartEvent, "typing_start_1");
}

#[test]
fn voice_state_update() {
    p!(VoiceStateUpdateEvent, "voice_state_update_1");
    p!(VoiceStateUpdateEvent, "voice_state_update_2");
}

#[test]
fn webhooks_update() {
    p!(WebhookUpdateEvent, "webhooks_update_1");
}

#[test]
fn message_type_7() {
    p!(MessageCreateEvent, "message_type_7");
}
