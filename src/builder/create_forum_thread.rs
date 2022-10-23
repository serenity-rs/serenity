use crate::builder::{CreateMessage, CreateThread};
use crate::json::{self, from_number, json, JsonMap, Value};
use crate::model::channel::ChannelType;

#[derive(Debug, Clone, Default)]
pub struct CreateForumThread<'a> {
    thread: CreateThread,
    message: CreateMessage<'a>,
}

impl<'a> CreateForumThread<'a> {
    /// Make default payload for creating a forum thread.
    ///
    /// It would automatically contain the right type for a channel.
    pub fn new() -> Self {
        let mut instance = CreateForumThread::default();

        instance.thread.0.insert("type", from_number(ChannelType::Forum as u8));

        instance
    }

    pub fn to_map(&self) -> JsonMap {
        let mut map = JsonMap::new();

        map.insert("thread".to_owned(), Value::from(json::hashmap_to_json_map(self.thread.0.clone())));
        map.insert("message".to_owned(), Value::from(json::hashmap_to_json_map(self.message.0.clone())));

        map
    }
}
