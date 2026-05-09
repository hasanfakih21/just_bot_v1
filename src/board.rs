pub mod squares;
pub mod pieces;
pub mod sides;
pub mod constants;

pub use squares::*;
pub use pieces::*;
pub use sides::*;

pub use crate::attacks::*;
use crate::{magics::{BISHOP_MAGIC_NUMBERS, ROOK_MAGIC_NUMBERS, get_magic_index}, occupancy::{BISHOP_OCCUPANCY_BIT_COUNTS, ROOK_OCCUPANCY_BIT_COUNTS, set_occupancy}};

pub struct Board {
    pub board_pieces: [[u64; 6]; 2],
    pub pawn_attacks: [[u64; 64]; 2],
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],
    pub bishop_masks: [u64; 64],
    pub rook_masks: [u64; 64],
    pub bishop_attacks: Vec<u64>,
    pub rook_attacks: Vec<u64>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

//Little-Endian Rank-File Mapping
impl Board {
    pub fn new() -> Self {
        let mut b = Board {
            board_pieces: [
                [0x0000_0000_0000_FF00,
                 0x0000_0000_0000_0042,
                 0x0000_0000_0000_0024,
                 0x0000_0000_0000_0081,
                 0x0000_0000_0000_0008,
                 0x0000_0000_0000_0010],

                [0x00FF_0000_0000_0000,
                 0x4200_0000_0000_0000,
                 0x2400_0000_0000_0000,
                 0x8100_0000_0000_0000,
                 0x0800_0000_0000_0000,
                 0x1000_0000_0000_0000]
            ],
            pawn_attacks: [[0; 64]; 2],
            knight_attacks: [0; 64],
            king_attacks: [0; 64],
            bishop_masks: std::array::from_fn(|i| {
                mask_bishop_attacks(Square::from(i))
            }),
            rook_masks: std::array::from_fn(|i| {
                mask_rook_attacks(Square::from(i))
            }), 
            bishop_attacks: vec![0; 64 * 512],
            rook_attacks: vec![0; 64 * 4096],
        };

        b.init_leaping_attacks();
        b.init_bishop_attacks();
        b.init_rook_attacks();
        b
    }

    pub fn set_piece_bit(&mut self, side: Side, piece: Piece, position: Square) {
        set_bit(&mut self.board_pieces[side as usize][piece as usize], position);
    }

    pub fn clear_piece_bit(&mut self, side: Side, piece: Piece, position: Square) {
        clear_bit(&mut self.board_pieces[side as usize][piece as usize], position);
    }

    pub fn get_bit(&self, side: Side, piece: Piece, position: Square) -> bool {
        let b = 1u64 << position as u64;
        (self.board_pieces[side as usize][piece as usize] & b) != 0
    }

    pub fn get_piece_at_square(&self, side: Side, position: Square) -> Option<Piece> {
        for piece_index in 0..6 {
            if self.get_bit(side, Piece::try_from(piece_index).unwrap(), position) {
                return Some(Piece::try_from(piece_index).unwrap());
            }
        }
        None
    }

    pub fn init_leaping_attacks(&mut self) {
        for i in 0..64 {
            let square = Square::from(i);
            self.pawn_attacks[Side::White as usize][i] = mask_pawn_attacks(Side::White, square);
            self.pawn_attacks[Side::Black as usize][i] = mask_pawn_attacks(Side::Black, square);
            self.knight_attacks[i] = mask_knight_attacks(square);
            self.king_attacks[i] = mask_king_attacks(square);
        }
    }

    pub fn init_rook_attacks(&mut self) {
        for i in 0..64 {
            let square = Square::from(i);
            let relevant_bits = ROOK_OCCUPANCY_BIT_COUNTS[square as usize]; 
            let magic_number = ROOK_MAGIC_NUMBERS[square as usize];

            for index in 0..4096 {
                let occupancy_bb = set_occupancy(index, relevant_bits, self.rook_masks[square as usize]);
                let magic_index = get_magic_index(occupancy_bb, relevant_bits, magic_number);
                self.rook_attacks[(square as usize * 4096) + magic_index] = blocked_rook_attacks(square, occupancy_bb);
            }
        }
    }

