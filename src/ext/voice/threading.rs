use std::sync::mpsc::{Receiver as MpscReceiver, TryRecvError};
use std::thread::Builder as ThreadBuilder;
use super::connection::Connection;
use super::{Status, Target};
use ::internal::Timer;

pub fn start(target_id: Target, rx: MpscReceiver<Status>) {
    let name = match target_id {
        Target::Channel(channel_id) => format!("Serenity Voice (C{})", channel_id),
        Target::Guild(guild_id) => format!("Serenity Voice (G{})", guild_id),
    };

    ThreadBuilder::new()
        .name(name)
        .spawn(move || runner(rx))
        .expect("Error starting voice");
}

fn runner(rx: MpscReceiver<Status>) {
    let mut sender = None;
    let mut receiver = None;
    let mut connection = None;
    let mut timer = Timer::new(20);

    'runner: loop {
        loop {
            match rx.try_recv() {
                Ok(Status::Connect(info)) => {
                    connection = match Connection::new(info) {
                        Ok(connection) => Some(connection),
                        Err(why) => {
                            error!("Error connecting via voice: {:?}", why);

                            None
                        },
                    };
                },
                Ok(Status::Disconnect) => {
                    connection = None;
                },
                Ok(Status::SetReceiver(r)) => {
                    receiver = r;
                },
                Ok(Status::SetSender(s)) => {
                    sender = s;
                },
                Err(TryRecvError::Empty) => {
                    // If we receieved nothing, then we can perform an update.
                    break;
                },
                Err(TryRecvError::Disconnected) => {
                    break 'runner;
                },
            }
        }

        // Overall here, check if there's an error.
        //
        // If there is a connection, try to send an update. This should not
        // error. If there is though for some spurious reason, then set `error`
        // to `true`.
        //
        // Otherwise, wait out the timer and do _not_ error and wait to receive
        // another event.
        let error = match connection.as_mut() {
            Some(connection) => {
                let update = connection.update(&mut sender,
                                               &mut receiver,
                                               &mut timer);

                match update {
                    Ok(()) => false,
                    Err(why) => {
                        error!("Error updating voice connection: {:?}", why);

                        true
                    },
                }
            },
            None => {
                timer.await();

                false
            },
        };

        // If there was an error, then just reset the connection and try to get
        // another.
        if error {
            connection = None;
        }
    }
}
