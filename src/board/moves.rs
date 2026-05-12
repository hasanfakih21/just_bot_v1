use crate::board::{Board, Piece, Side, Square};

pub enum MoveType {
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

impl Board {
    pub fn is_attacked_at_by(&self, square: Square, side: Side) -> bool {
        let pawns = self.board_pieces[side as usize][Piece::Pawn as usize];
        if (pawns & self.get_pawn_attacks(square, side.other())) != 0 {return true}

        let knights = self.board_pieces[side as usize][Piece::Knight as usize];
        if (knights & self.get_knight_attacks(square)) != 0 {return true}

        let king = self.board_pieces[side as usize][Piece::King as usize];
        if (king & self.get_king_attacks(square)) != 0 {return true}

        let bishop_queens = self.board_pieces[side as usize][Piece::Bishop as usize] | self.board_pieces[side as usize][Piece::Queen as usize];
        if (bishop_queens & self.get_bishop_attacks(square, self.get_all_occupancy())) != 0 {return true}

        let rook_queens = self.board_pieces[side as usize][Piece::Rook as usize] | self.board_pieces[side as usize][Piece::Queen as usize];
        if (rook_queens & self.get_rook_attacks(square, self.get_all_occupancy())) != 0 {return true}

        false
    }

    pub fn generate_moves(&self) {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_attacked_at_by() {
        use Square::*;
        use Side::*;

        let board = Board::from_fen("8/8/8/3p4/8/8/5N2/8 w - - 0 1");
        assert!(board.is_attacked_at_by(C4, Black));
        assert!(board.is_attacked_at_by(D3, White));
        assert!(!board.is_attacked_at_by(F2, Black));

        let board2 = Board::from_fen("6Q1/8/2R5/8/5b2/1q6/8/6K1 w - - 0 1");
        assert!(board2.is_attacked_at_by(C3, White));
        assert!(board2.is_attacked_at_by(B3, White));
        assert!(board2.is_attacked_at_by(F1, White));
        assert!(board2.is_attacked_at_by(A2, Black));
        assert!(board2.is_attacked_at_by(H6, Black));
        assert!(!board2.is_attacked_at_by(F5, Black));
        assert!(!board2.is_attacked_at_by(F5, White));
    }
}