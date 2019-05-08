use futures_channel::oneshot;

mod error;
pub mod once;

pub use error::*;
pub use once::*;

pub fn once<D>() -> (OnceSender<D>, OnceReceiver<D>) {
    let (sender, receiver) = oneshot::channel();

    (OnceSender::new(sender), OnceReceiver::new(receiver))
}
