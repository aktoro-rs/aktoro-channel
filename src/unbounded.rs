use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures_channel::mpsc::UnboundedReceiver as Receiver;
use futures_channel::mpsc::UnboundedSender as Sender;
use futures_core::stream::FusedStream;
use futures_core::stream::Stream;
use futures_sink::Sink;

use crate::error::*;

#[derive(Debug)]
/// A wrapper around a [`mpsc::UnboundedSender`] that stores
/// its state after sending data, closing the channel or
/// disconnecting itself.
///
/// [`mpsc::UnboundedSender`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures/channel/mpsc/struct.UnboundedSender.html
pub struct UnboundedSender<D> {
    /// Whether the channel has been closed.
    pub closed: bool,
    /// Whether the sender has diconnected itself from the
    /// channel.
    pub disconnected: bool,
    sender: Sender<D>,
}

#[derive(Debug)]
/// A wrapper arround a [`mpsc::UnboundedReceiver`] that
/// stores it state after trying to receive data or closing
/// the channel.
///
/// [`mpsc::UnboundedReceiver`]: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.15/futures/channel/mpsc/struct.UnboundedReceiver.html
pub struct UnboundedReceiver<D> {
    /// Whether the channel has been closed.
    pub closed: bool,
    receiver: Option<Receiver<D>>,
}

impl<D> UnboundedSender<D> {
    pub(crate) fn new(sender: Sender<D>) -> UnboundedSender<D> {
        UnboundedSender {
            closed: false,
            disconnected: false,
            sender,
        }
    }

    /// Sends `data` over the channel, returning `Ok(())` if
    /// it has been successfully sent, or either
    /// `Err(SendError::Disconnected)` if the sender has
    /// disconnected itself from the channel or
    /// `Err(SendError::Closed)` if the channel has been closed.
    pub fn send(&mut self, data: D) -> Result<(), SendError<D>> {
        if self.disconnected {
            return Err(SendError::Disconnected(data));
        } else if self.closed {
            return Err(SendError::Closed(data));
        } else if self.sender.is_closed() {
            self.closed = true;
            return Err(SendError::Closed(data));
        }

        match self.sender.unbounded_send(data) {
            Ok(()) => Ok(()),
            Err(err) => {
                if err.is_disconnected() {
                    self.closed = true;
                    Err(SendError::Closed(err.into_inner()))
                } else {
                    unreachable!();
                }
            }
        }
    }

    /// Tries to disconnect the sender from the channel,
    /// returning `Ok(())` if it succeeded, or either
    /// `Err(DiconnectError::Disconnected)` if the sender
    /// already disconnected itself, or
    /// `Err(DisconnectError::Closed)` if the channel was
    /// already closed.
    pub fn disconnect(&mut self) -> Result<(), DisconnectError> {
        if self.disconnected {
            Err(DisconnectError::Disconnected)
        } else if self.closed {
            Err(DisconnectError::Closed)
        } else if self.sender.is_closed() {
            self.closed = true;
            Err(DisconnectError::Closed)
        } else {
            self.sender.disconnect();
            self.disconnected = true;
            Ok(())
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or either `Err(CloseError::Disconnected)`
    /// if the sender already disconnected itself, or
    /// `Err(CloseError::Closed)` if the channel was already
    /// closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        if self.disconnected {
            Err(CloseError::Disconnected)
        } else if self.closed {
            Err(CloseError::Closed)
        } else if self.sender.is_closed() {
            self.closed = true;
            Err(CloseError::Closed)
        } else {
            self.sender.close_channel();
            self.closed = true;
            Ok(())
        }
    }
}

impl<D> UnboundedReceiver<D> {
    pub(crate) fn new(receiver: Receiver<D>) -> UnboundedReceiver<D> {
        UnboundedReceiver {
            closed: false,
            receiver: Some(receiver),
        }
    }

    /// Tries to receive a message over the channel, returning
    /// `Ok(D)` if it has received one, and either
    /// `Err(ReceiveError::Empty)` if it hasn't or
    /// `Err(ReceiveError::Closed)` if the channel has been
    /// closed.
    pub fn try_recv(&mut self) -> Result<D, ReceiveError> {
        if let Some(ref mut receiver) = self.receiver {
            match receiver.try_next() {
                Ok(Some(data)) => Ok(data),
                Ok(None) => {
                    self.receiver = None;
                    self.closed = true;
                    Err(ReceiveError::Closed)
                }
                Err(_) => Err(ReceiveError::Empty),
            }
        } else {
            Err(ReceiveError::Closed)
        }
    }

    /// Tries to close the channel, returning `Ok(())` if it
    /// succeeded, or `Err(CloseError::Closed)` if the channel
    /// was already closed.
    pub fn close(&mut self) -> Result<(), CloseError> {
        if self.closed {
            Err(CloseError::Closed)
        } else if let Some(ref mut receiver) = self.receiver {
            receiver.close();
            self.closed = true;
            Ok(())
        } else {
            unreachable!();
        }
    }
}

impl<D> Unpin for UnboundedReceiver<D> {}

impl<D> Sink<D> for UnboundedSender<D> {
    // FIXME: -`()` +`D` (the issue being that `poll_ready`,
    //   `poll_flush` and `poll_close` can't return `D` since
    //   they don't get any data in the first place).
    type SinkError = SendError<()>;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_ready(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn start_send(self: Pin<&mut Self>, msg: D) -> Result<(), SendError<()>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).start_send(msg).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_flush(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), SendError<()>>> {
        let receiver = self.get_mut();
        Pin::new(&mut receiver.sender).poll_close(cx).map_err(|_| {
            if receiver.disconnected {
                SendError::Disconnected(())
            } else if receiver.closed {
                SendError::Closed(())
            } else {
                receiver.closed = true;
                SendError::Closed(())
            }
        })
    }
}

impl<D> FusedStream for UnboundedReceiver<D> {
    fn is_terminated(&self) -> bool {
        self.closed
    }
}

impl<D> Stream for UnboundedReceiver<D> {
    type Item = D;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<D>> {
        if let Some(ref mut receiver) = self.receiver {
            match Pin::new(receiver).poll_next(cx) {
                Poll::Ready(None) => {
                    self.receiver = None;
                    self.closed = true;
                    Poll::Ready(None)
                }
                poll => poll,
            }
        } else if self.closed {
            Poll::Ready(None)
        } else {
            unreachable!();
        }
    }
}

impl<D> Clone for UnboundedSender<D> {
    fn clone(&self) -> UnboundedSender<D> {
        UnboundedSender {
            sender: self.sender.clone(),
            ..*self
        }
    }
}
