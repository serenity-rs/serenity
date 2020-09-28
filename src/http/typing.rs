use crate::{error::Result, http::Http};
use std::sync::Arc;
use tokio::{sync::oneshot::{self, Sender, error::TryRecvError}, time::{delay_for, Duration}};

/// A struct to start typing in a [`Channel`] for an indefinite period of time.
///
/// It indicates that the current user is currently typing in the channel.
///
/// Typing is started by using the [`Typing::start`] method
/// and stopped by using the [`Typing::stop`] method.
/// Note that on some clients, typing may persist for a few seconds after `stop` is called.
/// Typing is also stopped when the struct is dropped.
///
/// If a message is sent while typing is triggered, the user will stop typing for a brief period
/// of time and then resume again until either `stop` is called or the struct is dropped.
///
/// This should rarely be used for bots, although it is a good indicator that a
/// long-running command is still being processed.
///
/// ## Examples
///
/// ```rust,no_run
/// # use serenity::{http::{Http, Typing}, Result};
/// # use std::sync::Arc;
/// #
/// # fn long_process() {}
/// # fn main() -> Result<()> {
/// # let http = Http::default();
/// // Initiate typing (assuming `http` is bound)
/// let typing = Typing::start(Arc::new(http), 7)?;
///
/// // Run some long-running process
/// long_process();
///
/// // Stop typing
/// typing.stop();
/// #
/// # Ok(())
/// # }
/// ```
///
/// [`Channel`]: ../../model/channel/enum.Channel.html
/// [`Typing::start`]: struct.Typing.html#method.start
/// [`Typing::stop`]: struct.Typing.html#method.stop
#[derive(Debug)]
pub struct Typing(Sender<()>);

impl Typing {
    /// Starts typing in the specified [`Channel`] for an indefinite period of time.
    ///
    /// Returns [`Typing`]. To stop typing, you must call the [`Typing::stop`] method on
    /// the returned `Typing` object or wait for it to be dropped. Note that on some
    /// clients, typing may persist for a few seconds after stopped.
    ///
    /// [`Channel`]: ../../model/channel/enum.Channel.html
    /// [`Typing`]: struct.Typing.html
    /// [`Typing::stop`]: struct.Typing.html#method.stop
    pub fn start(http: Arc<Http>, channel_id: u64) -> Result<Self> {
        let (sx, mut rx) = oneshot::channel();

        tokio::spawn(async move {
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Closed) => break,
                    _ => (),
                }

                http.broadcast_typing(channel_id).await?;

                // It is unclear for how long typing persists after this method is called.
                // It is generally assumed to be 7 or 10 seconds, so we use 7 to be safe.
                delay_for(Duration::from_secs(7)).await;
            }

            Result::Ok(())
        });

        Ok(Self(sx))
    }

    /// Stops typing in [`Channel`].
    ///
    /// This should be used to stop typing after it is started using [`Typing::start`].
    /// Typing may persist for a few seconds on some clients after this is called.
    ///
    /// [`Channel`]: ../../model/channel/enum.Channel.html
    /// [`Typing::start`]: struct.Typing.html#method.start
    pub fn stop(self) -> Option<()> {
        self.0.send(()).ok()
    }
}
