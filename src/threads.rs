use std::collections::HashMap;

use crate::{
    board::Board,
    search::{
        data::{SearchData, SharedData},
        search_runner,
        time::TimeManager,
    },
    types::MoveEntry,
};

pub struct ThreadPool {
    threads: Vec<SearchData>,
}

impl ThreadPool {
    pub fn new(
        board: Board,
        time: TimeManager,
        shared: std::sync::Arc<SharedData>,
        count: usize,
    ) -> Self {
        let mut threads = Vec::new();
        for _ in 0..count {
            threads.push(SearchData {
                board: board.clone(),
                time: time.clone(),
                shared: shared.clone(),
                ..Default::default()
            });
        }

        ThreadPool { threads }
    }

    pub fn start(&mut self, shared: std::sync::Arc<SharedData>, mute: bool) -> Option<MoveEntry> {
        shared.clear_node_count();
        shared.tt.increase_age();

        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (i, t) in self.threads.iter_mut().enumerate() {
                if i != 0 || mute {
                    t.mute();
                }

                handles.push(scope.spawn(|| search_runner(t)));
            }

            let mut best_moves = HashMap::new();

            for handle in handles {
                if let Ok(Some(m)) = handle.join() {
                    best_moves.insert(m, best_moves.get(&m).unwrap_or(&0) + 1);
                }
            }

            best_moves
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(m, _)| m)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        board::Board,
        search::{data::SharedData, time::TimeManager},
        threads::ThreadPool,
        types::STARTING_FEN,
    };

    #[test]
    fn test_multithread() {
        let a = std::thread::available_parallelism().unwrap().get();
        println!("{a}");

        let shared = Arc::new(SharedData::default());
        let mut time = TimeManager::new();
        time.settings.wtime = 8080;
        time.settings.winc = 80;

        let board = Board::from_fen(STARTING_FEN).unwrap();

        let mut pool = ThreadPool::new(board, time, shared.clone(), 3);
        let m = pool.start(shared.clone(), false).unwrap();
        println!("{} : {}", m.mv, m.score);
    }
}