    pub fn init_bishop_attacks(&mut self) {
        for i in 0..64 {
            let square = Square::from(i);
            let relevant_bits = BISHOP_OCCUPANCY_BIT_COUNTS[square as usize]; 
            let magic_number = BISHOP_MAGIC_NUMBERS[square as usize];

            for index in 0..512 {
                let occupancy_bb = set_occupancy(index, relevant_bits, self.bishop_masks[square as usize]);
                let magic_index = get_magic_index(occupancy_bb, relevant_bits, magic_number);
                self.bishop_attacks[(square as usize * 512) + magic_index] = blocked_bishop_attacks(square, occupancy_bb);
            }
        }
    }

    pub fn get_bishop_attacks(&self, square: Square, board_occupancy: u64) -> u64 {
        let occupancy = board_occupancy & self.bishop_masks[square as usize];
        let magic_index = get_magic_index(occupancy, BISHOP_OCCUPANCY_BIT_COUNTS[square as usize], BISHOP_MAGIC_NUMBERS[square as usize]);
        let offset = (square as usize * 512) + magic_index;

        self.bishop_attacks[offset]
    }

    pub fn get_rook_attacks(&self, square: Square, board_occupancy: u64) -> u64 {
        let occupancy = board_occupancy & self.rook_masks[square as usize];
        let magic_index = get_magic_index(occupancy, ROOK_OCCUPANCY_BIT_COUNTS[square as usize], ROOK_MAGIC_NUMBERS[square as usize]);
        let offset = (square as usize * 4096) + magic_index;

        self.rook_attacks[offset]
    }
}

pub fn print_board(bit_board: &u64) {
    println!();

    for rank in (0..8).rev() {
        for file in 0..8 {
            let board_index = (rank * 8) + file; 
            let bit_state = bit_board & (1u64 << board_index);

            if file == 0 { print!("{}   ", 1 + rank);}
            print!("{}  ", if bit_state != 0 {1} else {0});
        }

        println!();
    }
    println!("\n    A  B  C  D  E  F  G  H");
    println!("\nBitboard: {bit_board}");
}

pub const fn set_bit(bit_board: &mut u64, square: Square) {
    *bit_board |= 1u64 << square as u64;
}

pub const fn clear_bit(bit_board: &mut u64, square: Square) {
    *bit_board &= !(1u64 << square as u64);
}

pub const fn count_bits(bit_board: &u64) -> usize {
    bit_board.count_ones() as usize
}

pub const fn least_sig_bit(bit_board: &u64) -> Square {
    Square::from(bit_board.trailing_zeros() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_bits() {
        let mut bb = 0u64;
        set_bit(&mut bb, Square::A1);

        assert_eq!(count_bits(&bb), 1);

        set_bit(&mut bb, Square::A3);
        set_bit(&mut bb, Square::A2);
        set_bit(&mut bb, Square::A1);

        assert_eq!(count_bits(&bb), 3);
    }

    #[test]
    fn test_least_sig_bit() {
        let mut bb = 0u64;
        set_bit(&mut bb, Square::A3);

        assert_eq!(least_sig_bit(&bb), Square::A3);

        set_bit(&mut bb, Square::B3);
        set_bit(&mut bb, Square::B2);
        set_bit(&mut bb, Square::H8);
        set_bit(&mut bb, Square::C2);

        assert_eq!(least_sig_bit(&bb), Square::B2);
        clear_bit(&mut bb, Square::B2);
        assert_eq!(least_sig_bit(&bb), Square::C2);
    }

    #[test]
    fn test_rook_attack_map() {
        let board = Board::new();
        print_board(&board.bishop_attacks[4]);
    }

    #[test]
    fn test_get_rook_attack() {
        let board = Board::new();
        let mut occ = 0u64;

        print_board(&board.get_rook_attacks(Square::A6, occ));

        set_bit(&mut occ, Square::E3);
        set_bit(&mut occ, Square::G5);
        set_bit(&mut occ, Square::G3);

        print_board(&board.get_rook_attacks(Square::G3, occ));
    }

    #[test]
    fn test_get_bishop_attack() {
        let board = Board::new();
        let mut occ = 0u64;

        print_board(&board.get_bishop_attacks(Square::A3, occ));
        
        set_bit(&mut occ, Square::D6);
        print_board(&board.get_bishop_attacks(Square::G3, occ));
    }
}