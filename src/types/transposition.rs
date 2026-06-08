use std::sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize, Ordering};

use crate::types::moves::Move;

const TT_DEFAULT_SIZE: usize = 16;
const MEGABYTE: usize = 1024 * 1024;
pub const SIZE_OF_ENTRY: usize = std::mem::size_of::<Entry>();

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bound {
    None,
    Exact,
    Upper,
    Lower,
}

#[derive(Debug, Clone)]
pub struct Entry {
    key: u64,        //8 bytes
    best_move: Move, //2 bytes
    depth: u8,       //1 byte
    score: i32,      //4 bytes
    bound: Bound,    //1 byte
                     //age
}

impl Entry {
    pub fn new(key: u64, best_move: Move, score: i32, bound: Bound, depth: u8) -> Self {
        Entry {
            key,
            best_move,
            score,
            bound,
            depth,
        }
    }

    pub fn get_key(&self) -> u64 {
        self.key
    }

    pub fn get_best_move(&self) -> Move {
        self.best_move
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_bound(&self) -> Bound {
        self.bound
    }

    pub fn get_depth(&self) -> u8 {
        self.depth
    }
}

//https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
const fn index(hash: u64, len: usize) -> usize {
    (((hash as u128) * (len as u128)) >> 64) as usize
}

#[derive(Debug)]
pub struct TranspositionTable {
    entries: AtomicPtr<Entry>,
    len: AtomicUsize,
    age: AtomicU8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let (len, p) = unsafe { allocate_entries(size_mb) };
        TranspositionTable {
            entries: AtomicPtr::new(p),
            len: AtomicUsize::new(len),
            age: AtomicU8::new(0),
        }
    }
    
    pub fn resize(&self, size_mb: usize) {
        unsafe { deallocate_entries(self.len(), self.ptr()) }
        let (new_len, new_p) = unsafe { allocate_entries(size_mb) };
        self.len.store(new_len, Ordering::Relaxed);
        self.entries.store(new_p, Ordering::Relaxed);
        self.age.store(0, Ordering::Relaxed);
    }

    fn ptr(&self) -> *mut Entry {
        self.entries.load(Ordering::Relaxed)
    }

    pub fn add_entry(&self, best_move: Move, score: i32, bound: Bound, hash: u64, depth: u8) {
        let entry = Entry::new(hash, best_move, score, bound, depth);
        let index = index(hash, self.len());
        debug_assert!(index < self.len());

        let old_entry = unsafe { &mut *self.ptr().add(index) };
        *old_entry = entry;
    }

    pub fn clear(&self) {
        self.age.store(0, Ordering::Relaxed);
        unsafe { self.ptr().write_bytes(0, self.len()) }
    }

    pub fn get_entry(&self, hash: u64) -> Option<&Entry> {
        let index = index(hash, self.len());
        debug_assert!(index < self.len());

        let entry = unsafe { &*self.ptr().add(index) };
        if entry.get_key() == hash {
            Some(entry)
        } else {
            None
        }
    }

    pub fn hashfull(&self) -> usize {
        let mut count = 0;
        let entries = unsafe { std::slice::from_raw_parts(self.ptr(), self.len()) };

        for e in entries.iter().take(1000) {
            if e.bound != Bound::None {
                count += 1;
            }
        }

        count
    }

    pub fn increase_age(&self) {
        self.age
            .update(Ordering::Relaxed, Ordering::Relaxed, |e| e + 1);
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(TT_DEFAULT_SIZE)
    }
}

impl Drop for TranspositionTable {
    fn drop(&mut self) {
        unsafe { deallocate_entries(self.len(), self.ptr()) };
    }
}

unsafe fn allocate_entries(size_mb: usize) -> (usize, *mut Entry) {
    let size = size_mb * MEGABYTE;
    let num_entries = size / SIZE_OF_ENTRY;

    let layout = std::alloc::Layout::from_size_align(size, align_of::<Entry>()).unwrap();
    let p = unsafe { std::alloc::alloc_zeroed(layout) };

    (num_entries, p.cast())
}

unsafe fn deallocate_entries(len: usize, p: *mut Entry) {
    let size = SIZE_OF_ENTRY * len;
    let layout = std::alloc::Layout::from_size_align(size, align_of::<Entry>()).unwrap();

    unsafe { std::alloc::dealloc(p.cast(), layout) };
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        search::{Root, data::SearchData, search},
        types::INFINITY,
    };

    #[test]
    fn test_transposition_table() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ")
                .unwrap();
        let mut data = SearchData {
            board,
            ..Default::default()
        };

        let score = search::<Root>(&mut data, 3, -INFINITY, INFINITY, 0);

        let hash = data.board.board_state.hash;
        let entry = data.shared.tt.get_entry(hash).unwrap();

        let m = entry.get_best_move();
        let s = entry.get_score();

        let best_move = data.get_pv().get(0).mv;

        assert_eq!(best_move, m);
        assert_eq!(score, s);

        let _ = data.board.make_move(best_move);
        search::<Root>(&mut data, 2, -INFINITY, INFINITY, 0);

        let entry = data.shared.tt.get_entry(hash).unwrap();

        let m = entry.get_best_move();
        let s = entry.get_score();

        assert_eq!(best_move, m);
        assert_eq!(score, s);
    }
}
