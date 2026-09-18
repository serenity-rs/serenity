use std::env::consts;
use std::io::Read;
use std::time::{Duration, SystemTime};

use flate2::read::ZlibDecoder;
#[cfg(feature = "transport_compression_zlib")]
use flate2::write::ZlibDecoder as ZlibWriter;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_with_config};
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, trace, warn};
use url::Url;
#[cfg(feature = "transport_compression_zstd")]
use zstd::stream::write::Decoder as ZstdWriter;

use super::{ActivityData, ChunkGuildFilter, GatewayError, PresenceData, TransportCompression};
use crate::constants::{self, Opcode};
use crate::model::event::GatewayEvent;
use crate::model::gateway::{GatewayCapabilities, GatewayIntents, ShardInfo};
#[cfg(feature = "voice")]
use crate::model::id::ChannelId;
use crate::model::id::{GuildId, UserId};
use crate::{Error, Result};

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
    SoundboardSounds {
        guild_ids: &'a [GuildId],
    },
    Identify {
        compress: bool,
        token: &'a str,
        large_threshold: u8,
        shard: &'a ShardInfo,
        intents: GatewayIntents,
        capabilities: Option<GatewayCapabilities>,
        properties: IdentifyProperties,
        presence: PresenceUpdateMessage<'a>,
    },
    ChannelInfo {
        guild_id: GuildId,
        fields: &'a [&'a str],
    },
    #[cfg(feature = "voice")]
    VoiceStateUpdate {
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
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

enum Compression {
    Payload {
        decompressed: Vec<u8>,
    },

    #[cfg(feature = "transport_compression_zlib")]
    Zlib {
        decoder: ZlibWriter<Vec<u8>>,
        compressed: Vec<u8>,
    },

    #[cfg(feature = "transport_compression_zstd")]
    Zstd {
        decoder: ZstdWriter<'static, Vec<u8>>,
    },
}

impl Compression {
    const DECOMPRESSED_CAPACITY: usize = 174_504;

    fn inflate(&mut self, slice: &[u8]) -> Result<Option<&[u8]>> {
        match self {
            Compression::Payload {
                decompressed,
            } => {
                decompressed.clear();
                decompressed.reserve(Self::DECOMPRESSED_CAPACITY);

                ZlibDecoder::new(slice).read_to_end(decompressed).map_err(|why| {
                    warn!("Err decompressing bytes: {why:?}");
                    why
                })?;

                Ok(Some(decompressed.as_slice()))
            },

            #[cfg(feature = "transport_compression_zlib")]
            Compression::Zlib {
                decoder,
                compressed,
            } => {
                use std::io::Write;

                const ZLIB_SUFFIX: [u8; 4] = [0x00, 0x00, 0xFF, 0xFF];

                compressed.extend_from_slice(slice);

                let len = compressed.len();

                if len < 4 || compressed[len - 4..] != ZLIB_SUFFIX {
                    return Ok(None);
                }

                decoder.get_mut().clear();
                decoder.write_all(compressed).map_err(|why| {
                    warn!("Err decompressing bytes: {why:?}");
                    why
                })?;
                decoder.flush()?;
                compressed.clear();

                Ok(Some(decoder.get_ref().as_slice()))
            },

            #[cfg(feature = "transport_compression_zstd")]
            Compression::Zstd {
                decoder,
            } => {
                use std::io::Write;

                decoder.get_mut().clear();
                decoder.write_all(slice).map_err(|why| {
                    warn!("Err decompressing bytes: {why:?}");
                    why
                })?;
                decoder.flush()?;

                Ok(Some(decoder.get_ref().as_slice()))
            },
        }
    }
}

impl From<TransportCompression> for Compression {
    fn from(value: TransportCompression) -> Self {
        match value {
            TransportCompression::None => Compression::Payload {
                decompressed: Vec::new(),
            },

            #[cfg(feature = "transport_compression_zlib")]
            TransportCompression::Zlib => Compression::Zlib {
                decoder: ZlibWriter::new(Vec::with_capacity(Self::DECOMPRESSED_CAPACITY)),
                compressed: Vec::new(),
            },

            #[cfg(feature = "transport_compression_zstd")]
            TransportCompression::Zstd => Compression::Zstd {
                decoder: ZstdWriter::new(Vec::with_capacity(Self::DECOMPRESSED_CAPACITY))
                    .expect("Failed to initialize Zstd decoder"),
            },
        }
    }
}

pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    compression: Compression,
}

