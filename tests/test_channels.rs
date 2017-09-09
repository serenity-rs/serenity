extern crate serenity;

#[cfg(feature = "utils")]
mod utils {
    use serenity::model::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    fn group() -> Group {
        Group {
            channel_id: ChannelId(1),
            icon: None,
            last_message_id: None,
            last_pin_timestamp: None,
            name: None,
            owner_id: UserId(2),
            recipients: HashMap::new(),
        }
    }

    fn guild_channel() -> GuildChannel {
        GuildChannel {
            id: ChannelId(1),
            bitrate: None,
            category_id: None,
            guild_id: GuildId(2),
            kind: ChannelType::Text,
            last_message_id: None,
            last_pin_timestamp: None,
            name: "nsfw-stuff".to_owned(),
            permission_overwrites: vec![],
            position: 0,
            topic: None,
            user_limit: None,
            nsfw: false,
        }
    }

    fn private_channel() -> PrivateChannel {
        PrivateChannel {
            id: ChannelId(1),
            last_message_id: None,
            last_pin_timestamp: None,
            kind: ChannelType::Private,
            recipient: Arc::new(RwLock::new(User {
                id: UserId(2),
                avatar: None,
                bot: false,
                discriminator: 1,
                name: "ab".to_owned(),
            })),
        }
    }

    #[test]
    fn nsfw_checks() {
        let mut channel = guild_channel();
        assert!(channel.is_nsfw());
        channel.kind = ChannelType::Voice;
        assert!(!channel.is_nsfw());

        channel.kind = ChannelType::Text;
        channel.name = "nsfw-".to_owned();
        assert!(channel.is_nsfw());

        channel.name = "nsfw".to_owned();
        assert!(channel.is_nsfw());
        channel.kind = ChannelType::Voice;
        assert!(!channel.is_nsfw());
        channel.kind = ChannelType::Text;

        channel.name = "nsf".to_owned();
        channel.nsfw = true;
        assert!(channel.is_nsfw());
        channel.nsfw = false;
        assert!(!channel.is_nsfw());

        let channel = Channel::Guild(Arc::new(RwLock::new(channel)));
        assert!(!channel.is_nsfw());

        let group = group();
        assert!(!group.is_nsfw());

        let private_channel = private_channel();
        assert!(!private_channel.is_nsfw());
    }
}
