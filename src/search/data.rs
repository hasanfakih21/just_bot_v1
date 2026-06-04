use std::sync::atomic::{AtomicUsize, Ordering};

use crate::search::time::{TimeManager, TimeSettings};
use crate::types::{History, Move, MoveList, Side};

use crate::types::TranspositionTable;

#[derive(Debug)]
pub struct SearchData {
    playing_as: Side,
    depth: usize,
    pv: Vec<MoveList>,
    total_nodes: AtomicUsize,

    pub tt: TranspositionTable,
    pub time: TimeManager,
    pub history: History,
}

#[derive(Debug)]
pub struct SearchCancelled;

impl SearchData {
    pub fn new() -> Self {
        SearchData {
            playing_as: Side::White,
            depth: 0,
            pv: vec![MoveList::new(); 128],
            tt: TranspositionTable::new(),
            time: TimeManager::new(),
            total_nodes: AtomicUsize::new(0),
            history: History::new(),
        }
    }

    pub fn add_to_history(&mut self, m: Move, side: Side, depth: usize) {
        self.history.0[side as usize].0[m.get_from() as usize][m.get_to() as usize] += depth as i32 * depth as i32;
    }

    pub fn get_history(&self, m: Move, side: Side) -> i32 {
        self.history.0[side as usize].0[m.get_from() as usize][m.get_to() as usize]
    }

    pub fn clear_history(&mut self) {
        self.history = History::new();
    }

    pub fn get_searched_depth(&self) -> usize {
        self.depth
    }

    pub fn get_total_nodes_searched(&self) -> usize {
        self.total_nodes.load(Ordering::Acquire)
    }

    pub fn get_pv(&self) -> &MoveList {
        &self.pv[0]
    }

    pub fn add_nodes(&self, nodes: usize) {
        self.total_nodes.fetch_add(nodes, Ordering::Relaxed);
    }

    pub fn increase_depth(&mut self) {
        self.depth += 1;
    }

    pub fn nodes_per_second(&self) -> usize {
        (self.get_total_nodes_searched() as f32 / self.time.elapsed().as_secs_f32()) as usize
    }

    pub fn start_time(&mut self) {
        self.time.reset_clock(self.playing_as);
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

    pub fn clear_table(&mut self) {
        self.tt.clear();
    }

    pub fn get_time_settings(&mut self) -> &mut TimeSettings {
        &mut self.time.settings
    }

    pub fn over_limit(&self) -> bool {
        self.time.over_limit()
    }

    pub fn set_playing_as(&mut self, side: Side) {
        self.playing_as = side;
    }

    pub fn clear_node_count(&self) {
        self.total_nodes.store(0, Ordering::Release);
    }

    pub fn reset_pv(&mut self) {
        self.pv = vec![MoveList::new(); 128];
    }
}

impl Default for SearchData {
    fn default() -> Self {
        Self::new()
    }
}
