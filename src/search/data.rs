use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::board::Board;
use crate::search::time::{TimeManager, TimeSettings};
use crate::types::TranspositionTable;
use crate::types::{History, Move, MoveList, STARTING_FEN};

#[derive(Debug)]
pub enum Status {
    Stop,
    Running,
}

#[derive(Debug)]
pub struct SharedData {
    pub tt: TranspositionTable,
    pub total_nodes: AtomicUsize,
    pub status: Status,
}

impl SharedData {
    pub fn get_total_nodes_searched(&self) -> usize {
        self.total_nodes.load(Ordering::Acquire)
    }

    pub fn add_nodes(&self, nodes: usize) {
        self.total_nodes.fetch_add(nodes, Ordering::Relaxed);
    }

    pub fn clear_node_count(&self) {
        self.total_nodes.store(0, Ordering::Release);
    }
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            tt: TranspositionTable::default(),
            total_nodes: AtomicUsize::new(0),
            status: Status::Running,
        }
    }
}

#[derive(Debug)]
pub struct SearchData {
    pub shared: Arc<SharedData>,
    pub pv: Vec<MoveList>,
    pub board: Board,
    pub time: TimeManager,
    pub history: History,
}

impl SearchData {
    pub fn new(shared: SharedData) -> Self {
        SearchData {
            shared: Arc::new(shared),
            pv: vec![MoveList::new(); 128],
            board: Board::from_fen(STARTING_FEN),
            time: TimeManager::new(),
            history: History::new(),
        }
    }

    pub fn clear_history(&mut self) {
        self.history = History::new();
    }

    pub fn get_pv(&self) -> &MoveList {
        &self.pv[0]
    }

    pub fn get_best_move(&self) -> Move {
        self.get_pv().get(0).mv
    }

    pub fn nodes_per_second(&self) -> usize {
        (self.shared.get_total_nodes_searched() as f32 / self.time.elapsed().as_secs_f32()) as usize
    }

    pub fn start_time(&mut self) {
        self.time
            .set_time_limit(self.board.board_state.side_to_move);
        self.time.reset_clock();
    }

    pub fn add_pv_move(&mut self, m: Move, ply: usize) {
        self.pv[ply].clear();
        self.pv[ply].push(m);
        for child_m in self.pv[ply + 1].clone().iter() {
            self.pv[ply].push(child_m.mv);
        }
    }

    pub fn clear_pv(&mut self, ply: usize) {
        self.pv[ply].clear();
    }

    pub fn get_time_settings(&mut self) -> &mut TimeSettings {
        &mut self.time.settings
    }

    pub fn over_limit(&self) -> bool {
        self.time.over_limit()
    }

    pub fn reset_pv(&mut self) {
        self.pv = vec![MoveList::new(); 128];
    }
}

impl Default for SearchData {
    fn default() -> Self {
        Self::new(SharedData::default())
    }
}
