use crate::attacks::*;
use crate::evaluation::GAMEPHASE;
use crate::magics::*;
use crate::types::*;
use std::fmt::Display;

pub mod makemove;
pub mod movegen;
pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub struct BoardState {
    pub pieces: [BitBoard; 6],
    pub occupancies: [BitBoard; 2],
    pub mailbox: [Option<(Side, Piece)>; 64],
    pub side_to_move: Side,
    pub enpassant: Option<Square>,
    pub castling_rights: CastlingRights,

    pub material_value: [i32; 2],
    pub pq_mg_value: [i32; 2],
    pub pq_eg_value: [i32; 2],
    pub game_phase: i32,

    pub half_move_clock: u8,
    pub full_move: usize,
    pub hash: u64,
}

impl BoardState {
    pub fn new() -> Self {
        BoardState {
            pieces: [BitBoard(0); 6],
            occupancies: [BitBoard(0); 2],
            mailbox: [None; 64],
            side_to_move: Side::White,
            enpassant: None,
            castling_rights: CastlingRights::new(),

            material_value: [0; 2],
            pq_mg_value: [0; 2],
            pq_eg_value: [0; 2],
            game_phase: 0,

            half_move_clock: 0,
            full_move: 0,
            hash: 0,
        }
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub struct Board {
    pub state: BoardState,
    pub state_stack: Vec<BoardState>,
    pub game_history: Vec<u64>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

//Little-Endian Rank-File Mapping
impl Board {
    pub fn new() -> Self {
        Board {
            state_stack: Vec::new(),
            state: BoardState::new(),
            game_history: Vec::new(),
        }
    }

    pub const fn get_piece_bb(&self, side: Side, piece: Piece) -> BitBoard {
        BitBoard(self.state.pieces[piece as usize].0 & self.state.occupancies[side as usize].0)
    }

    pub const fn get_piece_at_square(&self, square: Square) -> Option<(Side, Piece)> {
        self.state.mailbox[square as usize]
    }

    pub fn place_piece(&mut self, side: Side, piece: Piece, square: Square) {
        //Bitboards
        self.state.pieces[piece as usize].set_bit(square);
        self.state.occupancies[side as usize].set_bit(square);
        //Mailbox
        self.state.mailbox[square as usize] = Some((side, piece));
        //Material Eval
        self.state.material_value[side as usize] += piece.value();
        //Piece Square Table
        self.state.pq_mg_value[side as usize] += self.get_mg_score(piece, square, side);
        self.state.pq_eg_value[side as usize] += self.get_eg_score(piece, square, side);
        //Game Phase
        self.state.game_phase += GAMEPHASE[piece as usize];
        //Zobrist Hash
        self.state.hash ^= ZOBRIST.get_piece_num(side, piece, square);
    }

    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        //Bitboards
        self.state.pieces[piece as usize].clear_bit(square);
        self.state.occupancies[side as usize].clear_bit(square);
        //Mailbox
        self.state.mailbox[square as usize] = None;
        //Material Eval
        self.state.material_value[side as usize] -= piece.value();
        //Piece Square Table
        self.state.pq_mg_value[side as usize] -= self.get_mg_score(piece, square, side);
        self.state.pq_eg_value[side as usize] -= self.get_eg_score(piece, square, side);
        //Game Phase
        self.state.game_phase -= GAMEPHASE[piece as usize];
        //Zobrist Hash
        self.state.hash ^= ZOBRIST.get_piece_num(side, piece, square);
    }

    pub fn get_piece_attack(&self, side: Side, square: Square, piece: Piece) -> BitBoard {
        match piece {
            Piece::Pawn => self.get_pawn_attacks(square, side),
            Piece::Knight => self.get_knight_attacks(square),
            Piece::Bishop => self.get_bishop_attacks(square, self.get_all_occupancy()),
            Piece::Rook => self.get_rook_attacks(square, self.get_all_occupancy()),
            Piece::Queen => self.get_queen_attacks(square, self.get_all_occupancy()),
            Piece::King => self.get_king_attacks(square),
        }
    }

    pub fn get_all_attacks(&self, side: Side) -> BitBoard {
        let mut attacks = BitBoard(0);
        for i in 0..6 {
            for source in self.state.pieces[i + (side as usize * 6)].iter() {
                attacks |= self.get_piece_attack(side, source, Piece::from(i));
            }
        }

        attacks & !self.state.occupancies[side as usize]
    }

    pub const fn get_pawn_attacks(&self, square: Square, side: Side) -> BitBoard {
        PAWN_ATTACKS[side as usize][square as usize]
    }

    pub const fn get_knight_attacks(&self, square: Square) -> BitBoard {
        KNIGHT_ATTACKS[square as usize]
    }

    pub const fn get_king_attacks(&self, square: Square) -> BitBoard {
        KING_ATTACKS[square as usize]
    }

    pub fn get_bishop_attacks(&self, square: Square, board_occupancy: BitBoard) -> BitBoard {
        let occupancy = board_occupancy & BISHOP_MASKS[square as usize];
        let magic_index = get_magic_index(
            occupancy,
            BISHOP_OCCUPANCY_BIT_COUNTS[square as usize],
            BISHOP_MAGIC_NUMBERS[square as usize],
        );

        let offset = (square as usize * 512) + magic_index;

        BISHOP_ATTACKS[offset]
    }

    pub fn get_rook_attacks(&self, square: Square, board_occupancy: BitBoard) -> BitBoard {
        let occupancy = board_occupancy & ROOK_MASKS[square as usize];
        let magic_index = get_magic_index(
            occupancy,
            ROOK_OCCUPANCY_BIT_COUNTS[square as usize],
            ROOK_MAGIC_NUMBERS[square as usize],
        );

        let offset = (square as usize * 4096) + magic_index;

        ROOK_ATTACKS[offset]
    }

    pub fn get_queen_attacks(&self, square: Square, board_occupancy: BitBoard) -> BitBoard {
        self.get_bishop_attacks(square, board_occupancy)
            | self.get_rook_attacks(square, board_occupancy)
    }

    pub fn get_all_occupancy(&self) -> BitBoard {
        self.state.occupancies[Side::White as usize] | self.state.occupancies[Side::Black as usize]
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("\n\n");
        for rank in (0..8).rev() {
            output.push_str(&format!("{}   ", 1 + rank));
            for file in 0..8 {
                let square = Square::from_rank_and_file(rank, file);
                let piece: Option<(Side, Piece)> = self.get_piece_at_square(square);

                if let Some(p) = piece {
                    let mut s = format!(" {} ", p.1);
                    if let Side::Black = p.0 {
                        s = s.to_lowercase();
                    }
                    output.push_str(&s);
                } else {
                    output.push_str(" . ");
                }
            }
            output.push('\n');
        }
        output.push_str("\n     A  B  C  D  E  F  G  H\n");
        output.push_str(&format!(
            "\n     Side to move: {} \n     Castling: {}\n     Enpassant: {:?}\n",
            self.state.side_to_move, self.state.castling_rights, self.state.enpassant
        ));
        write!(f, "{}", output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::moves::Move;

    #[test]
    fn test_get_rook_attack() {
        let board = Board::new();
        let mut occ = BitBoard(0);

        board.get_rook_attacks(Square::A6, occ).print_board();

        occ.set_bit(Square::E3);
        occ.set_bit(Square::G5);
        occ.set_bit(Square::G3);

        board.get_rook_attacks(Square::G3, occ).print_board();
    }

    #[test]
    fn test_get_bishop_attack() {
        let board = Board::new();
        let mut occ = BitBoard(0);

        board.get_bishop_attacks(Square::A3, occ).print_board();

        occ.set_bit(Square::D6);
        board.get_bishop_attacks(Square::G3, occ).print_board();
    }

    #[test]
    fn test_get_queen_attack() {
        let board = Board::new();
        let mut occ = BitBoard(0);

        board.get_queen_attacks(Square::A6, occ).print_board();

        occ.set_bit(Square::E3);
        occ.set_bit(Square::G5);
        occ.set_bit(Square::G3);
        occ.set_bit(Square::D6);

        board.get_queen_attacks(Square::G3, occ).print_board();
        board.get_queen_attacks(Square::E4, occ).print_board();
    }

    #[test]
    fn test_board_occupancy() {
        let mut board = Board::from_fen(STARTING_FEN).unwrap();
        board.remove_piece(Side::White, Piece::Pawn, Square::A2);
        board.get_all_occupancy().print_board();
        board.state.occupancies[Side::Black as usize].print_board();
        board.state.occupancies[Side::White as usize].print_board();
    }

    #[test]
    fn test_full_board_print() {
        let board = Board::new();
        println!("{board}");
    }

    #[test]
    fn test_get_all_attacks() {
        let mut board = Board::from_fen(STARTING_FEN).unwrap();
        let m = Move::new(Square::E2, Square::E4, moves::MoveKind::DoublePawn);
        let _ = board.make_move(m);
        println!("{board}");
        board.get_all_attacks(Side::White).print_board();
    }
}
