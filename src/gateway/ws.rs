use std::env::consts;
#[cfg(feature = "client")]
use std::io::Read;
use std::time::SystemTime;

#[cfg(feature = "client")]
use flate2::read::ZlibDecoder;
use futures::SinkExt;
#[cfg(feature = "client")]
use futures::StreamExt;
#[cfg(feature = "client")]
use small_fixed_array::FixedString;
use tokio::net::TcpStream;
#[cfg(feature = "client")]
use tokio::time::{timeout, Duration};
#[cfg(feature = "client")]
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
#[cfg(feature = "client")]
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_with_config, MaybeTlsStream, WebSocketStream};
#[cfg(feature = "client")]
use tracing::warn;
use tracing::{debug, trace};
use url::Url;

use super::{ActivityData, ChunkGuildFilter, PresenceData};
use crate::constants::{self, Opcode};
#[cfg(feature = "client")]
use crate::gateway::GatewayError;
#[cfg(feature = "client")]
use crate::model::event::GatewayEvent;
use crate::model::gateway::{GatewayIntents, ShardInfo};
use crate::model::id::{GuildId, UserId};
#[cfg(feature = "client")]
use crate::Error;
use crate::Result;

#[derive(Serialize)]
struct IdentifyProperties {
    browser: &'static str,
    device: &'static str,
    os: &'static str,
}

#[derive(Serialize)]
struct ChunkGuildMessage<'a> {
    guild_id: GuildId,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a str>,
    limit: u16,
    presences: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_ids: Option<Vec<UserId>>,
    nonce: &'a str,
}

#[derive(Serialize)]
struct PresenceUpdateMessage<'a> {
    afk: bool,
    status: &'a str,
    since: SystemTime,
    activities: &'a [ActivityData],
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
        presence: PresenceUpdateMessage<'a>,
    },
    PresenceUpdate(PresenceUpdateMessage<'a>),
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

pub struct WsClient(WebSocketStream<MaybeTlsStream<TcpStream>>);

#[cfg(feature = "client")]
const TIMEOUT: Duration = Duration::from_millis(500);
#[cfg(feature = "client")]
const DECOMPRESSION_MULTIPLIER: usize = 3;

impl WsClient {
    pub(crate) async fn connect(url: Url) -> Result<Self> {
        let config = WebSocketConfig {
            max_message_size: None,
            max_frame_size: None,
            ..Default::default()
        };
        let (stream, _) = connect_async_with_config(url, Some(config), false).await?;

        Ok(Self(stream))
    }

    #[cfg(feature = "client")]
    pub(crate) async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.0.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => return Ok(None),
        };

        let json_str = match message {
            Message::Text(payload) => payload,
            Message::Binary(bytes) => {
                let mut decompressed =
                    String::with_capacity(bytes.len() * DECOMPRESSION_MULTIPLIER);

                ZlibDecoder::new(&bytes[..]).read_to_string(&mut decompressed).map_err(|why| {
                    warn!("Err decompressing bytes: {why:?}");
                    debug!("Failing bytes: {bytes:?}");

                    why
                })?;

                decompressed
            },
            Message::Close(Some(frame)) => {
                return Err(Error::Gateway(GatewayError::Closed(Some(frame))));
            },
            _ => return Ok(None),
        };

        match serde_json::from_str(&json_str) {
            Ok(mut event) => {
                if let GatewayEvent::Dispatch {
                    original_str, ..
                } = &mut event
                {
                    *original_str = FixedString::from_string_trunc(json_str);
                }

                Ok(Some(event))
            },
            Err(err) => {
                debug!("Failing text: {json_str}");
                Err(Error::Json(err))
            },
        }
    }

    pub(crate) async fn send_json(&mut self, value: &impl serde::Serialize) -> Result<()> {
        let message = serde_json::to_string(value).map(Message::Text)?;

        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `StreamExt::next`
    #[cfg(feature = "client")]
    pub(crate) async fn next(&mut self) -> Option<std::result::Result<Message, WsError>> {
        self.0.next().await
    }

    /// Delegate to `SinkExt::send`
    #[cfg(feature = "client")]
    pub(crate) async fn send(&mut self, message: Message) -> Result<()> {
        self.0.send(message).await?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    #[cfg(feature = "client")]
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
        presences: bool,
        filter: ChunkGuildFilter,
        nonce: Option<&str>,
    ) -> Result<()> {
        debug!("[{:?}] Requesting member chunks", shard_info);

        let (query, user_ids) = match filter {
            ChunkGuildFilter::None => (Some(String::new()), None),
            ChunkGuildFilter::Query(query) => (Some(query), None),
            ChunkGuildFilter::UserIds(user_ids) => (None, Some(user_ids)),
        };

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestGuildMembers,
            d: WebSocketMessageData::ChunkGuild(ChunkGuildMessage {
                guild_id,
                query: query.as_deref(),
                limit: limit.unwrap_or(0),
                presences,
                user_ids,
                nonce: nonce.unwrap_or(""),
            }),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn send_heartbeat(&mut self, shard_info: &ShardInfo, seq: Option<u64>) -> Result<()> {
        trace!("[{:?}] Sending heartbeat d: {:?}", shard_info, seq);

        self.send_json(&WebSocketMessage {
            op: Opcode::Heartbeat,
            d: WebSocketMessageData::Heartbeat(seq),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, token)))]
    pub async fn send_identify(
        &mut self,
        shard: &ShardInfo,
        token: &str,
        intents: GatewayIntents,
        presence: &PresenceData,
    ) -> Result<()> {
        let now = SystemTime::now();
        let activities = presence.activity.as_ref().map(std::slice::from_ref).unwrap_or_default();

        debug!("[{:?}] Identifying", shard);

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
                presence: PresenceUpdateMessage {
                    afk: false,
                    since: now,
                    status: presence.status.name(),
                    activities,
                },
            },
        };

        self.send_json(&msg).await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn send_presence_update(
        &mut self,
        shard_info: &ShardInfo,
        presence: &PresenceData,
    ) -> Result<()> {
        let now = SystemTime::now();
        let activities = presence.activity.as_ref().map(std::slice::from_ref).unwrap_or_default();

        debug!("[{shard_info:?}] Sending presence update");

        self.send_json(&WebSocketMessage {
            op: Opcode::PresenceUpdate,
            d: WebSocketMessageData::PresenceUpdate(PresenceUpdateMessage {
                afk: false,
                since: now,
                activities,
                status: presence.status.name(),
            }),
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self, token)))]
    pub async fn send_resume(
        &mut self,
        shard_info: &ShardInfo,
        session_id: &str,
        seq: u64,
        token: &str,
    ) -> Result<()> {
        debug!("[{:?}] Sending resume; seq: {}", shard_info, seq);

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
