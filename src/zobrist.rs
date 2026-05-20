use std::{array, sync::Mutex};

use crate::{board::{CastlingRights, Piece, Side, Square}, magics::get_random_u64_num};


#[derive(Debug)]
pub struct Zobrist {
    pieces: [[u64; 64]; 12],
    side: u64,
    castling: [u64; 16],
    enpassant: [u64; 8],
}

impl Zobrist {
    pub fn get_piece_num(&self, side: Side, piece: Piece, square: Square) -> u64 {
        self.pieces[(piece as usize) + (side as usize * 6)][square as usize]
    }

    pub fn get_side_num(&self) -> u64 {
        self.side
    }

    pub fn get_castling_num(&self, rights: CastlingRights) -> u64 {
        self.castling[rights.0 as usize]
    }

    pub fn get_enpassant_num(&self, square: Square) -> u64 {
        self.enpassant[square as usize % 8]
    }
}

pub static ZOBRIST: Mutex<Zobrist> = {
    Mutex::new(Zobrist {
        pieces: [[0; 64]; 12], //Number for each piece on each square 12 pieces 64 squares 
        side: 0, //Number to indicate the side to move is black
        castling: [0; 16], //Castling rights 2^4 aka all possible castling combinations.
        enpassant: [0; 8], //File of valid en-passant square
    })
};

pub fn init_zobrist_nums() {
    let mut zobrist = ZOBRIST.lock().unwrap();
    zobrist.pieces.iter_mut().for_each(|e| *e = array::from_fn(|_| get_random_u64_num()));
    zobrist.side = get_random_u64_num();
    zobrist.castling = array::from_fn(|_| get_random_u64_num());
    zobrist.enpassant = array::from_fn(|_| get_random_u64_num());
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn test_zobrist() {
        init_zobrist_nums();
        let zobrist = ZOBRIST.lock().unwrap();
        println!("{:?}", zobrist);

        //want to check if every number is unique
        let mut seen = HashSet::new(); 
        assert!(zobrist.pieces
                .iter().flatten()
                .chain(zobrist.castling.iter())
                .chain(zobrist.enpassant.iter())
                .chain([zobrist.side].iter())
                .all(|e| seen.insert(e))); 
    }
}