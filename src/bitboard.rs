#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn new() -> Self {
        Bitboard(0)
    }

    pub fn get_bit(&self, square: u8) -> bool {
        (self.0 >> square) & 1 == 1
    }

    pub fn set_bit(&mut self, square: u8) {
        self.0 = self.0 | 1 << square;
    }

    pub fn clear_bit(&mut self, square: u8) {
        self.0 = self.0 & !(1 << square);
    }

    pub fn pop_lsb(&mut self) -> u8 {
        let index: u8 = self.0.trailing_zeros() as u8;
        self.0 = self.0 & (self.0 - 1);
        index
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn visualize(&self) {
        println!("  A B C D E F G H");
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square = rank * 8 + file;
                if self.get_bit(square) {
                    print!("X ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        println!();
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Bitboard;
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

impl std::ops::Shl<u8> for Bitboard {
    type Output = Bitboard;
    fn shl(self, rhs: u8) -> Bitboard {
        Bitboard(self.0 << rhs)
    }
}

impl std::ops::Shr<u8> for Bitboard {
    type Output = Bitboard;
    fn shr(self, rhs: u8) -> Bitboard {
        Bitboard(self.0 >> rhs)
    }
}
