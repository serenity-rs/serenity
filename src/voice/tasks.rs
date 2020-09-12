use crate::internal::Timer;
use crate::model::id::GuildId;
use futures::channel::mpsc::UnboundedReceiver as Receiver;
use super::{
    connection::Connection,
    Status,
    audio,
};
use tracing::{info, error, warn, instrument};

#[instrument(skip(rx))]
pub(crate) fn start(guild_id: GuildId, mut rx: Receiver<Status>) {
    tokio::spawn(async move {
        info!("[Voice] Starts running for guild id: {}", guild_id);
        runner(&mut rx).await;
        info!("[Voice] Ended running for guild id: {}", guild_id);
    });
}

#[instrument(skip(rx))]
async fn runner(rx: &mut Receiver<Status>) {
    let mut senders = Vec::new();
    let mut receiver = None;
    let mut connection = None;
    let mut timer = Timer::new(20);
    let mut bitrate = audio::DEFAULT_BITRATE;
    let mut mute = false;

    'runner: loop {
        loop {
            match rx.try_next() {
                Ok(Some(Status::Connect(info))) => {
                    connection = match Connection::new(info).await {
                        Ok(connection) => Some(connection),
                        Err(why) => {
                            warn!("[Voice] Error connecting: {:?}", why);

                            None
                        },
                    };
                },
                Ok(Some(Status::Disconnect)) => {
                    connection = None;
                },
                Ok(Some(Status::SetReceiver(r))) => {
                    receiver = r;
                },
                Ok(Some(Status::SetSender(s))) => {
                    senders.clear();

                    if let Some(aud) = s {
                        senders.push(aud);
                    }
                },
                Ok(Some(Status::AddSender(s))) => {
                    senders.push(s);
                },
                Ok(Some(Status::SetBitrate(b))) => {
                    bitrate = b;
                },
                Ok(Some(Status::Mute(m))) => {
                    mute = m;
                },
                Ok(None) => {
                    // Other channel closed.
                    rx.close();
                    break 'runner;
                },
                Err(_) => {
                    // If we received nothing, then we can perform an update.
                    break;
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
                let cycle = connection
                    .cycle(&mut senders, &mut receiver, &mut timer, bitrate, mute).await;

                match cycle {
                    Ok(()) => false,
                    Err(why) => {
                        error!(
                            "[Voice] Error updating connection: {:?}",
                            why
                        );

                        true
                    },
                }
            },
            None => {
                timer.hold().await;

                false
            },
        };

        // If there was an error, then just reset the connection and try to get
        // another.
        if error {
            let mut conn = connection.expect("[Voice] Shouldn't have had a voice connection error without a connection.");
            connection = conn.reconnect()
                .await
                .ok()
                .map(|_| conn);
        }
    }
}
