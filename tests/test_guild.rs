#![cfg(feature = "model")]

extern crate chrono;
extern crate serenity;

use chrono::prelude::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::*;
use std::sync::Arc;

fn gen_user() -> User {
    User {
        id: UserId(210),
        avatar: Some("abc".to_string()),
        bot: true,
        discriminator: 1432,
        name: "test".to_string(),
    }
}

fn gen_member() -> Member {
    let dt: DateTime<FixedOffset> = FixedOffset::east(5 * 3600)
        .ymd(2016, 11, 08)
        .and_hms(0, 0, 0);
    let vec1 = Vec::new();
    let u = Arc::new(RwLock::new(gen_user()));

    Member {
        deaf: false,
        guild_id: GuildId(1),
        joined_at: Some(dt),
        mute: false,
        nick: Some("aaaa".to_string()),
        roles: vec1,
        user: u,
    }
}

fn gen() -> Guild {
    let u = gen_user();
    let m = gen_member();

    let hm1 = HashMap::new();
    let hm2 = HashMap::new();
    let vec1 = Vec::new();
    let dt: DateTime<FixedOffset> = FixedOffset::east(5 * 3600)
        .ymd(2016, 11, 08)
        .and_hms(0, 0, 0);
    let mut hm3 = HashMap::new();
    let hm4 = HashMap::new();
    let hm5 = HashMap::new();
    let hm6 = HashMap::new();

    hm3.insert(u.id, m);

    Guild {
        afk_channel_id: Some(ChannelId(0)),
        afk_timeout: 0,
        channels: hm1,
        default_message_notifications: DefaultMessageNotificationLevel::All,
        emojis: hm2,
        features: vec1,
        icon: Some("/avatars/210/a_aaa.webp?size=1024".to_string()),
        id: GuildId(1),
        joined_at: dt,
        large: false,
        member_count: 1,
        members: hm3,
        mfa_level: MfaLevel::Elevated,
        name: "Spaghetti".to_string(),
        owner_id: UserId(210),
        presences: hm4,
        region: "NA".to_string(),
        roles: hm5,
        splash: Some("asdf".to_string()),
        verification_level: VerificationLevel::None,
        voice_states: hm6,
        application_id: Some(ApplicationId(0)),
        explicit_content_filter: ExplicitContentFilter::None,
        system_channel_id: Some(ChannelId(0)),
    }
}


#[test]
fn member_named_username() {
    let guild = gen();
    let lhs = guild
        .member_named("test#1432")
        .unwrap()
        .display_name();

    assert_eq!(lhs, gen_member().display_name());
}

#[test]
fn member_named_nickname() {
    let guild = gen();
    let lhs = guild.member_named("aaaa").unwrap().display_name();

    assert_eq!(lhs, gen_member().display_name());
}
