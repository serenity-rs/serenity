use std::env::consts;
use std::io::Read;
use std::time::SystemTime;

use async_tungstenite::tokio::{connect_async_with_config, ConnectStream};
use async_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use async_tungstenite::tungstenite::{Error as WsError, Message};
use async_tungstenite::WebSocketStream;
use flate2::read::ZlibDecoder;
use futures::{SinkExt, StreamExt};
use tokio::time::{timeout, Duration};
use tracing::{debug, instrument, trace, warn};
use url::Url;

use crate::client::bridge::gateway::ChunkGuildFilter;
use crate::constants::{self, Opcode};
use crate::gateway::{CurrentPresence, GatewayError};
use crate::json::{from_str, json, to_string, Value};
use crate::model::event::GatewayEvent;
use crate::model::gateway::GatewayIntents;
use crate::model::id::GuildId;
use crate::{Error, Result};

pub struct WsClient(WebSocketStream<ConnectStream>);

const TIMEOUT: Duration = Duration::from_millis(500);
const DECOMPRESSION_MULTIPLIER: usize = 3;

impl WsClient {
    pub(crate) async fn connect(url: Url) -> Result<Self> {
        let config = WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            max_send_queue: None,
            accept_unmasked_frames: false,
        };
        let (stream, _) = connect_async_with_config(url, Some(config)).await?;

        Ok(Self(stream))
    }

    pub(crate) async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.0.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => return Ok(None),
        };

        let value = match message {
            Message::Binary(bytes) => {
                let mut decompressed =
                    String::with_capacity(bytes.len() * DECOMPRESSION_MULTIPLIER);

                ZlibDecoder::new(&bytes[..]).read_to_string(&mut decompressed).map_err(|why| {
                    warn!("Err decompressing bytes: {:?}; bytes: {:?}", why, bytes);

                    why
                })?;

                from_str(decompressed.as_mut_str()).map_err(|why| {
                    warn!("Err deserializing bytes: {:?}; bytes: {:?}", why, bytes);

                    why
                })?
            },
            Message::Text(mut payload) => from_str(&mut payload).map_err(|why| {
                warn!("Err deserializing text: {:?}; text: {}", why, payload);

                why
            })?,
            Message::Close(Some(frame)) => {
                return Err(Error::Gateway(GatewayError::Closed(Some(frame))));
            },
            _ => return Ok(None),
        };

        Ok(Some(value))
    }

    pub(crate) async fn send_json(&mut self, value: &Value) -> Result<()> {
        let message = to_string(value).map(Message::Text)?;

        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `StreamExt::next`
    pub(crate) async fn next(&mut self) -> Option<std::result::Result<Message, WsError>> {
        self.0.next().await
    }

    /// Delegate to `SinkExt::send`
    pub(crate) async fn send(&mut self, message: Message) -> Result<()> {
        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    pub(crate) async fn close(&mut self, msg: Option<CloseFrame<'_>>) -> Result<()> {
        self.0.close(msg).await?;
        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn send_chunk_guild(
        &mut self,
        guild_id: GuildId,
        shard_info: &[u64; 2],
        limit: Option<u16>,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[Shard {:?}] Requesting member chunks", shard_info);

        let mut payload = json!({
            "op": Opcode::RequestGuildMembers,
            "d": {
                "guild_id": guild_id.as_ref().0.to_string(),
                "limit": limit.unwrap_or(0),
                "nonce": nonce.unwrap_or(""),
            },
        });

        match filter {
            ChunkGuildFilter::None => payload["d"]["query"] = json!(""),
            ChunkGuildFilter::Query(query) => payload["d"]["query"] = json!(query),
            ChunkGuildFilter::UserIds(user_ids) => {
                let ids = user_ids.iter().map(|x| x.0).collect::<Vec<u64>>();
                payload["d"]["user_ids"] = json!(ids);
            },
        };

        self.send_json(&payload).await.map_err(From::from)
    }

    #[instrument(skip(self))]
    pub async fn send_heartbeat(&mut self, shard_info: &[u64; 2], seq: Option<u64>) -> Result<()> {
        trace!("[Shard {:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&json!({
            "op": Opcode::Heartbeat,
            "d": seq,
        }))
        .await
        .map_err(From::from)
    }

    #[instrument(skip(self, token))]
    pub async fn send_identify(
        &mut self,
        shard_info: &[u64; 2],
        token: &str,
        intents: GatewayIntents,
    ) -> Result<()> {
        debug!("[Shard {:?}] Identifying", shard_info);

        self.send_json(&json!({
            "op": Opcode::Identify,
            "d": {
                "compress": true,
                "large_threshold": constants::LARGE_THRESHOLD,
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
        }))
        .await
    }

    #[instrument(skip(self))]
    pub async fn send_presence_update(
        &mut self,
        shard_info: &[u64; 2],
        current_presence: &CurrentPresence,
    ) -> Result<()> {
        let &(ref activity, ref status) = current_presence;
        let now = SystemTime::now();

        debug!("[Shard {:?}] Sending presence update", shard_info);

        self.send_json(&json!({
            "op": Opcode::PresenceUpdate,
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
        .await
    }

    #[instrument(skip(self, token))]
    pub async fn send_resume(
        &mut self,
        shard_info: &[u64; 2],
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[Shard {:?}] Sending resume; seq: {}", shard_info, seq);

        self.send_json(&json!({
            "op": Opcode::Resume,
            "d": {
                "session_id": session_id,
                "seq": seq,
                "token": token,
            },
        }))
        .await
        .map_err(From::from)
    }
}
