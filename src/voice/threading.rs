use future_utils::StreamExt;
use futures::{
    future::{err, ok, result, Either},
    stream::repeat,
    sync::oneshot,
    Future,
    Stream,
};
use internal::{
    either_n::Either4,
    long_lock::long_lock,
    prelude::*,
};
use model::id::GuildId;
use parking_lot::Mutex;
use std::{
    mem,
    sync::{
        mpsc::{Receiver as MpscReceiver, TryRecvError},
        Arc
    },
    time::{Duration, Instant},
};
use super::{
    audio,
    connection::Connection,
    error::VoiceError,
    Bitrate,
    Status
};
use tokio_core::reactor::Core;
use tokio_timer::{wheel, Timer};

pub(crate) struct TaskState {
    pub bitrate: Bitrate,
    connection: Option<Connection>,
    pub cycle_error: bool,
    kill_tx: Option<oneshot::Sender<()>>,
    pub receiver: Option<Box<audio::AudioReceiver>>,
    rx: MpscReceiver<Status>,
    pub senders: Vec<audio::LockedAudio>,
}

impl TaskState {
    pub fn maybe_conn(&mut self) -> Option<Connection> {
        mem::replace(&mut self.connection, None)
    }

    pub fn restore_conn(&mut self, conn: Connection) -> &mut Self {
        self.connection = Some(conn);
        self
    }

    pub fn kill_loop(&mut self) -> StdResult<(),()> {
        match mem::replace(&mut self.kill_tx, None) {
            Some(tx) => {let _ = tx.send(());},
            None => {},
        };

        Ok(())
    }
}

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let timer = wheel()
        .tick_duration(Duration::from_millis(20))
        .build();

    // TODO: reveal to the outside world
    runner(rx, timer);
}

fn runner(rx: MpscReceiver<Status>, timer: Timer) -> impl Future<Item = (), Error = ()> {
    let (kill_tx, kill_rx) = oneshot::channel();

    let state_shared = repeat(Arc::new(Mutex::new(TaskState {
        bitrate: Bitrate::Bits(audio::DEFAULT_BITRATE),
        connection: None,
        cycle_error: false,
        kill_tx: Some(kill_tx),
        receiver: None,
        rx,
        senders: Vec::new(),
    })));

    // TEMP: FIX ME
    let mut core = Core::new().unwrap();
    let remote = core.handle().remote().clone();

    timer.interval_at(Instant::now(), Duration::from_millis(20))
        .map_err(|why| {error!("[voice] Timer error for running connection. {:?}", why)})
        .until(
            kill_rx.map_err(|why| {
                error!("[voice] Oneshot error for voice connection poison. {:?}", why)
            })
        )
        .map(|()| Instant::now())
        .zip(state_shared)
        .for_each(move |(instant, state_lock)| {
            remote.spawn(move |handle| {
                // NOTE: might want to make late tasks die early.
                // May need to store task spawn times etc.

                let mut received_info = None;
                let mut should_disconnect = false;
                let conn_handle = handle.clone();

                let mut state = long_lock(state_lock);

                // Handle any control messages, drain them all synchronously.
                // All are obvious, except connection state changes, which
                // we want to collect to minimise spurious sub-frame channel changes.
                loop {
                    match state.rx.try_recv() {
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
                    return Either::A(result(state.kill_loop()));
                }

                // Now we know what to do with the connection.
                // There are 3 cases:
                //  * There was a error on a conn last time, reconnect cheaply.
                //    If that fails, completely reconnect.
                //  * We already have a connection.
                //  * We want to make a new connection.
                let conn_future = match state.maybe_conn() {
                    Some(connection) => if state.cycle_error {
                            Either4::A(connection.reconnect(conn_handle))
                        } else {
                            Either4::B(ok(connection))
                        },
                    None => if let Some(info) = received_info {
                        Either4::C(Connection::new(info, conn_handle))
                    } else {
                        Either4::D(err(Error::Voice(VoiceError::VoiceModeInvalid)))
                    },
                };

                Either::B(conn_future
                    .and_then(move |conn| {
                        // This drops the lock on completion.
                        // The cycle is responsible for setting/unsetting the error flag.
                        conn.cycle(instant, state)
                    })
                    .map_err(|why| {
                        error!(
                            "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                            why
                        )
                    }))
            });
            
            Ok(())
        })
}
