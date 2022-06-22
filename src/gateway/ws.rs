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
use crate::json::{from_str, to_string};
use crate::model::event::GatewayEvent;
use crate::model::gateway::{ActivityType, GatewayIntents, ShardInfo};
use crate::model::id::{GuildId, UserId};
use crate::{Error, Result};

#[derive(Serialize)]
struct IdentifyProperties {
    browser: &'static str,
    device: &'static str,
    os: &'static str,
}

#[derive(Serialize)]
struct ActivityData<'a> {
    #[serde(rename = "type")]
    kind: ActivityType,
    url: &'a Option<Url>,
    name: &'a str,
}

#[derive(Serialize)]
struct ChunkGuildMessage<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<UserId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    guild_id: GuildId,
    nonce: &'a str,
    limit: u16,
}

#[derive(Serialize)]
#[serde(untagged)]
enum WebSocketMessageData<'a> {
    Heartbeat(Option<u64>),
    ChunkGuild(ChunkGuildMessage<'a>),
    Identify {
        compress: bool,
        token: &'a str,
        large_threshold: u8,
        shard: &'a ShardInfo,
        intents: GatewayIntents,
        properties: IdentifyProperties,
    },
    PresenceUpdate {
        afk: bool,
        status: &'a str,
        since: SystemTime,
        game: Option<ActivityData<'a>>,
    },
    Resume {
        session_id: &'a str,
        token: &'a str,
        seq: u64,
    },
}

#[derive(Serialize)]
struct WebSocketMessage<'a> {
    op: Opcode,
    d: WebSocketMessageData<'a>,
}

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

    pub(crate) async fn send_json(&mut self, value: &impl serde::Serialize) -> Result<()> {
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
        shard_info: &ShardInfo,
        limit: Option<u16>,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[Shard {:?}] Requesting member chunks", shard_info);

        let (query, user_ids) = match filter {
            ChunkGuildFilter::None => (Some(String::new()), None),
            ChunkGuildFilter::Query(query) => (Some(query), None),
            ChunkGuildFilter::UserIds(user_ids) => (None, Some(user_ids)),
        };

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestGuildMembers,
            d: WebSocketMessageData::ChunkGuild(ChunkGuildMessage {
                guild_id,
                user_ids,
                query: query.as_deref(),
                limit: limit.unwrap_or(0),
                nonce: nonce.unwrap_or(""),
            }),
        })
        .await
    }

    #[instrument(skip(self))]
    pub async fn send_heartbeat(&mut self, shard_info: &ShardInfo, seq: Option<u64>) -> Result<()> {
        trace!("[Shard {:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Heartbeat,
            d: WebSocketMessageData::Heartbeat(seq),
        })
        .await
    }

    #[instrument(skip(self, token))]
    pub async fn send_identify(
        &mut self,
        shard: &ShardInfo,
        token: &str,
        intents: GatewayIntents,
    ) -> Result<()> {
        debug!("[Shard {:?}] Identifying", shard);

        let msg = WebSocketMessage {
            op: Opcode::Identify,
            d: WebSocketMessageData::Identify {
                token,
                shard,
                intents,
                compress: true,
                large_threshold: constants::LARGE_THRESHOLD,
                properties: IdentifyProperties {
                    browser: "serenity",
                    device: "serenity",
                    os: consts::OS,
                },
            },
        };

        self.send_json(&msg).await
    }

    #[instrument(skip(self))]
    pub async fn send_presence_update(
        &mut self,
        shard_info: &ShardInfo,
        current_presence: &CurrentPresence,
    ) -> Result<()> {
        let &(ref activity, ref status) = current_presence;
        let now = SystemTime::now();

        debug!("[Shard {:?}] Sending presence update", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::PresenceUpdate,
            d: WebSocketMessageData::PresenceUpdate {
                afk: false,
                since: now,
                status: status.name(),
                game: activity.as_ref().map(|x| ActivityData {
                    name: &x.name,
                    kind: x.kind,
                    url: &x.url,
                }),
            },
        })
        .await
    }

    #[instrument(skip(self, token))]
    pub async fn send_resume(
        &mut self,
        shard_info: &ShardInfo,
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[Shard {:?}] Sending resume; seq: {}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Resume,
            d: WebSocketMessageData::Resume {
                session_id,
                token,
                seq,
            },
        })
        .await
    }
}
