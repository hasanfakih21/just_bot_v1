use crate::board::{Board, Piece, Square, moves::Move};

impl Board {
    pub fn parse_move(&self, move_string: &str) -> Result<Move, &str> {
        let from = Square::try_from(&move_string[0..2]).unwrap();
        let to = Square::try_from(&move_string[2..4]).unwrap();
        let mut promotion_piece: Option<Piece> = None;
        if move_string.len() > 4 {
            match &move_string[4..] {
                "n" => promotion_piece = Some(Piece::Knight),
                "b" => promotion_piece = Some(Piece::Bishop),
                "r" => promotion_piece = Some(Piece::Rook),
                "q" => promotion_piece = Some(Piece::Queen),
                _ => ()
            }
        }

        let move_list = self.generate_all_moves();
        if let Some(m) = move_list.iter().find(|e| e.get_from() == from && e.get_to() == to && e.get_promoted_piece() == promotion_piece) {
            Ok(*m)
        } else {
            Err("Invalid move string")
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::board::constants::STARTING_FEN;
    use super::*;

    #[test]
    fn test_parse_move() {
        let board = Board::from_fen(STARTING_FEN);
        if let Ok(m) = board.parse_move("e2e4") {
            println!("{m}");
        }
    }
}