use internal::Timer;
use model::id::GuildId;
use std::{
    sync::mpsc::{Receiver as MpscReceiver, TryRecvError},
    thread::Builder as ThreadBuilder
};
use super::{
    connection::Connection,
    Status
};

pub(crate) fn start(guild_id: GuildId, rx: MpscReceiver<Status>) {
    let name = format!("Serenity Voice (G{})", guild_id);

    ThreadBuilder::new()
        .name(name)
        .spawn(move || runner(&rx))
        .expect(&format!("[Voice] Error starting guild: {:?}", guild_id));
}

fn runner(rx: &MpscReceiver<Status>) {
    let mut senders = Vec::new();
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
                            warn!("[Voice] Error connecting: {:?}", why);

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
                    senders.clear();

                    if let Some(aud) = s {
                        senders.push(aud);
                    }
                },
                Ok(Status::AddSender(s)) => {
                    senders.push(s);
                },
                Err(TryRecvError::Empty) => {
                    // If we received nothing, then we can perform an update.
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
                let cycle = connection.cycle(&mut senders, &mut receiver, &mut timer);

                match cycle {
                    Ok(()) => false,
                    Err(why) => {
                        error!(
                            "(╯°□°）╯︵ ┻━┻ Error updating connection: {:?}",
                            why
                        );

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
