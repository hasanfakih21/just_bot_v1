use std::f32::consts::PI;

use crate::board::Board;
use crate::search::data::SearchData;
use crate::search::movepicker::MovePicker;
use crate::types::*;

pub mod data;
pub mod movepicker;
pub mod time;

impl Board {
    //Needs fixing
    pub fn detect_repetitions(&self) -> usize {
        let half_moves = self.board_state.half_move_clock as usize;
        let mut count = 0;

        if self.game_history.len() < half_moves {
            return 0;
        }

        let last_halfmove_ply = self.game_history.len() - half_moves;
        for position in self.game_history[last_halfmove_ply..].iter() {
            if self.board_state.hash == *position {
                count += 1
            }
        }

        count
    }
}

pub trait NodeType {
    const PV: bool;
    const ROOT: bool;
}

pub struct PV;
pub struct Root;
pub struct NonPV;

impl NodeType for PV {
    const PV: bool = true;
    const ROOT: bool = false;
}

impl NodeType for NonPV {
    const PV: bool = false;
    const ROOT: bool = false;
}

impl NodeType for Root {
    const PV: bool = true;
    const ROOT: bool = true;
}

pub fn search_runner(board: &mut Board, data: &mut SearchData) -> Option<(Move, i32)> {
    data.clear_node_count();
    data.reset_pv();
    data.start_time();
    let mut depth = 1;

    //Initialize with move from first depth
    let mut best_move = search_root(data, depth, board, -INFINITY, INFINITY)?;
    depth += 1;

    //Aspiration Window
    let mut score = best_move.1;
    let mut alpha_window = 25;
    let mut beta_window = 25;
    let mut alpha = score - alpha_window;
    let mut beta = score + beta_window;

    //All infos belonging to the pv should be sent together e.g. info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv e2e4 e7e5 g1f3
    println!(
        "info depth {} time {} score cp {} nodes {} nps {} pv {} hashfull {}",
        depth - 1,
        data.time.elapsed().as_millis(),
        score,
        data.get_total_nodes_searched(),
        data.nodes_per_second(),
        data.get_pv(),
        data.tt.hashfull(),
    );

    //Iterative Deepening
    loop {
        let deeper_move = search_root(data, depth, board, alpha, beta);
        if data.over_limit() || depth >= MAX_DEPTH - 1 {
            println!(
                "Searched for: {}ms\nTime Limit: {}ms",
                data.time.elapsed().as_millis(),
                data.time.get_limit()
            );
            break;
        }
        let new_score = deeper_move.unwrap().1;
        if new_score <= alpha {
            //Failed Low
            //println!("Failed Low Score: {new_score} Window: {alpha_window} Alpha: {alpha} Depth: {depth}");
            alpha_window *= 2;
            alpha -= alpha_window;
            continue;
        } else if new_score >= beta {
            //Failed High
            //println!("Failed High Score: {new_score} Window {beta_window} Beta: {beta} Depth: {depth}");
            beta_window *= 2;
            beta += beta_window;
            continue;
        }

        depth += 1;

        best_move = deeper_move?;
        score = new_score;
        alpha_window = 25;
        beta_window = 25;
        alpha = score - alpha_window;
        beta = score + beta_window;
        println!(
            "info depth {} time {} score cp {} nodes {} nps {} pv {} hashfull {}",
            depth - 1,
            data.time.elapsed().as_millis(),
            score,
            data.get_total_nodes_searched(),
            data.nodes_per_second(),
            data.get_pv(),
            data.tt.hashfull()
        );
    }

    Some(best_move)
}

//Root Search
pub fn search_root (
    data: &mut SearchData,
    depth: usize,
    board: &mut Board,
    alpha: i32,
    beta: i32,
) -> Option<(Move, i32)> {
    let mut best_score = -INFINITY;
    let mut best_move: Option<(Move, i32)> = None;
    let ply = 0;
    data.clear_pv(0);

    let mut move_picker = MovePicker::new(board, data);

    while let Some(m) = move_picker.next(board, false) {
        if board.make_move(m).is_ok() {
            let score = -search::<PV>(data, depth - 1, board, -beta, -alpha, ply + 1);

            board.unmake_move();
            if data.over_limit() {
                return None;
            }
            //println!("{m}: {score}"); //Debug Print
            if score >= best_score {
                data.add_pv_move(m, ply);
                best_score = score;
                best_move = Some((m, score));
            }
        }
    }

    if let Some((m, s)) = best_move {
        data.tt
            .add_entry(m, s, Bound::Exact, board.board_state.hash, depth);
    }
    best_move
}

