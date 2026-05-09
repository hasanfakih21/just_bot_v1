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

#[repr(u8)]
pub enum Castling {
    WhiteKing  = 0b0001,
    WhiteQueen = 0b0010,
    BlackKing  = 0b0100,
    BlackQueen = 0b1000,
}

pub struct CastlingRights(u8);

impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights(0b1111)
    }

    pub const fn can_king_side(&self, side: Side) -> bool {
        match side {
            Side::White => (Castling::WhiteKing as u8 & self.0) > 0,
            Side::Black => (Castling::BlackKing as u8 & self.0) > 0
        }
    }

    pub const fn can_queen_side(&self, side: Side) -> bool {
        match side {
            Side::White => (Castling::WhiteQueen as u8 & self.0) > 0,
            Side::Black => (Castling::BlackQueen as u8 & self.0) > 0
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::new()
    }
}