use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::board::Board;
use crate::nnue::{Accumulator, NNUE};
use crate::search::time::{TimeManager, TimeSettings};
use crate::types::{
    KING_SIDE_ROOK_BLACK, KING_SIDE_ROOK_WHITE, Move, MoveKind, MoveList, NoisyHistory, Piece,
    QUEEN_SIDE_ROOK_BLACK, QUEEN_SIDE_ROOK_WHITE, STARTING_FEN, Side, Square, to_file_bb,
};
use crate::types::{QuietHistory, TranspositionTable};

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

    pub quiet_history: QuietHistory,
    pub noisy_history: NoisyHistory,

    pub white_features: Accumulator,
    pub black_features: Accumulator,
}

impl SearchData {
    pub fn new(shared: SharedData) -> Self {
        SearchData {
            shared: Arc::new(shared),
            pv: vec![MoveList::new(); 128],
            board: Board::from_fen(STARTING_FEN).unwrap(),
            time: TimeManager::new(),
            quiet_history: QuietHistory::new(),
            noisy_history: NoisyHistory::new(),

            white_features: Accumulator::new(&NNUE),
            black_features: Accumulator::new(&NNUE),
        }
    }

    pub fn clear_histories(&mut self) {
        self.quiet_history = QuietHistory::new();
        self.noisy_history = NoisyHistory::new();
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
        if let Some(node_limt) = self.time.node_limit()
            && self.shared.get_total_nodes_searched() >= node_limt
        {
            return true;
        }

        self.time.over_limit()
    }

    pub fn reset_pv(&mut self) {
        self.pv = vec![MoveList::new(); 128];
    }

    //Called before move is made on the board
    pub fn make_move(&mut self, m: Move) {
        let from = m.get_from();
        let to = m.get_to();
        let kind = m.get_kind();
        let stm = self.board.state.side_to_move;

        //Need to toggle off extra captured piece in case of capture
        if kind.is_capture() {
            let capture_square = m.get_capture_square();
            let (captured_side, captured_piece) =
                self.board.get_piece_at_square(capture_square).unwrap();

            self.white_features.toggle_off(
                captured_side == Side::White,
                captured_piece,
                capture_square,
            );
            self.black_features.toggle_off(
                captured_side == Side::Black,
                captured_piece,
                capture_square,
            );
        } 

        //Need to toggle rook in case of castling
        if kind == MoveKind::KingCastle {
            debug_assert!(!(from.to_bb() & to_file_bb(Square::E4)).is_empty());
            let king_rook_square = match stm {
                Side::White => KING_SIDE_ROOK_WHITE,
                Side::Black => KING_SIDE_ROOK_BLACK,
            };

            self.white_features
                .toggle_off(stm == Side::White, Piece::Rook, king_rook_square);
            self.black_features
                .toggle_off(stm == Side::Black, Piece::Rook, king_rook_square);

            self.white_features
                .toggle_on(stm == Side::White, Piece::Rook, from.shift(1).unwrap());
            self.white_features
                .toggle_on(stm == Side::Black, Piece::Rook, from.shift(1).unwrap());
        }

        //Need to toggle rook in case of castling
        if kind == MoveKind::QueenCastle {
            debug_assert!(!(from.to_bb() & to_file_bb(Square::E4)).is_empty());
            let queen_rook_square = match stm {
                Side::White => QUEEN_SIDE_ROOK_WHITE,
                Side::Black => QUEEN_SIDE_ROOK_BLACK,
            };

            self.white_features
                .toggle_off(stm == Side::White, Piece::Rook, queen_rook_square);
            self.black_features
                .toggle_off(stm == Side::Black, Piece::Rook, queen_rook_square);

            self.white_features
                .toggle_on(stm == Side::White, Piece::Rook, from.shift(-1).unwrap());
            self.white_features
                .toggle_on(stm == Side::Black, Piece::Rook, from.shift(-1).unwrap());
        }

        let moving_piece = self.board.get_piece_at_square(from).unwrap().1;
        //Need to handle promotions
        if kind.is_promotion() {
            let promotion_piece = m.get_promoted_piece().unwrap();
            self.white_features
                .toggle_off(stm == Side::White, moving_piece, from);
            self.white_features
                .toggle_off(stm == Side::Black, moving_piece, from);

            self.white_features
                .toggle_on(stm == Side::White, promotion_piece, to);
            self.white_features
                .toggle_on(stm == Side::Black, promotion_piece, to);
        } else {
            self.white_features
                .toggle_off(stm == Side::White, moving_piece, from);
            self.white_features
                .toggle_off(stm == Side::Black, moving_piece, from);

            self.white_features
                .toggle_on(stm == Side::White, moving_piece, to);
            self.white_features
                .toggle_on(stm == Side::Black, moving_piece, to);
        }
    }

