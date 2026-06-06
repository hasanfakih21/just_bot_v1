use crate::board::Board;
use crate::search::data::SearchData;
use crate::search::movepicker::MovePicker;
use crate::types::*;

pub mod data;
pub mod movepicker;
pub mod time;

#[cfg(test)]
mod tests;

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
    data.time.set_depth_limit();

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
        if data.over_limit() || depth > data.time.depth_limit() {
            break;
        }

        let new_score = deeper_move.unwrap().1;
        if new_score <= alpha {
            //Failed Low
            alpha_window *= 2;
            alpha -= alpha_window;
            continue;
        } else if new_score >= beta {
            //Failed High
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
pub fn search_root(
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

    while let Some(m) = move_picker.next(board, data, false) {
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

pub fn search<Node: NodeType>(
    data: &mut SearchData,
    depth: usize,
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ply: usize,
) -> i32 {
    let in_check = board.king_in_check();

    if depth == 0 {
        if in_check {
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
        && e.get_score().abs() < MATE_CUTOFF
    //Mate scores need to be properly adjusted for cutoffs
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

    //Reverse Futillity Pruning (RFP)
    if !in_check && !Node::PV && depth < 7 {
        let eval = board.evaluate();
        let margin = 150 * depth as i32;
        if eval >= beta + margin {
            return eval;
        }
    }

    //Null Move Pruning
    if !Node::PV && !in_check && !board.only_king_and_pawns() {
        let r = 4;
        board.make_null_move();
        let null_move_score = -search::<NonPV>(
            data,
            depth.saturating_sub(r),
            board,
            -beta,
            -(beta - 1),
            ply + 1,
        );
        board.unmake_move();
        if null_move_score >= beta {
            return null_move_score;
        }
    }

    let mut legal_moves = 0;
    let mut best_score = -INFINITY;
    let mut best_move: Option<Move> = None;
    let mut bound = Bound::Upper; //Fail-high means score is atleast this good so lower-bound/Fail-low means the score is an upper bound

    let mut move_picker = MovePicker::new(board, data);
    let mut quiets_searched = MoveList::new();

    while let Some(m) = move_picker.next(board, data, false) {
        //Late Move Pruning (LMP)
        if !in_check 
            && best_score.abs() < MATE_CUTOFF
            && m.get_kind().is_quiet()
            && legal_moves > 6 + 2 * depth * depth {
            continue;
        }

        if board.make_move(m).is_ok() {
            legal_moves += 1;
            let mut score = best_score;

            //PVS
            //Late Move Reductions (LMR)
            if depth > 3 && !Node::PV {
                //let reduction = (0.99 + f32::ln(depth as f32) * f32::ln(legal_moves as f32)) / PI; //https://www.chessprogramming.org/Late_Move_Reductions Obsidian formula
                let mut reduction = 0.7844 + f32::ln(depth as f32) * f32::ln(legal_moves as f32);
                if m.get_kind().is_quiet() {
                    reduction /= 2.4696;
                } else {
                    reduction /= 3.0;
                }
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
                //Add quiet moves to history
                if m.get_kind().is_quiet() {
                    let bonus = 300 * depth as i32 - 250;
                    let side = board.board_state.side_to_move;
                    data.history.update(side, m, bonus);
                    //Add malus to previously searched quiet moves
                    for e in quiets_searched.iter() {
                        let quiet_move = e.mv;
                        data.history.update(side, quiet_move, -bonus);
                    }
                }

                if let Some(m) = best_move {
                    let tt_score = best_score;
                    data.tt
                        .add_entry(m, tt_score, Bound::Lower, board.board_state.hash, depth);
                }
                return best_score;
            }

            //Add searched quiet moves to list
            if m.get_kind().is_quiet() {
                quiets_searched.push(m);
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
    ply: usize,
) -> i32 {
    data.add_nodes(1);
    let in_check = board.king_in_check();
    let mut best_score = if in_check {
        -MATE_SCORE + ply as i32
    } else {
        board.evaluate()
    };

    if best_score >= beta {
        return best_score;
    }

    if best_score > alpha {
        alpha = best_score;
    }

    let mut move_picker = MovePicker::new(board, data);
    let skip_quiets = !in_check;

    while let Some(m) = move_picker.next(board, data, skip_quiets) {
        if board.make_move(m).is_ok() {
            let score = -quiesce(data, board, -beta, -alpha, ply + 1);
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

    while let Some(m) = move_picker.next(board, data, false) {
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
