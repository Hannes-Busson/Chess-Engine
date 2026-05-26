#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
}

pub struct TransponationTable {
    pub vault: Vec<TTEntry>,
}

impl TransponationTable {
    pub fn new() -> Self {
        let vault = vec![
            TTEntry {
                hash: 0,
                score: 0,
                depth: 0,
                flag: 0
            };
            1 << 22
        ];
        TransponationTable { vault }
    }

    pub fn store(&mut self, hash: u64, score: i32, depth: u8, flag: u8) {
        self.vault[hash as usize & ((1 << 22) - 1)] = TTEntry {
            hash,
            score,
            depth,
            flag,
        };
    }

    pub fn lookup(&self, hash: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {}
}
