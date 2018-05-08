use futures::{future::{err, ok}, Future, Stream};
use internal::{
    either_n::{Either3, Either4},
    prelude::*,
};
use model::id::GuildId;
use parking_lot::Mutex;
use std::sync::{mpsc::{Receiver as MpscReceiver, TryRecvError}, Arc,};
use std::thread::Builder as ThreadBuilder;
use std::time::{Duration, Instant};
use super::connection::Connection;
use super::{audio, error::VoiceError, Bitrate, Status};
use tokio_core::reactor::{Core, Handle, Remote};
use tokio_timer::{wheel, Timer};

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice (G{})", guild_id);

    let timer = wheel()
        .tick_duration(Duration::from_millis(20))
        .build();

    // TODO: reveal to the outside world
    runner(rx, timer);
}

fn runner(rx: MpscReceiver<Status>, timer: Timer) -> Box<Future<Item = (), Error = ()>>{
    let mut senders = Arc::new(Mutex::new(Vec::new()));
    let mut receiver = Arc::new(Mutex::new(None));
    let mut connection: Option<Connection> = None;
    let mut bitrate = Bitrate::Bits(audio::DEFAULT_BITRATE);
    let mut cycle_error = false;

    let mut senders_cycle = senders.clone();
    let mut receiver_cycle = receiver.clone();

    // TEMP: FIX ME
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let out = timer.interval_at(Instant::now(), Duration::from_millis(20))
        .map_err(|why| {error!("[voice] Timer error for running connection. {:?}", why)})
        .for_each(|()| {
            let mut received_info = None;
            let mut should_disconnect = false;

            // Handle any control messages, drain them all synchronously.
            // All are obvious, except connection state changes, which
            // we want to collect to minimise spurious sub-frame channel changes.

            // TODO: probably fight w/ the borrow checker on how to keep this alive
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
                        let mut receiver = receiver.lock();
                        *receiver = r;
                    },
                    Ok(Status::SetSender(s)) => {
                        let senders = senders.lock();

                        senders.clear();

                        if let Some(aud) = s {
                            senders.push(aud);
                        }
                    },
                    Ok(Status::AddSender(s)) => {
                        let senders = senders.lock();

                        senders.push(s);
                    },
                    Ok(Status::SetBitrate(b)) => {
                        bitrate = b;
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
            // TODO.

            // Now we know what to do with the connection.
            // There are 3 cases:
            //  * There was a error on a conn last time, reconnect cheaply.
            //    If that fails, completely reconnect.
            //  * We already have a connection.
            //  * We want to make a new connection.

            let conn_future = match connection {
                Some(connection) => if cycle_error {
                        Either4::One(connection.reconnect(handle))
                    } else {
                        Either4::Two(ok(connection))
                    },
                None => if let Some(info) = received_info {
                    Either4::Three(Connection::new(info, handle))
                } else {
                    Either4::Four(err(Error::Voice(VoiceError::VoiceModeInvalid)))
                },
            };

            let out = conn_future
                .and_then(|conn| {
                    connection = Some(conn);
                    cycle_error = false;

                    conn.cycle(senders_cycle, receiver_cycle, bitrate)
                })
                .map_err(|why| {
                    error!(
                        "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                        why
                    )
                });

            // TODO: figure out what to do with this future.
            out.map(|_| ())
        });

    Box::new(out)
}
