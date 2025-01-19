use std::collections::VecDeque;

use dashmap::DashMap;
use replace_with::replace_with_or_default;

use super::wrappers::BuildHasher;
use super::{CacheRef, ChannelId, ChannelMessagesRef, Message, MessageId, MessageRef};

/// A wrapper for implementing high level operations for message cache in a centralised place.
#[derive(Debug, Default)]
pub(super) struct MessageCache {
    // Invariants:
    // - VecDeque is no larger than the Cache's max_messages setting
    // - VecDeque is ordered via the Message ID (debug checked in MessageCache::update_message)
    // - VecDeque does not contain duplicate messages identified by their ID
    storage: DashMap<ChannelId, VecDeque<Message>, BuildHasher>,
}

impl MessageCache {
    #[cfg(test)]
    pub fn storage(&self) -> &DashMap<ChannelId, VecDeque<Message>, BuildHasher> {
        &self.storage
    }

    /// Add a message to a channel.
    pub fn create_message(&self, max_messages: usize, message: &Message) -> Option<Message> {
        // Since a message create event may be sent multiple times, it's basically an edit.
        self.update_message(max_messages, message, false)
    }

    /// Add many unsorted messages to a channel.
    pub fn create_messages(
        &self,
        max_messages: usize,
        channel_id: ChannelId,
        new_messages: impl Iterator<Item = Message>,
    ) {
        if max_messages == 0 {
            // Early exit for common case of message cache being disabled.
            return;
        }

        let mut channel_messages = self.storage.entry(channel_id).or_default();
        replace_with_or_default(&mut *channel_messages, |channel_messages| {
            // Turn the message deque into a normal contiguious Vec
            let mut contiguous_messages = Vec::from(channel_messages);

            // Fill up the existing cache
            contiguous_messages.extend(new_messages.take(max_messages));

            // Make sure the cache stays sorted to messages
            contiguous_messages.sort_unstable_by_key(|m| m.id);

            // Make sure the cache doesn't get duplicate messages
            contiguous_messages.dedup_by_key(|m| m.id);

            // Get rid of the overflow at the front of the queue.
            let truncate_end_index = contiguous_messages.len().saturating_sub(max_messages);
            contiguous_messages.drain(..truncate_end_index);

            VecDeque::from(contiguous_messages)
        });
    }

    /// Update a message for a channel.
    pub fn update_message(
        &self,
        max_messages: usize,
        message: &Message,
        return_old_message: bool,
    ) -> Option<Message> {
        if max_messages == 0 {
            return None;
        }

        // Check if the message cache for a channel exists, if not just push.
        let mut channel = match self.storage.entry(message.channel_id) {
            dashmap::Entry::Occupied(occupied_entry) => occupied_entry.into_ref(),
            dashmap::Entry::Vacant(vacant) => {
                let mut channel = VecDeque::with_capacity(1);
                channel.push_back(message.clone());
                vacant.insert(channel);

                return None;
            },
        };

        debug_assert!(
            channel.make_contiguous().is_sorted_by_key(|m| m.id),
            "message cache isn't sorted, invariant broken"
        );

        // Binary search for the message updated
        match channel.binary_search_by_key(&message.id, |m| m.id) {
            // If the message is already in cache, apply changes using `clone_from`
            Ok(existing_pos) => {
                let message_slot = &mut channel[existing_pos];
                if return_old_message {
                    let old_message = message_slot.clone();
                    message_slot.clone_from(message);
                    Some(old_message)
                } else {
                    message_slot.clone_from(message);
                    None
                }
            },
            // If the message isn't already in cache, insert it at the found position.
            Err(mut new_pos) => {
                let pruned_message = if channel.len() == max_messages {
                    let front_message = channel.front().expect("channel should have messages as `max_messages` != `0` and `channel.len()` == `max_messages`");

                    // The channel at capacity but the new message is before the last message cached
                    if front_message.id >= message.id {
                        return None;
                    }

                    new_pos -= 1;
                    channel.pop_front()
                } else {
                    None
                };

                let message = message.clone();
                channel.insert(new_pos, message);
                pruned_message
            },
        }
    }

    /// Truncate channels due to a max_messages change
    pub fn truncate_channels(&self, max: usize) {
        for mut entry in self.storage.iter_mut() {
            let message_queue = entry.value_mut();
            let queue_len = message_queue.len();

            if queue_len > max {
                message_queue.drain(..queue_len - max);
            }
        }
    }

    /// Remove the cached messages for the channel.
    pub fn remove_channel(&self, channel_id: ChannelId) -> Option<VecDeque<Message>> {
        self.storage.remove(&channel_id).map(|(_, messages)| messages)
    }

    /// Get a single message for a channel.
    pub fn get_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Option<MessageRef<'_>> {
        let messages = self.storage.get(&channel_id)?;
        let message = messages.try_map(|msgs| msgs.iter().find(|m| m.id == message_id)).ok()?;

        Some(CacheRef::from_mapped_ref(message))
    }

    /// Get a reference to all the messages in a channel.
    pub fn get_messages(&self, channel_id: ChannelId) -> Option<ChannelMessagesRef<'_>> {
        self.storage.get(&channel_id).map(CacheRef::from_ref)
    }
}
