use aktoro_channel::*;

#[test]
fn works() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    assert_eq!(send.send(42), Ok(()));
    assert!(send.sent);
    assert!(!send.cancelled);
    assert_eq!(send.send(24), Err(SendError::Full));

    assert_eq!(recv.try_recv(), Ok(42));
    assert!(recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
}

#[test]
fn cancel_send() {
    let (send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    drop(send);

    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(recv.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
}

#[test]
fn cancel_recv() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    drop(recv);

    assert_eq!(send.send(42), Err(SendError::Closed));
    assert!(!send.sent);
    assert!(send.cancelled);
    assert_eq!(send.send(24), Err(SendError::Closed));
}

#[test]
fn close() {
    let (mut send, mut recv) = once::<u8>();

    assert!(!send.sent);
    assert!(!send.cancelled);
    assert_eq!(recv.try_recv(), Err(ReceiveError::Empty));
    assert!(!recv.received);
    assert!(!recv.closed);
    assert!(!recv.cancelled);

    assert_eq!(recv.close(), Ok(()));
    assert!(recv.closed);
    assert_eq!(recv.close(), Err(CloseError::Closed));

    assert_eq!(send.send(42), Err(SendError::Closed));
    assert!(!send.sent);
    assert!(send.cancelled);
    assert_eq!(send.send(24), Err(SendError::Closed));

    assert_eq!(recv.try_recv(), Err(ReceiveError::Closed));
    assert!(!recv.received);
    assert!(recv.closed);
    assert!(!recv.cancelled);
}
