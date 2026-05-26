use crate::types::Square;

pub const A_FILE: u64 = 0x0101010101010101;
pub const B_FILE: u64 = 0x0202020202020202;
pub const G_FILE: u64 = 0x4040404040404040;
pub const H_FILE: u64 = 0x8080808080808080;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_8: u64 = 0xFF00000000000000;

pub const FULL: u64 = 0xFFFFFFFFFFFFFFFF;
pub const WK_SIDE: u64 = 0x0000000000000060;
pub const WQ_SIDE: u64 = 0x000000000000000E;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const NORTH: i8 = 8;
pub const SOUTH: i8 = -8;
pub const WEST: i8 = -1;
pub const EAST: i8 = 1;

pub const KING_SIDE_ROOK_WHITE: Square = Square::H1;
pub const QUEEN_SIDE_ROOK_WHITE: Square = Square::A1;

pub const KING_SIDE_ROOK_BLACK: Square = Square::H8;
pub const QUEEN_SIDE_ROOK_BLACK: Square = Square::A8;

pub const INFINITY: i32 = 100000; 
pub const MATE_SCORE: i32 = 9000;
pub const MATE_CUTOFF: i32 = 8900;