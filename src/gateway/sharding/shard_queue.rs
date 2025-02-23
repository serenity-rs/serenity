use std::collections::VecDeque;
use std::num::NonZeroU16;

use super::ShardId;
use crate::internal::prelude::*;

/// A queue of [`ShardId`]s that is split up into multiple buckets according to the value of
/// [`max_concurrency`](crate::model::gateway::SessionStartLimit::max_concurrency).
#[must_use]
pub struct ShardQueue {
    buckets: FixedArray<VecDeque<ShardId>, u16>,
}

impl ShardQueue {
    pub fn new(max_concurrency: NonZeroU16) -> Self {
        let buckets = vec![VecDeque::new(); max_concurrency.get() as usize].into_boxed_slice();
        let buckets = FixedArray::try_from(buckets).expect("should fit without truncation");

        Self {
            buckets,
        }
    }

    fn calculate_bucket(&self, shard_id: ShardId) -> u16 {
        shard_id.0 % self.buckets.len()
    }

    /// Calculates the corresponding bucket for the given `ShardId` and **appends** to it.
    pub fn push_back(&mut self, shard_id: ShardId) {
        let bucket = self.calculate_bucket(shard_id);
        self.buckets[bucket].push_back(shard_id);
    }

    /// Calculates the corresponding bucket for the given `ShardId` and **prepends** to it.
    pub fn push_front(&mut self, shard_id: ShardId) {
        let bucket = self.calculate_bucket(shard_id);
        self.buckets[bucket].push_front(shard_id);
    }

    /// Pops a `ShardId` from every bucket containing at least one and returns them all as a `Vec`.
    pub fn pop_batch(&mut self) -> Vec<ShardId> {
        self.buckets.iter_mut().filter_map(VecDeque::pop_front).collect()
    }
}
