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

    pub fn pop_lsb(&mut self) -> u8 {
        for index in 0..64{
            if 
        }
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

pub struct CastlingRights;

impl CastlingRights {
    pub const WK: u8 = 0b00000001;
    pub const WQ: u8 = 0b00000010;
    pub const BK: u8 = 0b00000100;
    pub const BQ: u8 = 0b00001000;
    pub const ALL: u8 = 0b00001111;
    pub const NONE: u8 = 0b00000000;
}

const CASTLING_UPDATE: [u8; 64] = {
    let mut table = [0xFFu8; 64];
    table[4] = !(CastlingRights::WK | CastlingRights::WQ);
    table[0] = !CastlingRights::WQ;
    table[7] = !CastlingRights::WK;
    table[60] = !(CastlingRights::BK | CastlingRights::BQ);
    table[56] = !CastlingRights::BQ;
    table[63] = !CastlingRights::BK;
    table
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub pieces: [Bitboard; 12],
    pub side_to_move: Color,
    pub castling: u8,
    pub en_passant: u8,
}

impl Position {
    pub fn start() -> Self {
        let mut pieces = [Bitboard(0); 12];
        pieces[Color::White as usize * 6 + PieceType::Pawn as usize] = Bitboard(0x000000000000FF00);
        pieces[Color::Black as usize * 6 + PieceType::Pawn as usize] = Bitboard(0x00FF000000000000);
        pieces[Color::White as usize * 6 + PieceType::Knight as usize] =
            Bitboard(0x0000000000000042);
        pieces[Color::Black as usize * 6 + PieceType::Knight as usize] =
            Bitboard(0x4200000000000000);
        pieces[Color::White as usize * 6 + PieceType::Bishop as usize] =
            Bitboard(0x0000000000000024);
        pieces[Color::Black as usize * 6 + PieceType::Bishop as usize] =
            Bitboard(0x2400000000000000);
        pieces[Color::White as usize * 6 + PieceType::Rock as usize] = Bitboard(0x0000000000000081);
        pieces[Color::Black as usize * 6 + PieceType::Rock as usize] = Bitboard(0x8100000000000000);
        pieces[Color::White as usize * 6 + PieceType::Queen as usize] =
            Bitboard(0x0000000000000008);
        pieces[Color::Black as usize * 6 + PieceType::Queen as usize] =
            Bitboard(0x0800000000000000);
        pieces[Color::White as usize * 6 + PieceType::King as usize] = Bitboard(0x0000000000000010);
        pieces[Color::Black as usize * 6 + PieceType::King as usize] = Bitboard(0x1000000000000000);

        let side_to_move = Color::White;
        let castling = CastlingRights::ALL;
        let en_passant = 64;

        Position {
            pieces: pieces,
            side_to_move: side_to_move,
            castling: castling,
            en_passant: en_passant,
        }
    }

    pub fn get_piece_bitboard(&self, color: Color, piece: PieceType) -> &Bitboard {
        let index = (color as usize * 6) + (piece as usize);
        &self.pieces[index]
    }

    pub fn get_piece_bitboard_mut(&mut self, color: Color, piece: PieceType) -> &mut Bitboard {
        let index = (color as usize * 6) + (piece as usize);
        &mut self.pieces[index]
    }

    pub fn occupancy(&self) -> Bitboard {
        let mut value = Bitboard(0);
        for s in self.pieces {
            value = value | s;
        }
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub value: u16,
}

impl Move {
    pub fn new(from: u8, to: u8, flags: u8) -> Self {
        let value: u16 = (from as u16) | ((to as u16) << 6) | ((flags as u16) << 12);
        Move { value }
    }
    pub fn from(&self) -> u8 {
        (self.value & 0x3F) as u8
    }
    pub fn to(&self) -> u8 {
        ((self.value >> 6) & 0x3F) as u8
    }
    pub fn flags(&self) -> u8 {
        ((self.value >> 12) & 0xF) as u8
    }
}

pub struct MoveFlags;

impl MoveFlags {
    pub const QUIET: u8 = 0;
    pub const DOUBLE_PAWN_PUSH: u8 = 1;
    pub const KINGSIDE_CASTLE: u8 = 2;
    pub const QUEENSIDE_CASTLE: u8 = 3;
    pub const CAPTURE: u8 = 4;
    pub const EN_PASSANT: u8 = 5;
    pub const KNIGHT_PROMOTION: u8 = 8;
    pub const BISHOP_PROMOTION: u8 = 9;
    pub const ROOK_PROMOTION: u8 = 10;
    pub const QUEEN_PROMOTION: u8 = 11;
}