    //Called after move is already unmade on the board
    pub fn unmake_move(&mut self, m: Move) {
        let from = m.get_from();
        let to = m.get_to();
        let kind = m.get_kind();
        let stm = self.board.state.side_to_move;

        //Need to toggle off extra captured piece in case of capture
        if kind.is_capture() {
            let capture_square = m.get_capture_square();
            let (captured_side, captured_piece) =
                self.board.get_piece_at_square(capture_square).unwrap();

            self.white_features.toggle_on(
                captured_side == Side::White,
                captured_piece,
                capture_square,
            );
            self.black_features.toggle_on(
                captured_side == Side::Black,
                captured_piece,
                capture_square,
            );
        } 

        //Need to toggle rook in case of castling
        if kind == MoveKind::KingCastle {
            debug_assert!(!(from.to_bb() & to_file_bb(Square::E4)).is_empty());
            let king_rook_square = match stm {
                Side::White => KING_SIDE_ROOK_WHITE,
                Side::Black => KING_SIDE_ROOK_BLACK,
            };

            self.white_features
                .toggle_on(stm == Side::White, Piece::Rook, king_rook_square);
            self.black_features
                .toggle_on(stm == Side::Black, Piece::Rook, king_rook_square);

            self.white_features
                .toggle_off(stm == Side::White, Piece::Rook, from.shift(1).unwrap());
            self.white_features
                .toggle_off(stm == Side::Black, Piece::Rook, from.shift(1).unwrap());
        }

        //Need to toggle rook in case of castling
        if kind == MoveKind::QueenCastle {
            debug_assert!(!(from.to_bb() & to_file_bb(Square::E4)).is_empty());
            let queen_rook_square = match stm {
                Side::White => QUEEN_SIDE_ROOK_WHITE,
                Side::Black => QUEEN_SIDE_ROOK_BLACK,
            };

            self.white_features
                .toggle_on(stm == Side::White, Piece::Rook, queen_rook_square);
            self.black_features
                .toggle_on(stm == Side::Black, Piece::Rook, queen_rook_square);

            self.white_features
                .toggle_off(stm == Side::White, Piece::Rook, from.shift(-1).unwrap());
            self.white_features
                .toggle_off(stm == Side::Black, Piece::Rook, from.shift(-1).unwrap());
        }

        let moving_piece = self.board.get_piece_at_square(from).unwrap().1;
        //Need to handle promotions
        if kind.is_promotion() {
            let promotion_piece = m.get_promoted_piece().unwrap();
            self.white_features
                .toggle_on(stm == Side::White, moving_piece, from);
            self.white_features
                .toggle_on(stm == Side::Black, moving_piece, from);

            self.white_features
                .toggle_off(stm == Side::White, promotion_piece, to);
            self.white_features
                .toggle_off(stm == Side::Black, promotion_piece, to);
        } else {
            self.white_features
                .toggle_on(stm == Side::White, moving_piece, from);
            self.white_features
                .toggle_on(stm == Side::Black, moving_piece, from);

            self.white_features
                .toggle_off(stm == Side::White, moving_piece, to);
            self.white_features
                .toggle_off(stm == Side::Black, moving_piece, to);
        }
    }

    pub fn nnue_evaluate(&self) -> i32 {
        let stm = self.board.state.side_to_move;

        let (us, them) = match stm {
            Side::White => (&self.white_features, &self.black_features),
            Side::Black => (&self.black_features, &self.white_features), 
        };

        NNUE.evaluate(us, them)
    }
}

impl Default for SearchData {
    fn default() -> Self {
        Self::new(SharedData::default())
    }
}
