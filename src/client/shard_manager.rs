use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum ShardingStrategy {
    Autoshard,
    Range([u64; 3]),
}

impl ShardingStrategy {
    pub fn auto() -> Self {
        ShardingStrategy::Autoshard
    }

    pub fn multi(count: u64) -> Self {
        ShardingStrategy::Range([0, count, count])
    }

    pub fn simple() -> Self {
        ShardingStrategy::Range([0, 1, 1])
    }

    pub fn range(index: u64, count: u64, total: u64) -> Self {
        ShardingStrategy::Range([index, count, total])
    }
}

impl Default for ShardingStrategy {
    fn default() -> Self {
        ShardingStrategy::Autoshard
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShardManagerOptions {
    pub strategy: ShardingStrategy,
    pub token: Arc<String>,
    pub ws_uri: Arc<String>,
}

#[derive(Debug)]
pub struct ShardManager {
    pub queue: VecDeque<u64>,
    pub shards: (),
    pub strategy: ShardingStrategy,
    pub token: Arc<String>,
    pub ws_uri: Arc<String>,
    non_exhaustive: (),
}

impl ShardManager {
    pub fn new(options: ShardManagerOptions) -> Self {
        Self {
            queue: VecDeque::new(),
            shards: (),
            strategy: options.strategy,
            token: options.token,
            ws_uri: options.ws_uri,
            non_exhaustive: (),
        }
    }
}
