use futures_channel::mpsc;
use futures_channel::oneshot;

mod error;
pub mod once;
pub mod unbounded;

pub use error::*;
pub use once::*;
pub use unbounded::*;

pub fn once<D>() -> (OnceSender<D>, OnceReceiver<D>) {
    let (sender, receiver) = oneshot::channel();

    (OnceSender::new(sender), OnceReceiver::new(receiver))
}

pub fn unbounded<D>() -> (UnboundedSender<D>, UnboundedReceiver<D>) {
    let (sender, receiver) = mpsc::unbounded();

    (
        UnboundedSender::new(sender),
        UnboundedReceiver::new(receiver),
    )
}
