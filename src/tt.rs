use std::cell::UnsafeCell;

use crate::movegen::{Move, MoveFlags};

pub const TABLE_SIZE: i32 = 22;

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
    pub best_move: u16,
}

impl TTEntry {
    pub fn new() -> Self {
        TTEntry {
            hash: 0,
            score: 0,
            depth: 0,
            flag: 0,
            best_move: 0,
        }
    }
}

pub struct TranspositionTable {
    pub vault: Vec<UnsafeCell<TTEntry>>,
}

unsafe impl Send for TranspositionTable {}

unsafe impl Sync for TranspositionTable {}

pub const SHIFT: usize = (1 << TABLE_SIZE) - 1;

impl TranspositionTable {
    pub fn new() -> Self {
        let vault = (0..1 << TABLE_SIZE)
            .map(|_| UnsafeCell::new(TTEntry::new()))
            .collect();
        TranspositionTable { vault }
    }

    pub fn store(&self, hash: u64, score: i32, depth: u8, flag: u8, best_move: u16) {
        let existing = unsafe { *self.vault[hash as usize & SHIFT].get() };
        if existing.hash != hash || depth >= existing.depth {
            unsafe {
                *self.vault[hash as usize & SHIFT].get() = TTEntry {
                    hash,
                    score,
                    depth,
                    flag,
                    best_move,
                };
            }
        }
    }

    pub fn lookup(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let entry = unsafe { *self.vault[hash as usize & SHIFT].get() };
        if entry.hash == hash && entry.depth >= depth {
            match entry.flag {
                0 => Some(entry.score),
                1 => {
                    if entry.score <= alpha {
                        Some(alpha)
                    } else {
                        None
                    }
                }
                2 => {
                    if entry.score >= beta {
                        Some(beta)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_best_move(&self, hash: u64) -> Option<u16> {
        let entry = unsafe { *self.vault[hash as usize & SHIFT].get() };
        if entry.hash == hash && entry.best_move != 0 {
            return Some(entry.best_move);
        }
        None
    }

    pub fn stats(&self) {
        let filled = self
            .vault
            .iter()
            .filter(|entry| unsafe { (*entry.get()).hash != 0 })
            .count();
        eprintln!(
            "TT filled: {}/{} ({:.1}%)",
            filled,
            1 << TABLE_SIZE,
            filled as f64 / (1 << TABLE_SIZE) as f64 * 100.0
        );
    }

    pub fn store_with_bound(
        &self,
        hash: u64,
        alpha: i32,
        beta: i32,
        original_alpha: i32,
        depth: u8,
        best_move: u16,
    ) {
        if alpha >= beta {
            self.store(hash, alpha, depth, 2, best_move);
        } else if alpha > original_alpha {
            self.store(hash, alpha, depth, 0, best_move);
        } else {
            self.store(hash, alpha, depth, 1, best_move);
        }
    }
}