const TIMEOUT: Duration = Duration::from_millis(500);

impl WsClient {
    pub(crate) async fn connect(url: Url, compression: TransportCompression) -> Result<Self> {
        let config = {
            let mut config = WebSocketConfig::default();
            config.max_message_size = None;
            config.max_frame_size = None;

            config
        };

        let (stream, _) = connect_async_with_config(url, Some(config), false).await?;

        Ok(Self {
            stream,
            compression: compression.into(),
        })
    }

    pub(crate) async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.stream.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(None) | Err(_) => return Ok(None),
        };

        let json_bytes = match message {
            Message::Text(ref payload) => payload.as_bytes(),
            Message::Binary(ref bytes) => match self.compression.inflate(bytes)? {
                Some(decompressed) => decompressed,
                None => return Ok(None),
            },
            Message::Close(Some(frame)) => {
                return Err(Error::Gateway(GatewayError::Closed(Some(Box::new(frame)))));
            },
            _ => return Ok(None),
        };

        match serde_json::from_slice(json_bytes) {
            Ok(event) => Ok(Some(event)),
            Err(err) => {
                debug!("Failing text: {}", String::from_utf8_lossy(json_bytes));
                Err(Error::Json(err))
            },
        }
    }

    pub(crate) async fn send_json(&mut self, value: &impl serde::Serialize) -> Result<()> {
        let message = Message::Text(serde_json::to_string(value)?.into());

        self.stream.send(message).await?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    pub(crate) async fn close(&mut self, msg: Option<CloseFrame>) -> Result<()> {
        self.stream.close(msg).await?;
        Ok(())
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
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
    pub async fn request_soundboard_sounds(
        &mut self,
        guild_ids: &[GuildId],
        shard_info: &ShardInfo,
    ) -> Result<()> {
        debug!("[{:?}] Requesting soundboard sounds", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestSoundboardSounds,
            d: WebSocketMessageData::SoundboardSounds {
                guild_ids,
            },
        })
        .await
    }

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    pub async fn request_channel_info(
        &mut self,
        shard_info: &ShardInfo,
        guild_id: GuildId,
        fields: &[&str],
    ) -> Result<()> {
        debug!("[{:?}] Requesting channel info", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::RequestChannelInfo,
            d: WebSocketMessageData::ChannelInfo {
                guild_id,
                fields,
            },
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
        capabilities: Option<GatewayCapabilities>,
        presence: &PresenceData,
    ) -> Result<()> {
        let now = SystemTime::now();
        let activities = presence.activity.as_slice();

        debug!("[{:?}] Identifying", shard);

        let msg = WebSocketMessage {
            op: Opcode::Identify,
            d: WebSocketMessageData::Identify {
                token,
                shard,
                intents,
                capabilities,
                compress: matches!(self.compression, Compression::Payload { .. }),
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
        let activities = presence.activity.as_slice();

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

    /// # Errors
    ///
    /// Errors if there is a problem with the WS connection.
    #[cfg(feature = "voice")]
    #[cfg_attr(feature = "tracing_instrument", instrument(skip(self)))]
    pub async fn send_voice_state_update(
        &mut self,
        shard_info: &ShardInfo,
        guild_id: GuildId,
        channel_id: Option<ChannelId>,
        self_mute: bool,
        self_deaf: bool,
    ) -> Result<()> {
        debug!("[{:?}] Sending voice state update", shard_info);

        self.send_json(&WebSocketMessage {
            op: Opcode::VoiceStateUpdate,
            d: WebSocketMessageData::VoiceStateUpdate {
                guild_id,
                channel_id,
                self_mute,
                self_deaf,
            },
        })
        .await
    }
}
