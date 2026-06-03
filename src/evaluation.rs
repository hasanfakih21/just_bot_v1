use std::cmp::max;

use crate::board::Board;
use crate::board::movegen::MoveGenKind;
use crate::types::{BitBoard, Piece, Side, Square};

//Tables from https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
#[rustfmt::skip]
pub const MG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
pub const EG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub const MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
pub const EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
pub const MG_BISHOP_TABLE: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
pub const EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
pub const MG_ROOK_TABLE: [i32; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
pub const EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
pub const MG_QUEEN_TABLE: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
pub const EG_QUEEN_TABLE: [i32; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
pub const MG_KING_TABLE: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
pub const EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

pub const MG_TABLE_ARRAY: [[i32; 64]; 6] = [
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE,
];
pub const EG_TABLE_ARRAY: [[i32; 64]; 6] = [
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE,
];

pub const MAX_MATERIAL_VALUE: i32 = 8000;

impl Board {
    pub const fn get_material_evaluation(&self) -> i32 {
        let side = self.board_state.side_to_move;
        self.board_state.material_value[side as usize]
            - self.board_state.material_value[side.other() as usize]
    }

    pub fn evaluate(&self) -> i32 {
        let mut mop_up_bonus = 0;
        //If only KQK or Lower then should mop up
        if self.mop_up_pos() {
            mop_up_bonus = self.mop_up();
        }

        self.get_material_evaluation() + self.get_piece_square_evaluation() + mop_up_bonus
    }

    pub const fn mop_up_pos(&self) -> bool {
        self.total_material_value() <= 900 && self.get_material_evaluation() > 0
    }

    pub const fn total_material_value(&self) -> i32 {
        self.board_state.material_value[Side::White as usize] + self.board_state.material_value[Side::Black as usize]
    }

    pub fn get_piece_square_score(&mut self, piece: Piece, square: Square, side: Side) -> i32 {
        let mg_table = MG_TABLE_ARRAY[piece as usize];
        let eg_table = EG_TABLE_ARRAY[piece as usize];

        let total_material_value = self.total_material_value().min(MAX_MATERIAL_VALUE);
        let mg_weight: f32 = total_material_value as f32 / MAX_MATERIAL_VALUE as f32;
        let eg_weight: f32 = 1.0 - mg_weight;

        let index = match side {
            Side::White => (square ^ Square::A8) as usize,
            Side::Black => square as usize,
        };

        ((mg_table[index] as f32 * mg_weight) + (eg_table[index] as f32 * eg_weight)) as i32
    }

    pub const fn get_piece_square_evaluation(&self) -> i32 {
        let side = self.board_state.side_to_move;
        self.board_state.piece_square_value[side as usize]
            - self.board_state.piece_square_value[side.other() as usize]
    }

    pub fn has_legal_move(&mut self) -> bool {
        let move_list = self.generate_moves(MoveGenKind::All);
        for m in move_list.iter() {
            if self.make_move(m.mv).is_ok() {
                self.unmake_move();
                return true;
            }
        }

        false
    }

    //Only checks for the current side to move
    pub fn only_king_and_pawns(&self) -> bool {
        let side = self.board_state.side_to_move;
        self.get_piece_bb(side, Piece::Bishop) | self.get_piece_bb(side, Piece::Knight) | self.get_piece_bb(side, Piece::Queen) | self.get_piece_bb(side, Piece::Rook) == BitBoard(0)
    }

    //Still needs more work and testing
    pub fn mop_up(&self) -> i32 {
        let current_side = self.board_state.side_to_move;
        let opp_king_square = self.get_king_square(current_side.other());
        let curr_king_square = self.get_king_square(current_side);

        let cmd = cmd(opp_king_square);
        let distance = distance(curr_king_square, opp_king_square);

        //Bonus for CMD of opposite king + (max distance - actual distance)
        ((15.0 * cmd as f32) + 5.0 * (7 - distance) as f32) as i32
    }

    pub const fn get_king_square(&self, side: Side) -> Square {
        self.get_piece_bb(side, Piece::King).least_sig_bit().unwrap()
    }
}

//https://www.chessprogramming.org/Center_Manhattan-Distance
pub const fn cmd(square: Square) -> usize {
    let (mut file, mut rank) = square.to_rank_and_file();

    file ^= (file.wrapping_sub(4)) >> 8;
    rank ^= (rank.wrapping_sub(4)) >> 8;

    file.wrapping_add(rank) & 7
}

pub const fn manhattan_distance(square_1: Square, square_2: Square) -> usize {
    let (rank1, file1) = square_1.to_rank_and_file();
    let (rank2, file2) = square_2.to_rank_and_file();

    let rank_distance = rank2.abs_diff(rank1);
    let file_distance = file2.abs_diff(file1);

    rank_distance + file_distance
}

//Chebyshev Distance
pub fn distance(square_1: Square, square_2: Square) -> usize {
    let (rank1, file1) = square_1.to_rank_and_file();
    let (rank2, file2) = square_2.to_rank_and_file();

    let rank_distance = rank2.abs_diff(rank1);
    let file_distance = file2.abs_diff(file1);

    max(rank_distance, file_distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::STARTING_FEN;

    #[test]
    fn test_get_material_evaluation() {
        let mut board = Board::from_fen(STARTING_FEN);
        assert_eq!(0, board.get_material_evaluation());

        board.remove_piece(Side::White, Piece::Pawn, Square::E2);
        assert_eq!(-100, board.get_material_evaluation());

        board.remove_piece(Side::Black, Piece::Rook, Square::H8);
        assert_eq!(400, board.get_material_evaluation());

        board.place_piece(Side::Black, Piece::Queen, Square::E5);
        assert_eq!(-500, board.get_material_evaluation());
    }

    #[test]
    fn test_only_king_and_pawn_check() {
        let board = Board::from_fen("8/8/8/8/p7/P7/1k6/3K4 b - - 16 65");
        assert!(board.only_king_and_pawns());
        let board = Board::from_fen("8/r4pK1/5Rp1/6k1/p6p/P6P/6P1/8 w - - 2 50");
        assert!(!board.only_king_and_pawns());
    }

    #[test]
    fn test_cmd() {
        let square = Square::A8;

        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::from_rank_and_file(rank, file);
                print!("{} ", cmd(square));
            }
            println!()
        }

        assert_eq!(6, cmd(square));
    }

    #[test]
    fn test_distances() {
        let square_1 = Square::A8;
        let square_2 = Square::A4;

        assert_eq!(4, distance(square_1, square_2));        

        let square_1 = Square::B4;
        let square_2 = Square::A4;

        assert_eq!(1, distance(square_1, square_2));        

        let square_1 = Square::H8;
        let square_2 = Square::A1;

        assert_eq!(7, distance(square_1, square_2));        
        assert_eq!(14, manhattan_distance(square_1, square_2));
    }

    #[test]
    fn test_mop_up() {
        let board = Board::from_fen("2K2R2/8/8/8/8/8/8/3k4 w - - 0 1"); 
        assert_eq!(board.total_material_value(), Piece::Rook.value());
        println!("{}", board.mop_up());
        println!("{}", board.evaluate());
        let first_bonus = board.mop_up();
        let first_score = board.evaluate();

        let board = Board::from_fen("5R2/8/8/8/8/2K5/8/3k4 w - - 0 1");
        println!("{}", board.mop_up());
        println!("{}", board.evaluate());
        let second_bonus = board.mop_up();
        let second_score = board.evaluate();

        assert!(second_bonus > first_bonus);
        assert!(second_score > first_score);
    }
}
