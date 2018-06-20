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
    long_lock::LongLock,
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
use tokio_core::reactor::{Core, Handle, Remote};
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
        println!("Killing loop (?)");
        match mem::replace(&mut self.kill_tx, None) {
            Some(tx) => {let _ = tx.send(());},
            None => {},
        };

        Ok(())
    }
}

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>, remote: Remote) {
    let timer = wheel()
        .tick_duration(Duration::from_millis(20))
        .build();

    // TODO: reveal to the outside world
    println!("Built runner.");
    remote.spawn(move |handle| runner(rx, timer, handle.remote().clone()));
}

fn runner(rx: MpscReceiver<Status>, timer: Timer, remote: Remote) -> impl Future<Item = (), Error = ()> {
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
                println!("tick");
                // NOTE: might want to make late tasks die early.
                // May need to store task spawn times etc.

                let mut received_info = None;
                let mut should_disconnect = false;
                let conn_handle = handle.clone();

                let mut state = LongLock::new(state_lock);

                // Handle any control messages, drain them all synchronously.
                // All are obvious, except connection state changes, which
                // we want to collect to minimise spurious sub-frame channel changes.
                loop {
                    match state.rx.try_recv() {
                        Ok(Status::Connect(info)) => {
                            println!("Connection request.");
                            received_info = Some(info);
                            should_disconnect = false;
                        },
                        Ok(Status::Disconnect) => {
                            println!("Disconnection request.");
                            should_disconnect = true;
                        },
                        Ok(Status::SetReceiver(r)) => {
                            println!("Receiver added.");
                            state.receiver = r;
                        },
                        Ok(Status::SetSender(s)) => {
                            println!("Sender set.");
                            state.senders.clear();

                            if let Some(aud) = s {
                                state.senders.push(aud);
                            }
                        },
                        Ok(Status::AddSender(s)) => {
                            println!("Sender added.");
                            state.senders.push(s);
                        },
                        Ok(Status::SetBitrate(b)) => {
                            println!("Bitrate set.");
                            state.bitrate = b;
                        },
                        Err(TryRecvError::Empty) => {
                            // If we receieved nothing, then we can perform an update.
                            println!("Forwards!");
                            break;
                        },
                        Err(TryRecvError::Disconnected) => {
                            println!("Rip!");
                            should_disconnect = true;
                            break;
                        },
                    }
                }

                // If we *want* to disconnect, poison the stream here.
                if should_disconnect {
                    println!("Rip2!");
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
                            Either4::A(connection.reconnect(conn_handle).map(Some))
                        } else {
                            Either4::B(ok(Some(connection)))
                        },
                    None => if let Some(info) = received_info {
                        Either4::C(Connection::new(info, conn_handle).map(Some))
                    } else {
                        println!("Bad bad not good");
                        Either4::D(ok(None))
                    },
                };

                Either::B(conn_future
                    .and_then(move |conn| {
                        println!("About to maybe cycle...");
                        // This drops the lock on completion.
                        // The cycle is responsible for setting/unsetting the error flag.
                        match conn {
                            Some(conn) => Either::A(conn.cycle(instant, state)),
                            None => Either::B(ok(state)),
                        }
                    })
                    .map(|state| mem::drop(state))
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
