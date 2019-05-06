use futures_channel::oneshot::Receiver;
use futures_channel::oneshot::Sender;

/// A wrapper around a [`oneshot::Sender`] that doesn't consume
/// itself when sending data and stores it state after doing so.
///
/// [`oneshot::Sender`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Sender.html
pub struct OnceSender<D> {
    pub sent: bool,
    pub cancelled: bool,
    sender: Option<Sender<D>>,
}

/// A wrapper arround a [`oneshot::Receiver`] that doesn't
/// implement [`Future`] but stores the received data along
/// with the channel's state.
///
/// [`oneshot::Receiver`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Receiver.html
/// [`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
pub struct OnceReceiver<D> {
    pub data: Option<D>,
    pub received: bool,
    pub cancelled: bool,
    receiver: Option<Receiver<D>>,
}

impl<D> OnceSender<D> {
    pub(crate) fn new(sender: Sender<D>) -> OnceSender<D> {
        OnceSender {
            sent: false,
            cancelled: false,
            sender: Some(sender),
        }
    }

    pub fn send(&mut self, data: D) -> bool {
        if let Some(sender) = self.sender.take() {
            match sender.send(data) {
                Ok(()) => self.sent = true,
                Err(_) => self.cancelled = true,
            }
        }

        self.sent || self.cancelled
    }
}

impl<D> OnceReceiver<D> {
    pub(crate) fn new(receiver: Receiver<D>) -> OnceReceiver<D> {
        OnceReceiver {
            data: None,
            received: false,
            cancelled: false,
            receiver: Some(receiver),
        }
    }

    pub fn try_recv(&mut self) -> bool {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_recv() {
                Ok(Some(data)) => {
                    self.data = Some(data);
                    self.received = true;
                }
                Ok(None) => (),
                Err(_) => {
                    self.cancelled = true;
                    self.receiver = None;
                }
            }
        }

        self.received || self.cancelled
    }
}
