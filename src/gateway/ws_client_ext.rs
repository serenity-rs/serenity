use crate::constants::{self, OpCode};
use crate::gateway::{CurrentPresence, WsStream};
use crate::client::bridge::gateway::GatewayIntents;
use crate::internal::prelude::*;
use crate::internal::ws_impl::SenderExt;
use crate::model::id::GuildId;
use serde_json::json;
use std::env::consts;
use std::time::SystemTime;
use tracing::{debug, trace};
use async_trait::async_trait;
use tracing::instrument;

#[async_trait]
pub trait WebSocketGatewayClientExt {
    async fn send_chunk_guilds<It>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<()> where It: IntoIterator<Item=GuildId> + Send;

    async fn send_heartbeat(&mut self, shard_info: &[u64; 2], seq: Option<u64>)
        -> Result<()>;

    async fn send_identify(&mut self, shard_info: &[u64; 2], token: &str, guild_subscriptions: bool, intents: Option<GatewayIntents>)
        -> Result<()>;

    async fn send_presence_update(
        &mut self,
        shard_info: &[u64; 2],
        current_presence: &CurrentPresence,
    ) -> Result<()>;

    async fn send_resume(
        &mut self,
        shard_info: &[u64; 2],
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()>;
}

#[async_trait]
impl WebSocketGatewayClientExt for WsStream {
    #[instrument(skip(self, guild_ids))]
    async fn send_chunk_guilds<It>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<()> where It: IntoIterator<Item=GuildId> + Send {
        debug!("[Shard {:?}] Requesting member chunks", shard_info);

        self.send_json(&json!({
            "op": OpCode::GetGuildMembers.num(),
            "d": {
                "guild_id": guild_ids.into_iter().map(|x| x.as_ref().0).collect::<Vec<u64>>(),
                "limit": limit.unwrap_or(0),
                "query": query.unwrap_or(""),
            },
        })).await.map_err(From::from)
    }

    #[instrument(skip(self))]
    async fn send_heartbeat(&mut self, shard_info: &[u64; 2], seq: Option<u64>)
        -> Result<()> {
        trace!("[Shard {:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&json!({
            "d": seq,
            "op": OpCode::Heartbeat.num(),
        })).await.map_err(From::from)
    }

    #[instrument(skip(self, token))]
    async fn send_identify(&mut self, shard_info: &[u64; 2], token: &str, guild_subscriptions: bool, intents: Option<GatewayIntents>)
        -> Result<()> {
        debug!("[Shard {:?}] Identifying", shard_info);

        self.send_json(&json!({
            "op": OpCode::Identify.num(),
            "d": {
                "compress": true,
                "large_threshold": constants::LARGE_THRESHOLD,
                "guild_subscriptions": guild_subscriptions,
                "shard": shard_info,
                "token": token,
                "intents": intents,
                "v": constants::GATEWAY_VERSION,
                "properties": {
                    "$browser": "serenity",
                    "$device": "serenity",
                    "$os": consts::OS,
                },
            },
        })).await
    }

    #[instrument(skip(self))]
    async fn send_presence_update(
        &mut self,
        shard_info: &[u64; 2],
        current_presence: &CurrentPresence,
    ) -> Result<()> {
        let &(ref activity, ref status) = current_presence;
        let now = SystemTime::now();

        debug!("[Shard {:?}] Sending presence update", shard_info);

        self.send_json(&json!({
            "op": OpCode::StatusUpdate.num(),
            "d": {
                "afk": false,
                "since": now,
                "status": status.name(),
                "game": activity.as_ref().map(|x| json!({
                    "name": x.name,
                    "type": x.kind,
                    "url": x.url,
                })),
            },
        })).await
    }

    #[instrument(skip(self))]
    async fn send_resume(
        &mut self,
        shard_info: &[u64; 2],
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[Shard {:?}] Sending resume; seq: {}", shard_info, seq);

        self.send_json(&json!({
            "op": OpCode::Resume.num(),
            "d": {
                "session_id": session_id,
                "seq": seq,
                "token": token,
            },
        })).await.map_err(From::from)
    }
}
