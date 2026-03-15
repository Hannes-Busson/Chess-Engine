fn main() {
    let mut board = Bitboard::new();
    board.visualize();
    board.set_bit(23);
    board.visualize();
    board.set_bit(24);
    board.visualize();
    board.clear_bit(24);
    board.visualize();
}

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
