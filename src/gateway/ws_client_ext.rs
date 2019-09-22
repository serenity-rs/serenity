use chrono::Utc;
use crate::constants::{self, OpCode};
use crate::gateway::{CurrentPresence, WsClient};
use crate::internal::prelude::*;
use crate::internal::ws_impl::SenderExt;
use crate::model::id::GuildId;
use serde_json::json;
use std::env::consts;
use log::{debug, trace};

pub trait WebSocketGatewayClientExt {
    fn send_chunk_guilds<It>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<()> where It: IntoIterator<Item=GuildId>;

    fn send_heartbeat(&mut self, shard_info: &[u64; 2], seq: Option<u64>)
        -> Result<()>;

    fn send_identify(&mut self, shard_info: &[u64; 2], token: &str, guild_subscriptions: bool)
        -> Result<()>;

    fn send_presence_update(
        &mut self,
        shard_info: &[u64; 2],
        current_presence: &CurrentPresence,
    ) -> Result<()>;

    fn send_resume(
        &mut self,
        shard_info: &[u64; 2],
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()>;
}

impl WebSocketGatewayClientExt for WsClient {
    fn send_chunk_guilds<It>(
        &mut self,
        guild_ids: It,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        query: Option<&str>,
    ) -> Result<()> where It: IntoIterator<Item=GuildId> {
        debug!("[Shard {:?}] Requesting member chunks", shard_info);

        self.send_json(&json!({
            "op": OpCode::GetGuildMembers.num(),
            "d": {
                "guild_id": guild_ids.into_iter().map(|x| x.as_ref().0).collect::<Vec<u64>>(),
                "limit": limit.unwrap_or(0),
                "query": query.unwrap_or(""),
            },
        })).map_err(From::from)
    }

    fn send_heartbeat(&mut self, shard_info: &[u64; 2], seq: Option<u64>)
        -> Result<()> {
        trace!("[Shard {:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&json!({
            "d": seq,
            "op": OpCode::Heartbeat.num(),
        })).map_err(From::from)
    }

    fn send_identify(&mut self, shard_info: &[u64; 2], token: &str, guild_subscriptions: bool)
        -> Result<()> {
        debug!("[Shard {:?}] Identifying", shard_info);

        self.send_json(&json!({
            "op": OpCode::Identify.num(),
            "d": {
                "compression": true,
                "large_threshold": constants::LARGE_THRESHOLD,
                "guild_subscriptions": guild_subscriptions,
                "shard": shard_info,
                "token": token,
                "v": constants::GATEWAY_VERSION,
                "properties": {
                    "$browser": "serenity",
                    "$device": "serenity",
                    "$os": consts::OS,
                },
            },
        }))
    }

    fn send_presence_update(
        &mut self,
        shard_info: &[u64; 2],
        current_presence: &CurrentPresence,
    ) -> Result<()> {
        let &(ref activity, ref status) = current_presence;
        let now = Utc::now().timestamp() as u64;

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
        }))
    }

    fn send_resume(
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
        })).map_err(From::from)
    }
}
