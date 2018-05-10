use future_utils::StreamExt;
use futures::{
    future::{err, ok},
    sync::oneshot,
    Future,
    Stream,
};
use internal::{
    either_n::{Either3, Either4},
    prelude::*,
};
use model::id::GuildId;
use parking_lot::{Mutex, MutexGuard};
use std::mem;
use std::sync::{mpsc::{Receiver as MpscReceiver, TryRecvError}, Arc,};
use std::thread::Builder as ThreadBuilder;
use std::time::{Duration, Instant};
use super::connection::Connection;
use super::{audio, error::VoiceError, Bitrate, Status};
use tokio_core::reactor::{Core, Handle, Remote};
use tokio_timer::{wheel, Timer};

pub(crate) struct TaskState {
    pub bitrate: Bitrate,
    connection: Option<Connection>,
    cycle_error: bool,
    kill_tx: oneshot::Sender<()>,
    pub receiver: Option<Box<audio::AudioReceiver>>,
    pub senders: Vec<audio::LockedAudio>,
}

impl TaskState {
    pub fn conn(&mut self) -> Connection {
        self.maybe_conn()
            .expect("[voice] Failed to get udp...")
    }

    pub fn maybe_conn(&mut self) -> Option<Connection> {
        mem::replace(&mut self.connection, None)
    }

    pub fn restore_conn(&mut self, conn: Connection) -> &mut Self {
        self.connection = Some(conn);
        self
    }
}

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice (G{})", guild_id);

    let timer = wheel()
        .tick_duration(Duration::from_millis(20))
        .build();

    // TODO: reveal to the outside world
    runner(rx, timer);
}

fn runner(rx: MpscReceiver<Status>, timer: Timer) -> impl Future<Item = (), Error = ()>{
    let (kill_tx, kill_rx) = oneshot::channel();

    let mut state_lock = Arc::new(Mutex::new(TaskState {
        bitrate: Bitrate::Bits(audio::DEFAULT_BITRATE),
        connection: None,
        cycle_error: false,
        kill_tx,
        receiver: None,
        senders: Vec::new(),
    }));

    // TEMP: FIX ME
    let mut core = Core::new().unwrap();
    let remote = core.handle().remote().clone();

    let out = timer.interval_at(Instant::now(), Duration::from_millis(20))
        .map_err(|why| {error!("[voice] Timer error for running connection. {:?}", why)})
        .until(
            kill_rx.map_err(|why| {
                error!("[voice] Oneshot error for voice connection poison. {:?}", why)
            })
        )
        .map(|()| Instant::now())
        .for_each(move |instant| {
            // NOTE: might want to make late tasks die early.
            // May need to store task spawn times etc.

            let mut received_info = None;
            let mut should_disconnect = false;

            {
                // Handle any control messages, drain them all synchronously.
                // All are obvious, except connection state changes, which
                // we want to collect to minimise spurious sub-frame channel changes.
                let mut state = &mut state_lock.lock();

                loop {
                    match rx.try_recv() {
                        Ok(Status::Connect(info)) => {
                            received_info = Some(info);
                            should_disconnect = false;
                        },
                        Ok(Status::Disconnect) => {
                            should_disconnect = true;
                        },
                        Ok(Status::SetReceiver(r)) => {
                            state.receiver = r;
                        },
                        Ok(Status::SetSender(s)) => {
                            state.senders.clear();

                            if let Some(aud) = s {
                                state.senders.push(aud);
                            }
                        },
                        Ok(Status::AddSender(s)) => {
                            state.senders.push(s);
                        },
                        Ok(Status::SetBitrate(b)) => {
                            state.bitrate = b;
                        },
                        Err(TryRecvError::Empty) => {
                            // If we receieved nothing, then we can perform an update.
                            break;
                        },
                        Err(TryRecvError::Disconnected) => {
                            should_disconnect = true;
                            break;
                        },
                    }
                }

                // If we *want* to disconnect, poison the stream here.
                if should_disconnect {
                    return state.kill_tx.send(());
                }
            }

            remote.spawn(move |handle| {
                let mut state = state_lock.lock();
                let conn_handle = handle.clone();

                // Now we know what to do with the connection.
                // There are 3 cases:
                //  * There was a error on a conn last time, reconnect cheaply.
                //    If that fails, completely reconnect.
                //  * We already have a connection.
                //  * We want to make a new connection.
                let conn_future = match state.maybe_conn() {
                    Some(connection) => if state.cycle_error {
                            Either4::One(connection.reconnect(conn_handle)
                                .map(|conn| MutexGuard::map(state, |a| a.restore_conn(conn))))
                        } else {
                            Either4::Two(ok(MutexGuard::map(state, |a| a.restore_conn(connection))))
                        },
                    None => if let Some(info) = received_info {
                        Either4::Three(Connection::new(info, conn_handle)
                            .map(|conn| MutexGuard::map(state, |a| a.restore_conn(conn))))
                    } else {
                        Either4::Four(err(Error::Voice(VoiceError::VoiceModeInvalid)))
                    },
                };

                let out = conn_future
                    .and_then(move |mut state| {
                        // TODO: set this on error
                        state.cycle_error = false;

                        // This drops the lock.
                        state.conn().cycle(state)
                    })
                    .map_err(|why| {
                        error!(
                            "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                            why
                        )
                    });

                out
            });

            Ok(())
        }
        .map(|_| ()));

    Box::new(out)
}
