use crate::board::{Board, moves::Move};

pub fn best_move(depth: usize, board: &mut Board) -> Option<Move> { 
    let mut max = -10000;
    let mut best_move: Option<Move> = None;

    for m in board.generate_all_moves().iter() {
        if board.make_move(*m).is_ok() {
            let score = -negamax(depth - 1, board);
            println!("{m}: {score}");
            if score >= max {
                max = score;
                best_move = Some(*m);
            }
            board.unmake_move();
        }
    }

    best_move
}

pub fn negamax(depth: usize, board: &mut Board) -> i32 {
    if depth == 0 {
        return board.evaluate();
    }

    let mut max = -10000;
    for m in board.generate_all_moves().iter() {
        if board.make_move(*m).is_ok() {
            let score = -negamax(depth - 1, board);
            max = max.max(score);
            board.unmake_move();
        }
    }

    max
}

#[cfg(test)]
mod tests {
    use crate::{board::{Board, constants::STARTING_FEN}, search::best_move};

    #[test]
    fn test_negamax() {
        let mut board = Board::from_fen(STARTING_FEN); 
        let best_move = best_move(5, &mut board);
        if let Some(m) = best_move {
            println!("Best move: {m}");
        }
    }
}