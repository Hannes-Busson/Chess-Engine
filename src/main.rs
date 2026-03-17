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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Color {
    White = 0,
    Black = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rock = 3,
    Queen = 4,
    King = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub pieces: [Bitboard; 12],
    pub side_to_move: Color,
    pub en_passant: u8,
    pub castling: u8,
}

impl Position {
    pub fn get_piece_bitboard(&self, color: Color, piece: PieceType) -> &Bitboard {
        let index = (color as usize * 6) + (piece as usize);
        &self.pieces[index]
    }

    pub fn get_piece_bitboard_mut(&mut self, color: Color, piece: PieceType) -> &mut Bitboard {
        let index = (color as usize * 6) + (piece as usize);
        &mut self.pieces[index]
    }
}
