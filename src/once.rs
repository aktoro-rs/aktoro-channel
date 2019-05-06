use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_channel::oneshot::Receiver;
use futures_channel::oneshot::Sender;
use futures_core::future::Future;

/// A wrapper around a [`oneshot::Sender`] that doesn't consume
/// itself when sending data and stores it state after doing so.
///
/// [`oneshot::Sender`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Sender.html
#[derive(Debug)]
pub struct OnceSender<D> {
    pub sent: bool,
    pub cancelled: bool,
    sender: Option<Sender<D>>,
}

/// A wrapper arround a [`oneshot::Receiver`] that stores
/// the received data along with the channel's state.
///
/// [`oneshot::Receiver`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Receiver.html
#[derive(Debug)]
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

    /// Sends `data` over the channel, returning `true` if it
    /// has been successfully sent or `false` if the channel
    /// has been cancelled or a message already sent over it.
    pub fn send(&mut self, data: D) -> bool {
        if let Some(sender) = self.sender.take() {
            match sender.send(data) {
                Ok(()) => self.sent = true,
                Err(_) => self.cancelled = true,
            }

            self.sent
        } else {
            false
        }
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

    /// Tries to receive data over the channel, returning
    /// `Some(true)` if it has received some, `Some(false)`
    /// if the channel has been cancelled or a message
    /// already received and `None` otherwise.
    pub fn try_recv(&mut self) -> Option<bool> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_recv() {
                Ok(Some(data)) => {
                    self.data = Some(data);
                    self.received = true;
                    return Some(true);
                }
                Ok(None) => return None,
                Err(_) => {
                    self.cancelled = true;
                    self.receiver = None;
                    return Some(false);
                }
            }
        } else {
            Some(false)
        }
    }
}

impl<D> Unpin for OnceSender<D> {}
impl<D> Unpin for OnceReceiver<D> {}

impl<D> Future for OnceReceiver<D> {
    type Output = Result<D, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<D, ()>> {
        if let Some(ref mut receiver) = self.get_mut().receiver {
            Pin::new(receiver).poll(cx).map_err(|_| ())
        } else {
            Poll::Ready(Err(()))
        }
    }
}
