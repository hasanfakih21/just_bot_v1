use std::time::Instant;

use crate::{board::Board, search::data::SearchData, tools::uci::go, types::STARTING_FEN};

pub fn bench() {
    let positions = [
        "3N4/2b4Q/3k2P1/5pR1/2BP3B/4K3/p2pN3/4q1r1 w - - 0 1",
        "8/1P1n4/3k4/N7/1nR4P/1QKpb3/1PPP1B2/5q2 w - - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
        "K7/p1p1P1N1/P5rk/2p5/1P3Np1/5P2/P1q4B/8 w - - 0 1",
        "3B4/1r2p3/r2p1p2/bkp1P1p1/1p1P1PPp/p1P4P/PP1K4/3B4 w - - 0 1",
        "rk6/pP1p2p1/B7/3K1P2/8/8/7b/8 w - - 0 1",
        "8/1p1pNpbk/1q1P4/pP2p2K/P3N3/4P1P1/3P4/8 w - - 0 1",
        STARTING_FEN,
        "3n4/n4Rp1/3N2K1/2pkp3/1BbpR1Pp/4P1P1/r1Bb2p1/2N5 w - - 0 1",
        "4n3/P7/6PB/5k2/8/7P/5K2/8 w - - 0 1",
    ];

    let time = Instant::now();
    let mut total_node_count = 0;
    for fen in positions {
        let mut data = SearchData {
            board: Board::from_fen(fen).unwrap(),
            ..Default::default()
        };
        data.shared.mute();
        go("depth 8", &mut data);
        total_node_count += data.shared.get_total_nodes_searched();
    }

    let nps = (total_node_count as f32 / time.elapsed().as_secs_f32()) as usize;
    println!("{} nodes {} nps", total_node_count, nps);
}
