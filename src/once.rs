use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_channel::oneshot::Receiver;
use futures_channel::oneshot::Sender;
use futures_core::future::Future;

use crate::error::*;

#[derive(Debug)]
/// A wrapper around a [`oneshot::Sender`] that doesn't consume
/// itself when sending data and stores it state after doing so.
///
/// [`oneshot::Sender`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Sender.html
pub struct OnceSender<D> {
    /// Whether data has already been sent over the channel.
    pub sent: bool,
    /// Whether the channel has been cancelled.
    pub cancelled: bool,
    sender: Option<Sender<D>>,
}

#[derive(Debug)]
/// A wrapper arround a [`oneshot::Receiver`] that stores
/// the received data along with the channel's state.
///
/// [`oneshot::Receiver`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures_channel/oneshot/struct.Receiver.html
pub struct OnceReceiver<D> {
    /// Whether data has been received (same as
    /// `data.is_some()`).
    pub received: bool,
    /// Whether the channel has been closed.
    pub closed: bool,
    /// Whether the channel has been cancelled.
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

    /// Sends `data` over the channel, returning `Ok(())` if it
    /// has been successfully sent or either
    /// `Err(SendError::Closed)` if the channel has been cancelled
    /// or `Err(SendError::Full)` if a message has already been
    /// sent over it.
    pub fn send(&mut self, data: D) -> Result<(), SendError<D>> {
        if let Some(sender) = self.sender.take() {
            match sender.send(data) {
                Ok(()) => {
                    self.sent = true;
                    return Ok(());
                }
                Err(data) => {
                    self.cancelled = true;
                    return Err(SendError::Closed(data));
                }
            }
        } else if self.sent {
            Err(SendError::Full(data))
        } else if self.cancelled {
            Err(SendError::Closed(data))
        } else {
            unreachable!();
        }
    }
}

impl<D> OnceReceiver<D> {
    pub(crate) fn new(receiver: Receiver<D>) -> OnceReceiver<D> {
        OnceReceiver {
            received: false,
            closed: false,
            cancelled: false,
            receiver: Some(receiver),
        }
    }

    /// Tries to receive a message over the channel, returning
    /// `Ok(D)` if it has received one and either
    /// `Err(ReceiveError::Empty)` if it hasn't or
    /// `Err(ReceiveError::Closed)` if the channel has been
    /// cancelled or closed.
    pub fn try_recv(&mut self) -> Result<D, ReceiveError> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_recv() {
                Ok(Some(data)) => {
                    self.received = true;
                    return Ok(data);
                }
                Ok(None) => return Err(ReceiveError::Empty),
                Err(_) => {
                    self.cancelled = true;
                    self.receiver = None;
                    return Err(ReceiveError::Closed);
                }
            }
        } else {
            return Err(ReceiveError::Closed);
        }
    }

    /// Tries to close the channel, returning `Ok(())` if
    /// it succeeded or `Err(CloseError::Closed)` it the
    /// chanenl has already been closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        if let Some(ref mut receiver) = self.receiver {
            receiver.close();
            self.receiver = None;
            self.closed = true;
            Ok(())
        } else {
            Err(CloseError::Closed)
        }
    }
}

impl<D> Unpin for OnceSender<D> {}
impl<D> Unpin for OnceReceiver<D> {}

impl<D> Future for OnceReceiver<D> {
    type Output = Result<D, ReceiveError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<D, ReceiveError>> {
        let receiver = self.get_mut();
        if let Some(ref mut recv) = receiver.receiver {
            match Pin::new(recv).poll(cx) {
                Poll::Ready(Ok(data)) => {
                    receiver.received = true;
                    Poll::Ready(Ok(data))
                }
                Poll::Ready(Err(_)) => {
                    receiver.cancelled = true;
                    Poll::Ready(Err(ReceiveError::Closed))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(Err(ReceiveError::Closed))
        }
    }
}
