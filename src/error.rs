#[derive(PartialEq, Eq, Debug)]
pub enum SendError<D> {
    Full(D),
    Disconnected(D),
    Closed(D),
}

#[derive(PartialEq, Eq, Debug)]
pub enum ReceiveError {
    Empty,
    Disconnected,
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
pub enum DisconnectError {
    Disconnected,
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CloseError {
    Disconnected,
    Closed,
}

impl<D> SendError<D> {
    pub fn inner(&self) -> &D {
        match self {
            SendError::Full(data) => data,
            SendError::Disconnected(data) => data,
            SendError::Closed(data) => data,
        }
    }

    pub fn into_inner(self) -> D {
        match self {
            SendError::Full(data) => data,
            SendError::Disconnected(data) => data,
            SendError::Closed(data) => data,
        }
    }

    pub fn is_full(&self) -> bool {
        if let SendError::Full(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_disconnected(&self) -> bool {
        if let SendError::Disconnected(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_closed(&self) -> bool {
        if let SendError::Closed(_) = self {
            true
        } else {
            false
        }
    }
}

impl ReceiveError {
    pub fn is_empty(&self) -> bool {
        *self == ReceiveError::Empty
    }

    pub fn is_disconnected(&self) -> bool {
        *self == ReceiveError::Disconnected
    }

    pub fn is_closed(&self) -> bool {
        *self == ReceiveError::Closed
    }
}

impl DisconnectError {
    pub fn is_disconnected(&self) -> bool {
        *self == DisconnectError::Disconnected
    }

    pub fn is_closed(&self) -> bool {
        *self == DisconnectError::Closed
    }
}

impl CloseError {
    pub fn is_disconnected(&self) -> bool {
        *self == CloseError::Disconnected
    }

    pub fn is_closed(&self) -> bool {
        *self == CloseError::Closed
    }
}
