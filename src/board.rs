#[derive(Default)]
pub struct BitBoard {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_queens: u64,
    pub white_king: u64,

    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_queens: u64,
    pub black_king: u64,
}

//Little-Endian Rank-File Mapping
impl BitBoard {
    pub fn new() -> Self {
        BitBoard {
            white_pawns:   0x0000_0000_0000_FF00,
            white_knights: 0x0000_0000_0000_0042,
            white_bishops: 0x0000_0000_0000_0024,
            white_rooks:   0x0000_0000_0000_0081,
            white_queens:  0x0000_0000_0000_0008,
            white_king:    0x0000_0000_0000_0010,

            black_pawns:   0x00FF_0000_0000_0000,
            black_knights: 0x4200_0000_0000_0000,
            black_bishops: 0x2400_0000_0000_0000,
            black_rooks:   0x8100_0000_0000_0000,
            black_queens:  0x0800_0000_0000_0000,
            black_king:    0x1000_0000_0000_0000,
        }
    }
}

pub fn print_board(bit_board: &u64) {
    println!();

    for rank in (0..8).rev() {
        for file in 0..8 {
            let board_index = (rank * 8) + file; 
            let bit_state = bit_board & (1u64 << board_index);

            print!("{} ", if bit_state != 0 {1} else {0});
        }

        println!();
    }
}