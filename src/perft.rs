use crate::board::{Board, Square};

pub fn perft(depth: usize, board: &mut Board, nodes_count: &mut usize) {
    for m in board.generate_all_moves().iter() {
        if board.make_move(*m).is_ok() {  
            if m.get_from() == Square::D1 && m.get_to() == Square::A1 {println!("{board}")}
            let divided_nodes = perft_divide(depth - 1, board);
            println!("{m}: {divided_nodes}");
            *nodes_count += divided_nodes;
            board.unmake_move();
        } 
    }
}

pub fn perft_divide(depth: usize, board: &mut Board) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;

    for m in board.generate_all_moves().iter() {
        if board.make_move(*m).is_ok() {
            nodes += perft_divide(depth - 1, board);
            board.unmake_move();
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::{board::{Board, constants::STARTING_FEN}, perft::perft};

    #[test]
    fn test_perft() {
        let mut board = Board::from_fen(STARTING_FEN);
        println!("{board}");
        let mut nodes_count = 0;
        let mut clock = Instant::now();
        perft(5, &mut board, &mut nodes_count);
        println!("Number of nodes: {nodes_count}\nTime: {}ms", clock.elapsed().as_millis()); 
        assert_eq!(nodes_count, 4865609);

        let mut kiwipete = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        println!("{kiwipete}");
        nodes_count = 0;
        clock = Instant::now();
        perft(4, &mut kiwipete, &mut nodes_count);
        println!("Number of nodes: {nodes_count}\nTime: {}ms", clock.elapsed().as_millis()); 
        assert_eq!(nodes_count, 4085603);
                 
        let mut board3 = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ");
        println!("{board3}");
        nodes_count = 0;
        clock = Instant::now();
        perft(5, &mut board3, &mut nodes_count);
        println!("Number of nodes: {nodes_count}\nTime: {}ms", clock.elapsed().as_millis()); 
        assert_eq!(nodes_count, 674624);

        let mut board4 = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nPP5/BB2P3/q4N2/P2P2PP/r2Q1RK1 w kq - 0 2");
        println!("{board4}");
        nodes_count = 0;
        clock = Instant::now();
        perft(2, &mut board4, &mut nodes_count);
        println!("Number of nodes: {nodes_count}\nTime: {}ms", clock.elapsed().as_millis()); 
        assert_eq!(nodes_count, 1344);
    }
}