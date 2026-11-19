use std::env::consts;
use std::io::{self, ErrorKind, Read};
use std::time::{Duration, SystemTime};

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use flate2::read::ZlibDecoder;
#[cfg(feature = "transport_compression_zlib")]
use flate2::write::ZlibDecoder as ZlibWriter;
use futures::{SinkExt, StreamExt};
use hyper::Request;
use hyper::client::conn::http1;
use hyper::header::{HOST, PROXY_AUTHORIZATION};
use hyper_util::rt::TokioIo;
use percent_encoding::percent_decode_str;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use tokio_tungstenite::tungstenite::protocol::{
    CloseFrame as TungsteniteCloseFrame,
    WebSocketConfig,
};
use tokio_tungstenite::{
    MaybeTlsStream,
    WebSocketStream,
    client_async_tls_with_config,
    connect_async_with_config,
};
#[cfg(feature = "tracing_instrument")]
use tracing::instrument;
use tracing::{debug, trace, warn};
use url::Url;
#[cfg(feature = "transport_compression_zstd")]
use zstd::stream::write::Decoder as ZstdWriter;

use super::{
    ActivityData,
    ChunkGuildFilter,
    CloseFrame,
    GatewayError,
    GatewayTransportError,
    PresenceData,
    TransportCompression,
};
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

fn transport_err_to_lib(err: TungsteniteError) -> Error {
    Error::Gateway(GatewayError::Transport(GatewayTransportError::new(err)))
}

impl WsClient {
    pub(crate) async fn connect(
        url: Url,
        proxy: Option<&Url>,
        compression: TransportCompression,
    ) -> Result<Self> {
        let config = {
            let mut config = WebSocketConfig::default();
            config.max_message_size = None;
            config.max_frame_size = None;

            config
        };

        let (stream, _) = match proxy {
            Some(proxy) => {
                let tunnel = Self::connect_through_proxy(&url, proxy).await?;
                client_async_tls_with_config(url, tunnel, Some(config), None).await
            },
            None => connect_async_with_config(url, Some(config), false).await,
        }
        .map_err(transport_err_to_lib)?;

        Ok(Self {
            stream,
            compression: compression.into(),
        })
    }

    async fn connect_through_proxy(target: &Url, proxy: &Url) -> Result<TcpStream> {
        if proxy.scheme() != "http" {
            let why = io::Error::new(ErrorKind::Unsupported, "only http proxies are supported");
            return Err(Error::Io(why));
        }

        let (Some(target_host), Some(target_port), Some(proxy_host), Some(proxy_port)) = (
            target.host_str(),
            target.port_or_known_default(),
            proxy.host_str(),
            proxy.port_or_known_default(),
        ) else {
            return Err(Error::Gateway(GatewayError::BuildingUrl));
        };

        let authority = format!("{target_host}:{target_port}");
        let mut request = Request::connect(&authority).header(HOST, &authority);

        let username = percent_decode_str(proxy.username()).decode_utf8_lossy();
        if !username.is_empty() {
            let password = percent_decode_str(proxy.password().unwrap_or_default());
            let credentials = format!("{username}:{}", password.decode_utf8_lossy());
            let basic_auth = format!("Basic {}", BASE64_STANDARD.encode(credentials));
            request = request.header(PROXY_AUTHORIZATION, basic_auth);
        }
        let request = request.body(String::new()).map_err(io::Error::other)?;

        let stream = TcpStream::connect(format!("{proxy_host}:{proxy_port}")).await?;
        let (mut sender, conn) =
            http1::handshake(TokioIo::new(stream)).await.map_err(io::Error::other)?;

        let response = sender.send_request(request);
        // Without a live sender the connection finishes once the response is read
        drop(sender);
        let (response, conn) = tokio::join!(response, conn.with_upgrades());
        let response = response.map_err(io::Error::other)?;
        conn.map_err(io::Error::other)?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::Io(io::Error::other(format!("proxy refused to tunnel: {status}"))));
        }

        let upgraded = hyper::upgrade::on(response).await.map_err(io::Error::other)?;
        let parts = upgraded
            .downcast::<TokioIo<TcpStream>>()
            .map_err(|_| io::Error::other("unexpected stream type after proxy upgrade"))?;
        if !parts.read_buf.is_empty() {
            return Err(Error::Io(io::Error::other("proxy sent data before the tunnel was used")));
        }

        Ok(parts.io.into_inner())
    }

    pub(crate) async fn recv_json(&mut self) -> Result<Option<GatewayEvent>> {
        let message = match timeout(TIMEOUT, self.stream.next()).await {
            Ok(Some(Ok(msg))) => msg,
            Ok(Some(Err(e))) => return Err(transport_err_to_lib(e)),
            Ok(None) | Err(_) => return Ok(None),
        };

        let json_bytes = match message {
            Message::Text(ref payload) => payload.as_bytes(),
            Message::Binary(ref bytes) => match self.compression.inflate(bytes)? {
                Some(decompressed) => decompressed,
                None => return Ok(None),
            },
            Message::Close(Some(frame)) => {
                return Err(Error::Gateway(GatewayError::Closed(Some(CloseFrame::new(frame)))));
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

        self.stream.send(message).await.map_err(transport_err_to_lib)?;
        Ok(())
    }

    /// Delegate to `WebSocketStream::close`
    pub(crate) async fn close(&mut self, msg: Option<TungsteniteCloseFrame>) -> Result<()> {
        self.stream.close(msg).await.map_err(transport_err_to_lib)?;
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
