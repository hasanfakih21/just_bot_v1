use std::{fmt::Display, slice::{Iter, IterMut}};
use std::vec::IntoIter;

use crate::types::{Piece, Square};

#[derive(Default, Debug, Clone)]
pub struct MoveList(pub Vec <Move>);

impl MoveList {
    pub fn new() -> Self {
        MoveList(Vec::new())
    }

    pub fn add(&mut self, m: Move) {
        self.0.push(m);
    }

    pub fn iter(&self) -> Iter<'_, Move> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Move> {
        self.0.iter_mut()
    }

    pub fn into_iter(&self) -> IntoIter<Move> {
        self.0.clone().into_iter()
    }
}

impl FromIterator<Move> for MoveList {
    fn from_iter<T: IntoIterator<Item = Move>>(iter: T) -> Self {
        MoveList(iter.into_iter().collect())
    } 
}

//12 bits for to and from square and 4 bits for move type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move(u16);

impl Move {
    pub fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        Move(from as u16 | ((to as u16) << 6) | ((kind as u16) << 12))
    }

    pub fn get_from(&self) -> Square {
        Square::from((0x003F & self.0) as usize)
    }

    pub fn get_to(&self) -> Square {
        Square::from(((0x0FC0 & self.0) >> 6) as usize)
    }

    pub fn get_kind(&self) -> MoveKind {
        MoveKind::from(((0xF000 & self.0) >> 12) as u8)
    }

    pub fn get_promoted_piece(&self) -> Option<Piece> {
        let kind = self.get_kind();
        let mut promoted_piece = None;

        if kind.is_knight_promotion() {promoted_piece = Some(Piece::Knight)}
        else if kind.is_bishop_promotion() {promoted_piece = Some(Piece::Bishop)}
        else if kind.is_rook_promotion() {promoted_piece = Some(Piece::Rook)}
        else if kind.is_queen_promotion() {promoted_piece = Some(Piece::Queen)}
        promoted_piece
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut promotion_piece = "";
        match self.get_kind() {
            MoveKind::BPromotion | MoveKind::BPromCapture => promotion_piece = "b",    
            MoveKind::NPromotion | MoveKind::NPromCapture => promotion_piece = "n",
            MoveKind::RPromotion | MoveKind::RPromCapture => promotion_piece = "r",
            MoveKind::QPromotion | MoveKind::QPromCapture => promotion_piece = "q",    
            _ => ()
        }
        write!(f, "{}{}{}", self.get_from(), self.get_to(), promotion_piece)
    }
}

#[derive(Debug)]
pub enum MoveKind {
    QuietMove    = 0b0000,
    DoublePawn   = 0b0001,
    KingCastle   = 0b0010,
    QueenCastle  = 0b0011,
    Capture      = 0b0100,
    EnPassant    = 0b0101,
    NPromotion   = 0b1000,
    BPromotion   = 0b1001,
    RPromotion   = 0b1010,
    QPromotion   = 0b1011,
    NPromCapture = 0b1100,
    BPromCapture = 0b1101,
    RPromCapture = 0b1110,
    QPromCapture = 0b1111,
}

impl MoveKind {
    fn from(value: u8) -> Self {
        use MoveKind::*;
        match value {
            0b0000 => QuietMove, 
            0b0001 => DoublePawn, 
            0b0010 => KingCastle, 
            0b0011 => QueenCastle, 
            0b0100 => Capture, 
            0b0101 => EnPassant, 
            0b1000 => NPromotion, 
            0b1001 => BPromotion, 
            0b1010 => RPromotion, 
            0b1011 => QPromotion, 
            0b1100 => NPromCapture, 
            0b1101 => BPromCapture, 
            0b1110 => RPromCapture, 
            0b1111 => QPromCapture , 
            _ => panic!("Not a valid move kind!!")
        }
    }

    pub const fn is_quiet(&self) -> bool {
        use MoveKind::*;
        matches!(self, QuietMove | DoublePawn | KingCastle | QueenCastle)
    }

    pub const fn is_promotion(&self) -> bool {
        use MoveKind::*;
        matches!(self, NPromotion | BPromotion | RPromotion | QPromotion | NPromCapture | BPromCapture | RPromCapture | QPromCapture)
    }

    pub const fn is_knight_promotion(&self) -> bool {
        use MoveKind::*;
        matches!(self, NPromCapture | NPromotion)
    }

    pub const fn is_bishop_promotion(&self) -> bool {
        use MoveKind::*;
        matches!(self, BPromCapture | BPromotion)
    }

    pub const fn is_rook_promotion(&self) -> bool {
        use MoveKind::*;
        matches!(self, RPromCapture | RPromotion)
    }

    pub const fn is_queen_promotion(&self) -> bool {
        use MoveKind::*;
        matches!(self, QPromCapture | QPromotion)
    }

    pub const fn is_capture(&self) -> bool {
        use MoveKind::*;
        matches!(self, Capture | NPromCapture | BPromCapture | RPromCapture | QPromCapture | EnPassant)
    }
}

