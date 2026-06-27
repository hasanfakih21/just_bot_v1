use std::ops::{Index, IndexMut};

use crate::types::{MAX_PLY, Move, Piece, Side};

#[derive(Debug)]
pub struct PlyTable {
    data: [PlyData; MAX_PLY as usize + 1],
}

impl PlyTable {
    pub fn new() -> Self {
        PlyTable {
            data: [PlyData::default(); MAX_PLY as usize + 1],
        }
    }
}

impl Default for PlyTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlyData {
    pub m: Move,
    pub in_check: bool,
    pub piece: Option<(Side, Piece)>,
}

impl Index<usize> for PlyTable {
    type Output = PlyData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for PlyTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
