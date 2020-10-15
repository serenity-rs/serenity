use crate::id::{GuildId, UserId};
use std::fmt;

#[derive(Clone, Debug)]
pub(crate) enum ConnectionProgress {
    Complete(ConnectionInfo),
    Incomplete(Partial),
}

impl ConnectionProgress {
    pub fn new(guild_id: GuildId, user_id: UserId) -> Self {
        ConnectionProgress::Incomplete(Partial {
            guild_id,
            user_id,
            ..Default::default()
        })
    }

    pub(crate) fn apply_state_update(&mut self, session_id: String) -> bool {
        use ConnectionProgress::*;
        match self {
            Complete(c) => {
                let should_reconn = c.session_id != session_id;
                c.session_id = session_id;
                should_reconn
            },
            Incomplete(i) => i
                .apply_state_update(session_id)
                .map(|info| {
                    *self = Complete(info);
                })
                .is_some(),
        }
    }

    pub(crate) fn apply_server_update(&mut self, endpoint: String, token: String) -> bool {
        use ConnectionProgress::*;
        match self {
            Complete(c) => {
                let should_reconn = c.endpoint != endpoint || c.token != token;

                c.endpoint = endpoint;
                c.token = token;

                should_reconn
            },
            Incomplete(i) => i
                .apply_server_update(endpoint, token)
                .map(|info| {
                    *self = Complete(info);
                })
                .is_some(),
        }
    }
}

/// Parameters and information needed to start communicating with Discord's voice servers, either
/// with the Songbird driver, lavalink, or other system.
#[derive(Clone)]
pub struct ConnectionInfo {
    /// URL of the voice websocket gateway server assigned to this call.
    pub endpoint: String,
    /// ID of the target voice channel's parent guild.
    ///
    /// Bots cannot connect to a guildless (i.e., direct message) voice call.
    pub guild_id: GuildId,
    /// Unique string describing this session for validation/authentication purposes.
    pub session_id: String,
    /// Ephemeral secret used to validate the above session.
    pub token: String,
    /// UserID of this bot.
    pub user_id: UserId,
}

impl fmt::Debug for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionInfo")
            .field("endpoint", &self.endpoint)
            .field("guild_id", &self.guild_id)
            .field("session_id", &self.session_id)
            .field("token", &"<secret>")
            .field("user_id", &self.user_id)
            .finish()
    }
}

#[derive(Clone, Default)]
pub(crate) struct Partial {
    pub endpoint: Option<String>,
    pub guild_id: GuildId,
    pub session_id: Option<String>,
    pub token: Option<String>,
    pub user_id: UserId,
}

impl fmt::Debug for Partial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Partial")
            .field("endpoint", &self.endpoint)
            .field("session_id", &self.session_id)
            .field("token_is_some", &self.token.is_some())
            .finish()
    }
}

impl Partial {
    fn finalise(&mut self) -> Option<ConnectionInfo> {
        if self.endpoint.is_some() && self.session_id.is_some() && self.token.is_some() {
            let endpoint = self.endpoint.take().unwrap();
            let session_id = self.session_id.take().unwrap();
            let token = self.token.take().unwrap();

            Some(ConnectionInfo {
                endpoint,
                session_id,
                token,
                guild_id: self.guild_id,
                user_id: self.user_id,
            })
        } else {
            None
        }
    }

    fn apply_state_update(&mut self, session_id: String) -> Option<ConnectionInfo> {
        self.session_id = Some(session_id);

        self.finalise()
    }

    fn apply_server_update(&mut self, endpoint: String, token: String) -> Option<ConnectionInfo> {
        self.endpoint = Some(endpoint);
        self.token = Some(token);

        self.finalise()
    }
}
