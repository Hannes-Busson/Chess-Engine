#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
    pub best_move: u16,
}

pub struct TransponationTable {
    pub vault: Vec<TTEntry>,
}

pub const SHIFT: usize = (1 << 22) - 1;

impl TransponationTable {
    pub fn new() -> Self {
        let vault = vec![
            TTEntry {
                hash: 0,
                score: 0,
                depth: 0,
                flag: 0,
                best_move: 0,
            };
            1 << 22
        ];
        TransponationTable { vault }
    }

    pub fn store(&mut self, hash: u64, score: i32, depth: u8, flag: u8, best_move: u16) {
        if self.vault[hash as usize & SHIFT].depth <= depth {
            self.vault[hash as usize & SHIFT] = TTEntry {
                hash,
                score,
                depth,
                flag,
                best_move,
            };
        }
    }

    pub fn lookup(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let entry = self.vault[hash as usize & SHIFT];
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
        let entry = self.vault[hash as usize & SHIFT];
        if entry.hash == hash && entry.best_move != 0 {
            return Some(entry.best_move);
        }
        None
    }
}
