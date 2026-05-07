#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn other(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White
        }
    }
}