pub fn search<Node: NodeType> (
    data: &mut SearchData,
    depth: usize,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ply: usize,
) -> i32 {
    if depth == 0 {
        if board.king_in_check() {
            return search_checks(data, board, alpha, beta, ply);
        } else {
            return quiesce(data, board, alpha, beta, ply); //Horizon Node
        }
    }

    data.add_nodes(1);
    data.clear_pv(ply);

    if board.board_state.half_move_clock > 4 {
        //50 move rule
        if board.board_state.half_move_clock >= 100 {
            return 0;
        }
        //We need to check history if positions were repeated only for the side to move.
        let count = board.detect_repetitions();
        if count >= 2 {
            return 0;
        }
    }

    //TT Cutoffs only if depth of entry is greater or equal to the depth of the current node
    if let Some(e) = data.tt.get_entry(board.board_state.hash)
        && !Node::PV
        && board.board_state.hash == e.get_key()
        && e.get_depth() >= depth
        && e.get_score().abs() < MATE_CUTOFF //Mate scores need to be properly adjusted for cutoffs
    {
        let tt_score = e.get_score();

        match e.get_bound() {
            Bound::Exact => return tt_score,
            Bound::Lower => {
                if tt_score >= beta {
                    return tt_score;
                }
            }
            Bound::Upper => {
                if tt_score < alpha {
                    return tt_score;
                }
            }
        }
    }

    //Null Move Pruning
    if !Node::PV && !board.king_in_check() && !board.only_king_and_pawns() {
        let r = 4; 
        board.make_null_move();
        let null_move_score = -search::<NonPV>(data, depth.saturating_sub(r), board, -beta, -(beta - 1), ply + 1);
        board.unmake_move();
        if null_move_score >= beta {
            return null_move_score
        }
    }

    let mut legal_moves = 0;
    let mut best_score = -INFINITY;
    let mut best_move: Option<Move> = None;
    let mut bound = Bound::Upper; //Fail-high means score is atleast this good so lower-bound/Fail-low means the score is an upper bound

    let mut move_picker = MovePicker::new(board, data);

    while let Some(m) = move_picker.next(board, false) {
        if board.make_move(m).is_ok() {
            legal_moves += 1;
            let mut score = best_score;

            //PVS 
            //Late Move Reductions
            if depth > 3 && !Node::PV { 
                let reduction = (0.99 + f32::ln(depth as f32) * f32::ln(legal_moves as f32)) / PI; //https://www.chessprogramming.org/Late_Move_Reductions Obsidian formula
                //println!("Depth: {} Reduction: {}", depth, reduction);
                let reduced_depth = (depth - 1).saturating_sub(reduction as usize);
                score = -search::<NonPV>(data, reduced_depth, board, -alpha - 1, -alpha, ply + 1);
                if score > alpha && reduced_depth < depth - 1 {
                    score = -search::<NonPV>(data, depth - 1, board, -alpha - 1, -alpha, ply + 1);
                }
            } else if !Node::PV || legal_moves > 1 {
                score = -search::<NonPV>(data, depth - 1, board, -alpha - 1, -alpha, ply + 1);
            }

            if Node::PV && (legal_moves == 1 || score > alpha) {
                score = -search::<PV>(data, depth - 1, board, -beta, -alpha, ply + 1);
            }
                    
            board.unmake_move();
            if data.over_limit() {
                return TIMEOUT_SCORE;
            }

            if score > alpha {
                bound = Bound::Exact;
                data.add_pv_move(m, ply);
                alpha = score;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }

            if score >= beta {
                if let Some(m) = best_move {
                    let tt_score = best_score;
                    data.tt
                        .add_entry(m, tt_score, Bound::Lower, board.board_state.hash, depth);
                }
                return best_score;
            }
        }
    }

    if legal_moves == 0 {
        if board.is_king_in_attack(board.board_state.side_to_move) {
            return -MATE_SCORE + ply as i32;
        } else {
            return 0;
        }
    }

    if let Some(m) = best_move {
        let tt_score = best_score;
        data.tt
            .add_entry(m, tt_score, bound, board.board_state.hash, depth);
    }

    best_score
}

