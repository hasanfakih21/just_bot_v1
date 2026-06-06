use crate::types::{MAX_HISTORY, Move, Side};

#[derive(Debug, Clone)]
pub struct FromToHistory(pub [[i32; 64]; 64]);

impl FromToHistory {
    pub fn new() -> Self {
        Self([[0; 64]; 64])
    }
}

#[derive(Debug, Clone)]
//[Side to Move][From][To]
pub struct History(pub Box<[FromToHistory; 2]>);

impl History {
    pub fn new() -> Self {
        History(Box::new([FromToHistory::new(), FromToHistory::new()]))
    }

    pub fn update(&mut self, side: Side, m: Move, bonus: i32) {
        let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        self.0[side as usize].0[m.get_from() as usize][m.get_to() as usize] += clamped_bonus
            - self.0[side as usize].0[m.get_from() as usize][m.get_to() as usize]
                * clamped_bonus.abs()
                / MAX_HISTORY;
    }

    pub fn get(&self, side: Side, m: Move) -> i32 {
        self.0[side as usize].0[m.get_from() as usize][m.get_to() as usize]
    }
}

impl Default for FromToHistory {
    fn default() -> Self {
        FromToHistory::new()
    }
}

impl Default for History {
    fn default() -> Self {
        History::new()
    }
}
