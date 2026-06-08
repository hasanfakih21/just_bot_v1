use crate::board::Board;
use crate::board::movegen::MoveGenKind;
use crate::types::{Castling, Move, Piece, Side, Square, ZOBRIST, pieces};

#[derive(Debug)]
pub struct FenParseError;

impl From<pieces::InvalidPiece> for FenParseError {
    fn from(_: pieces::InvalidPiece) -> Self {
        FenParseError
    }
}

//Starting Position: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
//[pieces] [turn to move] [castling rights] [enpassant] [halfmove clock] [fullmove clock]

//FEN String Parser
impl Board {
    pub fn from_fen(fen_string: &str) -> Result<Self, FenParseError> {
        let mut board = Board::new();
        let mut fen = fen_string.split(" ");
        let piece_string = fen.next().ok_or(FenParseError)?;
        let ranks = piece_string.split('/');

        for (rank, r_str) in ranks.rev().enumerate() {
            let mut file: usize = 0;
            for p in r_str.chars() {
                if let Some(num) = p.to_digit(10) {
                    file += num as usize;
                    continue;
                }

                let side = if p.is_ascii_uppercase() {
                    Side::White
                } else {
                    Side::Black
                };
                let piece = Piece::from_char(p)?;
                let square = Square::from_rank_and_file(rank, file);
                board.place_piece(side, piece, square);

                file += 1;
            }
        }

        let turn = fen.next().ok_or(FenParseError)?;
        match turn {
            "w" => board.board_state.side_to_move = Side::White,
            "b" => board.board_state.side_to_move = Side::Black,
            _ => eprintln!("Invalid side to move"),
        }

        let castling_rights = fen.next().ok_or(FenParseError)?;
        for c in castling_rights.chars() {
            if c == '-' {
                continue;
            }

            board
                .board_state
                .castling_rights
                .set(Castling::from(c) as u8);
        }

        if let Some(enpassant) = fen.next()
            && let Ok(square) = Square::try_from(enpassant)
        {
            board.board_state.enpassant = Some(square);
        }

        if let Some(half_move) = fen.next()
            && let Ok(i) = half_move.parse::<u8>()
        {
            board.board_state.half_move_clock = i;
        }

        if let Some(full_move) = fen.next()
            && let Ok(i) = full_move.parse::<usize>()
        {
            board.board_state.full_move = i;
        }

        if board.board_state.side_to_move == Side::Black {
            board.board_state.hash ^= ZOBRIST.get_side_num()
        }

        board.board_state.hash ^= ZOBRIST.get_castling_num(board.board_state.castling_rights);
        if let Some(square) = board.board_state.enpassant {
            board.board_state.hash ^= ZOBRIST.get_enpassant_num(square);
        }

        Ok(board)
    }

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
                _ => (),
            }
        }

        let move_list = self.generate_moves(MoveGenKind::All);
        if let Some(m) = move_list.iter().find(|e| {
            e.mv.get_from() == from
                && e.mv.get_to() == to
                && e.mv.get_promoted_piece() == promotion_piece
        }) {
            Ok(m.mv)
        } else {
            Err("Invalid move string")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::STARTING_FEN;

    #[test]
    fn test_from_fen() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        println!("{board}");

        let board3 =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        println!("{board3}");

        let board4 =
            Board::from_fen("rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1")
                .unwrap();
        println!("{board4}");

        let board5 =
            Board::from_fen("r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9")
                .unwrap();
        println!("{board5}");

        let board5 =
            Board::from_fen("rnbqkbnr/pp3ppp/4p3/2pp4/3P4/2P2N2/PP2PPPP/RNBQKB1R w KQkq c6 0 4")
                .unwrap();
        println!("{board5}");

        let board6 =
            Board::from_fen("rnb1kbnr/pp1q1pp1/4p2p/2p1N3/3Pp3/2P5/PP2BPPP/RNBQK1R1 b Qkq - 1 7")
                .unwrap();
        println!("{board6}");
        println!("Half move: {}", board6.board_state.half_move_clock);
        println!("Full move: {}", board6.board_state.full_move);
    }
}