pub fn quiesce(
    data: &mut SearchData,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    _ply: usize,
) -> i32 {
    data.add_nodes(1);
    let static_eval = board.evaluate();
    // if board.is_king_in_attack(board.board_state.side_to_move) {
    //     return search_checks(data, board, alpha, beta, _ply);
    // }

    let mut best_score = static_eval;
    //let mut bound = Bound::Upper;
    //let mut best_move: Option<Move> = None;

    if best_score >= beta {
        return best_score;
    }
    if best_score > alpha {
        alpha = best_score;
    }

    let mut move_picker = MovePicker::new(board, data);

    while let Some(m) = move_picker.next(board, true) {
        if board.make_move(m).is_ok() {
            let score = -quiesce(data, board, -beta, -alpha, _ply + 1);
            board.unmake_move();
            if data.over_limit() {
                return TIMEOUT_SCORE;
            }

            if score >= beta {
                // if let Some(m) = best_move {
                //     board
                //         .tt
                //         .add_entry(m, best_score, Bound::Lower, board.board_state.hash, 0);
                // }
                return score;
            }
            if score > best_score {
                best_score = score;
                //best_move = Some(*m);
            }
            if score > alpha {
                alpha = score;
                //bound = Bound::Exact;
            }
        }
    }

    // if let Some(m) = best_move {
    //     board
    //         .tt
    //         .add_entry(m, best_score, bound, board.board_state.hash, 0);
    // }

    best_score
}

