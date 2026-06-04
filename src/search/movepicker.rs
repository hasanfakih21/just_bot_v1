use crate::{
    board::{Board, movegen::MoveGenKind},
    search::data::SearchData,
    types::{Move, MoveKind, MoveList, Square},
};

#[derive(Debug, PartialEq)]
pub enum Status {
    HashMove,
    FirstNoisy,
    Noisy,
    Quiet,
}

#[derive(Debug)]
pub struct MovePicker {
    moves: MoveList,
    tt_move: Option<Move>,
    status: Status,
}

impl MovePicker {
    pub fn new(board: &Board, data: &SearchData) -> MovePicker {
        Self {
            moves: MoveList::new(),
            tt_move: if let Some(e) = data.tt.get_entry(board.board_state.hash)
                && board.board_state.hash == e.get_key()
            {
                Some(e.get_best_move())
            } else {
                None
            },
            status: if let Some(e) = data.tt.get_entry(board.board_state.hash)
                && board.board_state.hash == e.get_key()
            {
                Status::HashMove
            } else {
                Status::FirstNoisy
            },
        }
    }

    pub fn next(&mut self, board: &Board, data: &SearchData, quiesce: bool) -> Option<Move> {
        if self.status == Status::HashMove {
            self.status = Status::FirstNoisy;
            if !quiesce || !self.tt_move.unwrap().get_kind().is_quiet() {
                return self.tt_move;
            }
        }

        if self.status == Status::FirstNoisy {
            board.append_moves(MoveGenKind::Noisy, &mut self.moves);
            self.remove_tt_move();
            self.score_noisy_moves(board);
            self.status = Status::Noisy;
            if !self.moves.is_empty() {
                return Some(self.best_move());
            }
        }

        if self.status == Status::Noisy {
            if self.moves.is_empty() {
                self.status = Status::Quiet;
                if !quiesce {
                    board.append_moves(MoveGenKind::Quiet, &mut self.moves);
                    self.remove_tt_move();
                    self.score_quiet_moves(board, data);
                }
            } else {
                return Some(self.best_move());
            }
        }

        if self.status == Status::Quiet && !self.moves.is_empty() && !quiesce {
            return Some(self.best_move());
        }

        None
    }

    fn score_noisy_moves(&mut self, board: &Board) {
        for entry in self.moves.iter_mut() {
            let mv = entry.mv;
            if mv.get_kind().is_capture() {
                entry.score = Some(score_attack_move(mv, board));
            } else if mv.get_kind().is_queen_promotion() {
                entry.score = Some(500);
            }
        }
    }

    fn score_quiet_moves(&mut self, board: &Board, data: &SearchData) {
        for entry in self.moves.iter_mut() {
            let mv = entry.mv;
            entry.score = Some(data.history.get(board.board_state.side_to_move, mv));
        }
    }

    fn best_move(&mut self) -> Move {
        let mut best_index = 0;
        let mut best_score = i32::MIN;

        for (index, entry) in self.moves.iter().enumerate() {
            let entry_score = entry.score.unwrap_or(i32::MIN);
            if entry_score >= best_score {
                best_score = entry_score;
                best_index = index;
            }
        }

        self.moves.remove(best_index).unwrap().mv
    }

    fn remove_tt_move(&mut self) {
        if let Some(tt_mv) = self.tt_move
            && let Some(index) = self.moves.iter().position(|e| tt_mv == e.mv)
        {
            self.moves.remove(index);
        }
    }
}

pub const fn score_attack_move(mv: Move, board: &Board) -> i32 {
    let attacker = board.get_piece_at_square(mv.get_from()).unwrap().1;
    let victim = match mv.get_kind() {
        MoveKind::EnPassant => {
            board
                .get_piece_at_square(Square::from(mv.get_to() as usize ^ 8))
                .unwrap()
                .1
        }
        _ => board.get_piece_at_square(mv.get_to()).unwrap().1,
    };

    victim.value() - attacker.value()
}

#[cfg(test)]
pub mod tests {
    use crate::{
        board::Board,
        search::{data::SearchData, movepicker::MovePicker},
    };

    #[test]
    fn test_move_picker() {
        let board =
            Board::from_fen("rnbqkb1r/pp3p2/4pnpp/1p1p2N1/1Q1P4/BP2P3/P1PN1PPP/R3K2R b KQkq - 0 1");
        let mut move_picker = MovePicker::new(&board, &SearchData::default());
        println!("{}", move_picker.moves);
        //println!("{:?}", move_picker);
        while let Some(m) = move_picker.next(&board, &SearchData::default(), true) {
            println!("{m}");
        }
    }
}
