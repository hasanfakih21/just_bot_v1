use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::board::Board;
use crate::search::time::{TimeManager, TimeSettings};
use crate::types::TranspositionTable;
use crate::types::{History, Move, MoveList, STARTING_FEN};

#[derive(Debug)]
pub struct Status(AtomicBool);

impl Status {
    pub const RUNNING: bool = true;
    pub const STOPPED: bool = false;

    pub fn stop(&self) {
        self.0.store(Self::STOPPED, Ordering::Relaxed);
    }

    pub fn run(&self) {
        self.0.store(Self::RUNNING, Ordering::Relaxed);
    }

    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub struct SharedData {
    pub tt: TranspositionTable,
    pub total_nodes: AtomicUsize,
    pub status: Status,
    pub mute: AtomicBool,
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

    pub fn report(&self) -> bool {
        !self.mute.load(Ordering::Relaxed)
    }

    pub fn mute(&self) {
        self.mute.store(true, Ordering::Relaxed);
    }
}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            tt: TranspositionTable::default(),
            total_nodes: AtomicUsize::new(0),
            status: Status(AtomicBool::new(Status::RUNNING)),
            mute: AtomicBool::new(false),
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
            board: Board::from_fen(STARTING_FEN).unwrap(),
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
        self.time.set_time_limit(self.board.state.side_to_move);
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
