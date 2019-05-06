use aktoro_channel::once;

#[test]
fn works() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), None);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    assert!(send.send(42));
    assert!(send.sent);
    assert!(!send.cancelled);
    assert!(!send.send(24));

    assert_eq!(recv.try_recv(), Some(true));
    assert_eq!(recv.data, Some(42));
    assert!(recv.received);
    assert!(!recv.cancelled);
    assert_eq!(recv.try_recv(), Some(false));
}

#[test]
fn cancel_send() {
    let (send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), None);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    drop(send);

    assert_eq!(recv.try_recv(), Some(false));
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(recv.cancelled);
    assert_eq!(recv.try_recv(), Some(false));
}

#[test]
fn cancel_recv() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), None);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    drop(recv);

    assert!(!send.send(42));
    assert!(!send.sent);
    assert!(send.cancelled);
    assert!(!send.send(24));
}

#[test]
fn close() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), None);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    assert!(recv.close());
    assert!(recv.closed);
    assert!(!recv.close());

    assert!(!send.send(42));
    assert!(!send.sent);
    assert!(send.cancelled);
    assert!(!send.send(24));

    assert_eq!(recv.try_recv(), Some(false));
    assert!(!recv.received);
    assert!(recv.closed);
    assert!(!recv.cancelled);
}
