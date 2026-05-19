use crate::{board::moves::Move, zobrist::Zobrist};

pub enum Node {
    PV,
    All,
    Cut,
}

pub struct Info {
    key: u64,
    best_move: Move,
    //depth: u8,
    score: i32,
    node: Node,
    //age
}

pub struct TranspositionTable(Vec<Info>);

