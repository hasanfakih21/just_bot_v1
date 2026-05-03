pub struct BitBoard {
    white_pawns: u64,
    white_knights: u64,
    white_bishops: u64,
    white_rooks: u64,
    white_queens: u64,
    white_king: u64,

    black_pawns: u64,
    black_knights: u64,
    black_bishops: u64,
    black_rooks: u64,
    black_queens: u64,
    black_king: u64,
}

//Little-Endian Rank-File Mapping
impl BitBoard {
    fn new() -> BitBoard {
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
