#[derive(PartialEq, Eq, Debug)]
pub enum SendError {
    Full,
    Disconnected,
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ReceiveError {
    Empty,
    Closed,
}

#[derive(PartialEq, Eq, Debug)]
pub enum CloseError {
    Disconnected,
    Closed,
}

impl SendError {
    pub fn is_full(&self) -> bool {
        *self == SendError::Full
    }

    pub fn is_disconnected(&self) -> bool {
        *self == SendError::Disconnected
    }

    pub fn is_closed(&self) -> bool {
        *self == SendError::Closed
    }
}

impl ReceiveError {
    pub fn is_empty(&self) -> bool {
        *self == ReceiveError::Empty
    }

    pub fn is_closed(&self) -> bool {
        *self == ReceiveError::Closed
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
