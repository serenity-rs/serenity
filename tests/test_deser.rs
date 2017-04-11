extern crate serde;
extern crate serde_json;
extern crate serenity;

use serde::de::Deserialize;
use serde_json::Value;
use serenity::model::event::*;
use serenity::model::*;
use std::fs::File;

// A game with null type.
#[test]
fn test_game_1() {
    let f = File::open("./tests/resources/game_1.json").unwrap();
    let v = serde_json::from_reader::<File, Value>(f).unwrap();
    let _ = Game::deserialize(v).unwrap();
}

// A Discord API GUILD_CREATE.
#[test]
fn test_guild_create_1() {
    let f = File::open("./tests/resources/guild_create_1.json").unwrap();
    let v = serde_json::from_reader::<File, Value>(f).unwrap();
    let _ = GatewayEvent::decode(v).unwrap();
}
