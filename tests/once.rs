use aktoro_channel::once;

#[test]
fn works() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.cancelled);

    assert!(send.send(42));
    assert!(send.sent);
    assert!(!send.cancelled);
    assert!(send.send(24));

    assert!(recv.try_recv());
    assert_eq!(recv.data, Some(42));
    assert!(recv.received);
    assert!(!recv.cancelled);
    assert!(recv.try_recv());
}

#[test]
fn cancel_send() {
    let (send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.cancelled);

    drop(send);

    assert!(!recv.try_recv());
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(recv.cancelled);
    assert!(!recv.try_recv());
}

#[test]
fn cancel_recv() {
    let (mut send, recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert!(recv.data.is_none());
    assert!(!recv.received);
    assert!(!recv.cancelled);

    drop(recv);

    assert!(!send.send(42));
    assert!(!send.sent);
    assert!(send.cancelled);
    assert!(!send.send(24));
}
