use crate::types::{BitBoard, MAX_HISTORY, Move, Side};

#[derive(Debug, Clone, Copy)]
pub struct FromToHistory(pub [[i32; 64]; 64]);

impl FromToHistory {
    pub fn new() -> Self {
        Self([[0; 64]; 64])
    }
}

#[derive(Debug, Clone)]
//[Side to Move][From_Threatened][To_Threatened][From][To]
pub struct QuietHistory(pub Box<[[[FromToHistory; 2]; 2]; 2]>);

impl QuietHistory {
    pub fn new() -> Self {
        QuietHistory(Box::new([[[FromToHistory::new(); 2]; 2]; 2]))
    }

    pub fn update(&mut self, threats: BitBoard, side: Side, m: Move, bonus: i32) {
        let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        let from = m.get_from();
        let to = m.get_to();

        let from_threats = threats.contains(from);
        let to_threats = threats.contains(to);

        self.0[side as usize][from_threats as usize][to_threats as usize].0[from as usize][to as usize] += clamped_bonus
            - self.0[side as usize][from_threats as usize][to_threats as usize].0[from as usize][to as usize]
                * clamped_bonus.abs()
                / MAX_HISTORY;
    }

    pub fn get(&self, threats: BitBoard, side: Side, m: Move) -> i32 {
        let from = m.get_from();
        let to = m.get_to();

        let from_threats = threats.contains(from);
        let to_threats = threats.contains(to);

        self.0[side as usize][from_threats as usize][to_threats as usize].0[from as usize][to as usize]
    }
}

impl Default for FromToHistory {
    fn default() -> Self {
        FromToHistory::new()
    }
}

impl Default for QuietHistory {
    fn default() -> Self {
        QuietHistory::new()
    }
}
