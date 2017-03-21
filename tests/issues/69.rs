extern crate serenity;

use serenity::ext::cache::Cache;
use serenity::model::event::ChannelCreateEvent;
use serenity::model::GuildId;

#[ignore]
fn test_private_channel_create() {
    let cache = Cache::default();

    let event = ChannelCreateEvent {
        channel: Channel {

        }
    }
}
