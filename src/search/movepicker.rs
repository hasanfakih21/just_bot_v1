use std::cmp::Reverse;

use crate::{board::{Board, movegen::MoveGenKind}, search::data::SearchData, types::{Move, MoveKind, MoveList, Square}};

pub fn order_moves(board: &mut Board, data: &mut SearchData) -> MoveList {
    //We want to sort the moves based on most valuable victim / least valuable attacker
    //Sort captures based on (attacked piece value - attacking piece value)
    let mut full_list = MoveList::new();
    let mut captures: MoveList = board.generate_moves(MoveGenKind::Captures);
    let mut best_move: Option<Move> = None;

    //Want to add the best move from the transposition table if it exists to the beginning of the list
    if let Some(e) = data.tt.get_entry(board.board_state.hash)
        && board.board_state.hash == e.get_key()
    {
        let bm = e.get_best_move();
        full_list.push(bm);
        best_move = Some(bm);
    }

    captures.iter_mut().into_slice().sort_by_key(|m| {
        let attacker = board.get_piece_at_square(m.get_from()).unwrap().1;
        let victim = match m.get_kind() {
            MoveKind::EnPassant => {
                board
                    .get_piece_at_square(Square::from(m.get_to() as usize ^ 8))
                    .unwrap()
                    .1
            }
            _ => board.get_piece_at_square(m.get_to()).unwrap().1,
        };
        Reverse(victim.value() - attacker.value())
    });

    let quiet = board.generate_moves(MoveGenKind::Quiet);
    let pawn_promos = board.generate_moves(MoveGenKind::NonCapturePromotions);

    for m in captures
        .iter()
        .chain(pawn_promos.iter())
        .chain(quiet.iter())
    {
        if let Some(bm) = best_move
            && *m == bm
        {
            continue;
        }
        full_list.push(*m);
    }

    full_list
}