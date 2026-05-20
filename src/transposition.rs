use crate::board::moves::Move;

const TT_SIZE: usize = 64;
const MEGABYTE: usize = 1024 * 1024;
const ENTRIES: usize = TT_SIZE * MEGABYTE / std::mem::size_of::<Entry>();

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    PV,
    All,
    Cut,
}

#[derive(Debug, Clone)]
pub struct Entry {
    key: u64,
    best_move: Move,
    //depth: u8,
    score: i32,
    node: NodeType,
    //age
}

impl Entry {
    pub fn new(key: u64, best_move: Move, score: i32, node: NodeType) -> Self {
        Entry { key, best_move, score, node}
    }

    pub fn get_key(&self) -> u64 {
        self.key
    }

    pub fn get_best_move(&self) -> Move {
        self.best_move
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }
    
    pub fn get_node_type(&self) -> NodeType {
        self.node
    }
}

const fn index(hash: u64) -> usize {
    (((hash as u128) * (ENTRIES as u128)) >> 64) as usize
}

#[derive(Debug, Clone)]
pub struct TranspositionTable(Vec<Option<Entry>>);

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable(vec![None; ENTRIES])
    }

    pub fn add_entry(&mut self, entry: Entry, hash: u64) {
        self.0[index(hash)] = Some(entry);
    }

    pub fn get_entry(&self, hash: u64) -> &Option<Entry> {
        &self.0[index(hash)]
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}