pub fn search_checks(
    data: &mut SearchData,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ply: usize,
) -> i32 {
    let mut best_score = -INFINITY;
    let mut legal_moves = 0;
    data.add_nodes(1);

    if board.board_state.half_move_clock > 4 {
        //50 move rule
        if board.board_state.half_move_clock >= 100 {
            return 0;
        }
        //We need to check history if positions were repeated only for the side to move.
        let count = board.detect_repetitions();
        if count >= 2 {
            return 0;
        }
    }

    if !board.king_in_check() {
        return quiesce(data, board, alpha, beta, ply);
    }

    let mut move_picker = MovePicker::new(board, data);

    while let Some(m) = move_picker.next(board, false) {
        if board.make_move(m).is_ok() {
            legal_moves += 1;
            let score = -search_checks(data, board, -beta, -alpha, ply + 1);
            board.unmake_move();
            if data.over_limit() {
                return TIMEOUT_SCORE;
            }

            if score >= beta {
                return score;
            }
            if score > best_score {
                best_score = score;
            }
            if score > alpha {
                alpha = score;
            }
        }
    }

    if legal_moves == 0 {
        if board.is_king_in_attack(board.board_state.side_to_move) {
            return -MATE_SCORE + ply as i32;
        } else {
            return 0;
        }
    }

    best_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_search() {
        let mut board = Board::from_fen(STARTING_FEN);
        let mut data = SearchData::default();
        let best_move = search_root(&mut data, 5, &mut board, -INFINITY, INFINITY);
        if let Some(m) = best_move {
            println!("Best move: {}", m.0);
        }
    }

    #[test]
    fn test_order_moves() {
        let board =
            Board::from_fen("rnbqkb1r/pp3p2/4pnpp/1p1p2N1/1Q1P4/BP2P3/P1PN1PPP/R3K2R b KQkq - 0 1");
        let mut move_picker = MovePicker::new(&board, &SearchData::default());
        let first_move = move_picker.next(&board, false).unwrap();

        assert_eq!(
            first_move,
            Move::new(Square::F8, Square::B4, MoveKind::Capture)
        );

        let board =
            Board::from_fen("rnbq1rk1/pN1p1ppp/4n2b/2p1p3/N1BP3R/2P2Q2/PP3PPP/2B1K2R w K - 0 1");
        let mut move_picker = MovePicker::new(&board, &SearchData::default());
        let first_move = move_picker.next(&board, false).unwrap();

        assert_eq!(
            first_move,
            Move::new(Square::B7, Square::D8, MoveKind::Capture)
        );
    }

    #[test]
    fn test_repetion_detection() {
        use MoveKind::*;
        use Square::*;

        let mut board = Board::from_fen("8/6K1/3N4/8/5Q2/8/1kr5/8 w - - 0 1");
        let _ = board.make_move(Move::new(F4, E4, QuietMove));
        let _ = board.make_move(Move::new(C2, C1, QuietMove));
        let _ = board.make_move(Move::new(E4, F4, QuietMove));
        let _ = board.make_move(Move::new(C1, C2, QuietMove));
        let _ = board.make_move(Move::new(F4, E4, QuietMove));
        let _ = board.make_move(Move::new(C2, C1, QuietMove));
        let _ = board.make_move(Move::new(E4, F4, QuietMove));

        let mut data = SearchData::default();
        let (m, score) = search_root(&mut data, 3, &mut board, -INFINITY, INFINITY).unwrap();
        println!(
            "{:?}\nCurrent Hash: {}",
            board.game_history, board.board_state.hash
        );
        println!("Repetions counted: {}", board.detect_repetitions());
        assert_eq!(score, 0);
        assert_eq!(m, Move::new(C1, C2, QuietMove));
    }

    #[test]
    fn test_mate_in_one() {
        let mut data = SearchData::default();
        let mut board =
            Board::from_fen("r1b4r/p1p1q3/1bppk3/4pp2/3PP1Q1/2P1R3/PP3PPP/RN4K1 w - - 0 18");
        data.set_playing_as(board.board_state.side_to_move);
        let best_move = search_root(&mut data, 1, &mut board, -INFINITY, INFINITY);
        println!("Best Move: {}", best_move.unwrap().0);
        assert_eq!(
            Move::new(Square::G4, Square::F5, MoveKind::Capture),
            best_move.unwrap().0
        );
    }

    #[test]
    fn test_mate_in_four() {
        let mut data = SearchData::default();
        let mut board = Board::from_fen("6k1/5pp1/5n1p/8/5P1q/2RQ3P/B5PK/8 b - - 0 36");
        data.set_playing_as(board.board_state.side_to_move);
        let best_move = search_root(&mut data, 4, &mut board, -INFINITY, INFINITY);
        println!("Best Move: {}", best_move.unwrap().0);
        assert_eq!(
            Move::new(Square::F6, Square::G4, MoveKind::QuietMove),
            best_move.unwrap().0
        );
    }

    #[test]
    fn test_pv_line() {
        use MoveKind::*;
        use Square::*;

        let mut data = SearchData::default();
        let mut board = Board::from_fen("6k1/5pp1/5n1p/8/5P1q/2RQ3P/B5PK/8 b - - 0 36");
        data.set_playing_as(board.board_state.side_to_move);
        data.get_time_settings().btime = 1000000;
        data.start_time();
        let best_move = search_root(&mut data, 7, &mut board, -INFINITY, INFINITY);
        println!("PV: {}", data.get_pv());
        println!("Eval: {}", best_move.unwrap().1);
        let mut pv_line = MoveList::new();
        pv_line.push(Move::new(F6, G4, QuietMove));
        pv_line.push(Move::new(H2, G1, QuietMove));
        pv_line.push(Move::new(H4, F2, QuietMove));
        pv_line.push(Move::new(G1, H1, QuietMove));
        pv_line.push(Move::new(F2, E1, QuietMove));
        pv_line.push(Move::new(D3, F1, QuietMove));
        pv_line.push(Move::new(E1, F1, Capture));

        assert_eq!(pv_line.to_string(), data.get_pv().to_string());

        println!("Best Move: {}", best_move.unwrap().0);
        assert_eq!(
            Move::new(Square::F6, Square::G4, MoveKind::QuietMove),
            best_move.unwrap().0
        );
    }

    #[test]
    fn test_bugged_position() {
        let mut board = Board::from_fen("6k1/5pp1/7p/8/5Pn1/2R4P/B5P1/4qQ1K b - - 6 39");
        println!("Hash: {}", board.board_state.hash);
        //Position hash: 6128121706435820836

        board = Board::from_fen("6k1/5pp1/7p/8/5Pn1/2RQ3P/B4qP1/6K1 w - - 3 38");
        println!("Hash 2: {}", board.board_state.hash);
        //Position hash: 16381162810209017462

        board = Board::from_fen("6k1/5pp1/7p/8/5Pnq/2RQ3P/B5P1/6K1 b - - 2 37");
        println!("Hash 3: {}", board.board_state.hash);
        //Position hash: 3246015867840709621
    }

    #[test]
    fn test_transposition_timeout() {
        let mut data = SearchData::new();
        data.set_playing_as(Side::Black);
        data.get_time_settings().btime = 8080;
        let mut board = Board::from_fen("6k1/2p5/4R1pp/1p1r4/pP1p4/P5PP/2P2P2/6K1 b - - 0 32");
        let _ = search_runner(&mut board, &mut data);
        println!();
        let _ = search_runner(&mut board, &mut data);
        println!();
        let _ = search_runner(&mut board, &mut data);
        println!();
        let _ = search_runner(&mut board, &mut data);
        println!();
        let _ = search_runner(&mut board, &mut data);
        println!();

        assert!(!data.tt.0.iter().any(|i| {
            if let Some(e) = i {
                e.get_score() == TIMEOUT_SCORE
            } else {
                false
            }
        }));

        //I want to count the number of entries in the table
        let total_size = data.tt.0.len();
        assert_eq!(total_size, ENTRIES);
        let count = data.tt.0.iter().filter(|e| e.is_some()).count();

        println!("Total Size: {total_size} Number of Entries: {count}");
        println!("Hashfull: {}", (count as f32 / total_size as f32) * 1000.0);
        println!("{}", data.tt.hashfull());
    }
}
