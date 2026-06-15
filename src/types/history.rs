use crate::types::{BitBoard, MAX_HISTORY, Move, Piece, Side, Square};

type FromToHistory<T> = [[T; 64]; 64];
type PieceToHistory<T> = [[T; 64]; 13];

#[derive(Debug, Clone)]
//[Side to Move][From Threatened][To Threatened][From][To]
pub struct QuietHistory(pub Box<[[[FromToHistory<i16>; 2]; 2]; 2]>);

impl QuietHistory {
    pub fn new() -> Self {
        Self(allocate_empty_history())
    }

    pub fn update(&mut self, threats: BitBoard, side: Side, m: Move, bonus: i32) {
        let from = m.get_from();
        let to = m.get_to();

        let from_threats = threats.contains(from);
        let to_threats = threats.contains(to);

        let entry = &mut self.0[side as usize][from_threats as usize][to_threats as usize]
            [from as usize][to as usize];
        update_entry(bonus, entry);
    }

    pub fn get(&self, threats: BitBoard, side: Side, m: Move) -> i32 {
        let from = m.get_from();
        let to = m.get_to();

        let from_threats = threats.contains(from);
        let to_threats = threats.contains(to);

        self.0[side as usize][from_threats as usize][to_threats as usize][from as usize]
            [to as usize] as i32
    }
}

#[derive(Debug, Clone)]
//[Piece][To][Captured Piece][To Threatened]
pub struct NoisyHistory(pub Box<PieceToHistory<[[i16; 2]; 7]>>);

impl NoisyHistory {
    pub fn new() -> Self {
        Self(allocate_empty_history())
    }

    pub fn update(
        &mut self,
        piece: Option<(Side, Piece)>,
        to: Square,
        captured: Option<Piece>,
        threats: BitBoard,
        bonus: i32,
    ) {
        let piece_index = match piece {
            Some((s, p)) => (s as usize * 6) + p as usize,
            None => 12,
        };

        let captured_index = match captured {
            Some(p) => p as usize,
            None => 6,
        };

        let entry =
            &mut self.0[piece_index][to as usize][captured_index][threats.contains(to) as usize];
        update_entry(bonus, entry);
    }

    pub fn get(
        &self,
        piece: Option<(Side, Piece)>,
        to: Square,
        captured: Option<Piece>,
        threats: BitBoard,
    ) -> i32 {
        let piece_index = match piece {
            Some((s, p)) => (s as usize * 6) + p as usize,
            None => 12,
        };

        let captured_index = match captured {
            Some(p) => p as usize,
            None => 6,
        };

        self.0[piece_index][to as usize][captured_index][threats.contains(to) as usize] as i32
    }
}

pub fn allocate_empty_history<T>() -> Box<T> {
    let layout = std::alloc::Layout::new::<T>();
    unsafe {
        let p = std::alloc::alloc_zeroed(layout);
        Box::<T>::from_raw(p.cast())
    }
}

pub fn update_entry(bonus: i32, entry: &mut i16) {
    let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
    *entry += (clamped_bonus - (*entry as i32) * clamped_bonus.abs() / MAX_HISTORY) as i16;
}

impl Default for QuietHistory {
    fn default() -> Self {
        QuietHistory::new()
    }
}

impl Default for NoisyHistory {
    fn default() -> Self {
        NoisyHistory::new()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::types::{BitBoard, NoisyHistory, Piece, Side, Square};

    #[test]
    fn test_history() {
        let mut noisy_history = NoisyHistory::new();

        let entry = noisy_history.get(None, Square::A4, None, BitBoard(0));
        println!("{}", entry);
        let piece = Some((Side::Black, Piece::Bishop));
        let captured = Some(Piece::Queen);
        noisy_history.update(piece, Square::A4, captured, BitBoard(0), 32);
        let entry2 = noisy_history.get(piece, Square::A4, captured, BitBoard(0));
        let entry = noisy_history.get(None, Square::A4, None, BitBoard(0));

        assert_eq!(entry2, 32);
        assert_eq!(entry, 0);
    }
}
