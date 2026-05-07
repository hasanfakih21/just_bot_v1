pub use super::board;
use crate::{Side, Square};
use crate::board::constants::*;

pub fn mask_pawn_attacks(side: Side, square: Square) -> u64 {
    let current = 1u64 << square as u64;
    let mut top_left = 0u64; 
    let mut top_right = 0u64;

    match side {
        Side::White => {
            if current & A_FILE == 0 {
                top_left = current << 7;
            }

            if current & H_FILE == 0 {
                top_right = current << 9;
            } 
        },
        Side::Black => {
            if current & H_FILE == 0 {
                top_left = current >> 7;
            }

            if current & A_FILE == 0 {
                top_right = current >> 9;
            }
        }
    }

    top_left | top_right
}

pub fn mask_knight_attacks(square: Square) -> u64 {
    let current = 1u64 << square as u64;

    let tl1 = if (current & A_FILE) == 0 {current << 15} else {0};
    let tl2 = if (current & (A_FILE | B_FILE)) == 0 {current << 6} else {0};

    let bl1 = if (current & (A_FILE | B_FILE)) == 0 {current >> 10} else {0};
    let bl2 = if (current & A_FILE) == 0 {current >> 17} else {0};

    let tr1 = if (current & H_FILE) == 0 {current << 17} else {0};
    let tr2 = if (current & (H_FILE | G_FILE)) == 0 {current << 10} else {0};

    let br1 = if (current & (H_FILE | G_FILE)) == 0 {current >> 6} else {0};
    let br2 = if (current & H_FILE) == 0 {current >> 15} else {0};

    tl1 | tl2 | bl1 | bl2 | tr1 | tr2 | br1 | br2
}

pub fn mask_king_attacks(square: Square) -> u64 {
    let current = 1u64 << square as u64;
    let n = current << 8;
    let nw = if current & A_FILE == 0 {current << 7} else {0};
    let w = if current & A_FILE == 0 {current >> 1} else {0};
    let sw = if current & A_FILE == 0 {current >> 9} else {0};
    let s = current >> 8;
    let se = if current & H_FILE == 0 {current >> 7} else {0};
    let e = if current & H_FILE == 0 {current << 1} else {0};
    let ne = if current & H_FILE == 0 {current << 9} else {0};

    n | nw | w | sw | s | se | e | ne
}

pub fn mask_bishop_attacks(square: Square) -> u64 {
    let mut attacks = 0u64;
    let (rank, file) = square.to_rank_and_file();

    let (mut r, mut f) = (rank, file);
    while r < 6 && f < 6 {
        r += 1;
        f += 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, f).unwrap() as u64;
    }

    let (mut r, mut f) = (rank, file);
    while r > 1 && f < 6 {
        r -= 1;
        f += 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, f).unwrap() as u64;
    }

    let (mut r, mut f) = (rank, file);
    while r < 6 && f > 1 {
        r += 1;
        f -= 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, f).unwrap() as u64;
    }

    let (mut r, mut f) = (rank, file);
    while r > 1 && f > 1 {
        r -= 1;
        f -= 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, f).unwrap() as u64;
    }
    
    attacks
}

pub fn mask_rook_attacks(square: Square) -> u64 {
    let mut attacks = 0u64;
    let (rank, file) = square.to_rank_and_file();
    
    let mut r = rank;
    while r > 1 {
        r -= 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, file).unwrap() as u64;
    }

    let mut r = rank;
    while r < 6 {
        r += 1;
        attacks |= 1u64 << Square::from_rank_and_file(r, file).unwrap() as u64;
    }

    let mut f = file;
    while f < 6 {
        f += 1;
        attacks |= 1u64 << Square::from_rank_and_file(rank, f).unwrap() as u64;
    }

    let mut f = file;
    while f > 1 {
        f -= 1;
        attacks |= 1u64 << Square::from_rank_and_file(rank, f).unwrap() as u64;
    }      

    attacks  
}