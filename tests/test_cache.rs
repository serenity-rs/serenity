#![cfg(feature = "cache")]

extern crate chrono;
extern crate serde_json;
extern crate serenity;

use chrono::DateTime;
use serde_json::{Number, Value};
use serenity::{
    cache::{Cache, CacheUpdate, Settings},
    model::prelude::*,
    prelude::RwLock,
};
use std::{
    collections::HashMap,
    sync::Arc,
};

#[test]
fn test_cache_messages() {
    let mut settings = Settings::new();
    settings.max_messages(2);
    let mut cache = Cache::new_with_settings(settings);

    // Test inserting one message into a channel's message cache.
    let datetime = DateTime::parse_from_str(
        "1983 Apr 13 12:09:14.274 +0000",
        "%Y %b %d %H:%M:%S%.3f %z",
    ).unwrap();
    let mut event = MessageCreateEvent {
        message: Message {
            id: MessageId(3),
            attachments: vec![],
            author: User {
                id: UserId(2),
                avatar: None,
                bot: false,
                discriminator: 1,
                name: "user 1".to_owned(),
            },
            channel_id: ChannelId(2),
            guild_id: Some(GuildId(1)),
            content: String::new(),
            edited_timestamp: None,
            embeds: vec![],
            kind: MessageType::Regular,
            member: None,
            mention_everyone: false,
            mention_roles: vec![],
            mentions: vec![],
            nonce: Value::Number(Number::from(1)),
            pinned: false,
            reactions: vec![],
            timestamp: datetime.clone(),
            tts: false,
            webhook_id: None,
        },
    };
    // Check that the channel cache doesn't exist.
    assert!(!cache.messages.contains_key(&event.message.channel_id));
    // Add first message, none because message ID 2 doesn't already exist.
    assert!(event.update(&mut cache).is_none());
    // None, it only returns the oldest message if the cache was already full.
    assert!(event.update(&mut cache).is_none());
    // Assert there's only 1 message in the channel's message cache.
    assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 1);

    // Add a second message, assert that channel message cache length is 2.
    event.message.id = MessageId(4);
    assert!(event.update(&mut cache).is_none());
    assert_eq!(cache.messages.get(&event.message.channel_id).unwrap().len(), 2);

    // Add a third message, the first should now be removed.
    event.message.id = MessageId(5);
    assert!(event.update(&mut cache).is_some());

    {
        let channel = cache.messages.get(&event.message.channel_id).unwrap();

        assert_eq!(channel.len(), 2);
        // Check that the first message is now removed.
        assert!(!channel.contains_key(&MessageId(3)));
    }

    let guild_channel = GuildChannel {
        id: event.message.channel_id,
        bitrate: None,
        category_id: None,
        guild_id: event.message.guild_id.unwrap(),
        kind: ChannelType::Text,
        last_message_id: None,
        last_pin_timestamp: None,
        name: String::new(),
        permission_overwrites: vec![],
        position: 0,
        topic: None,
        user_limit: None,
        nsfw: false,
    };

    // Add a channel delete event to the cache, the cached messages for that
    // channel should now be gone.
    let mut delete = ChannelDeleteEvent {
        channel: Channel::Guild(Arc::new(RwLock::new(guild_channel.clone()))),
    };
    assert!(cache.update(&mut delete).is_none());
    assert!(!cache.messages.contains_key(&delete.channel.id()));

    // Test deletion of a guild channel's message cache when a GuildDeleteEvent
    // is received.
    let mut guild_create = {
        let mut channels = HashMap::new();
        channels.insert(ChannelId(2), Arc::new(RwLock::new(guild_channel.clone())));

        GuildCreateEvent {
            guild: Guild {
                id: GuildId(1),
                afk_channel_id: None,
                afk_timeout: 0,
                application_id: None,
                default_message_notifications: DefaultMessageNotificationLevel::All,
                emojis: HashMap::new(),
                explicit_content_filter: ExplicitContentFilter::None,
                features: vec![],
                icon: None,
                joined_at: datetime,
                large: false,
                member_count: 0,
                members: HashMap::new(),
                mfa_level: MfaLevel::None,
                name: String::new(),
                owner_id: UserId(3),
                presences: HashMap::new(),
                region: String::new(),
                roles: HashMap::new(),
                splash: None,
                system_channel_id: None,
                verification_level: VerificationLevel::Low,
                voice_states: HashMap::new(),
                channels,
            },
        }
    };
    assert!(cache.update(&mut guild_create).is_none());
    assert!(cache.update(&mut event).is_none());

    let mut guild_delete = GuildDeleteEvent {
        guild: PartialGuild {
            id: GuildId(1),
            afk_channel_id: None,
            afk_timeout: 0,
            default_message_notifications: DefaultMessageNotificationLevel::All,
            embed_channel_id: None,
            embed_enabled: false,
            emojis: HashMap::new(),
            features: vec![],
            icon: None,
            mfa_level: MfaLevel::None,
            name: String::new(),
            owner_id: UserId(3),
            region: String::new(),
            roles: HashMap::new(),
            splash: None,
            verification_level: VerificationLevel::Low,
        },
    };

    // The guild existed in the cache, so the cache's guild is returned by the
    // update.
    assert!(cache.update(&mut guild_delete).is_some());

    // Assert that the channel's message cache no longer exists.
    assert!(!cache.messages.contains_key(&ChannelId(2)));
}
