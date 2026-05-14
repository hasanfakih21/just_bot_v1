use crate::board::{CastlingRights, Piece, Side, Square, bitboard::BitBoard, moves::{Move, MoveKind}};
use super::Board;

pub struct BoardState {
    pub board_pieces: [[BitBoard; 6]; 2],
    pub pieces_on_squares: [Option<(Side, Piece)>; 64],
    pub board_occupancies: [BitBoard; 2],
    pub side_to_move: Side,
    pub enpassant: Option<Square>,
    pub castling_rights: CastlingRights,
}

impl Board {
    pub fn make_move(&mut self, m: Move) {
        let from = m.get_from();
        let to = m.get_to();
        let kind = m.get_kind();
        let (side, piece) = self.get_piece_at_square(from).unwrap();

        if kind.is_quiet() {
            self.copy_state();
            self.remove_piece(side, piece, from);
            self.place_piece(side, piece, to);
            if let MoveKind::DoublePawn = kind {self.enpassant = Some(Square::from(from as usize ^ 8))}
            if piece == Piece::King {
                self.castling_rights.clear_king_side(side);
                self.castling_rights.clear_queen_side(side);
            }
        }

        else {
            self.copy_state();
            self.remove_piece(side, piece, from);
        }
    }

    pub fn unmake_move(&mut self) {
        if let Some(prev_state) = self.state_stack.pop() {
            self.board_pieces = prev_state.board_pieces;
            self.pieces_on_squares = prev_state.pieces_on_squares;
            self.board_occupancies = prev_state.board_occupancies;
            self.side_to_move = prev_state.side_to_move;
            self.enpassant = prev_state.enpassant;
            self.castling_rights = prev_state.castling_rights;
        }
    }

    pub fn copy_state(&mut self) {
        self.state_stack.push(
            BoardState {
                board_pieces: self.board_pieces,
                pieces_on_squares: self.pieces_on_squares,
                board_occupancies: self.board_occupancies,
                side_to_move: self.side_to_move,
                enpassant: self.enpassant,
                castling_rights: self.castling_rights,
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Square;

    #[test]
    fn test_make_move() {
        println!("{:?}", Square::from(Square::G4 as usize ^ 8));
    }
}