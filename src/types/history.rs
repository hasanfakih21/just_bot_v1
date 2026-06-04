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
