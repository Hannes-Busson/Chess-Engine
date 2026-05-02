use crate::bitboard::Bitboard;
use crate::position::Color;

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

const fn knight_attacks_u64(sq: u8) -> u64 {
    let bb = 1u64 << sq;
    let not_a = !0x0101010101010101u64;
    let not_h = !0x8080808080808080u64;
    let not_ab = !0x0303030303030303u64;
    let not_gh = !0xC0C0C0C0C0C0C0C0u64;
    ((bb & not_a) << 15)
        | ((bb & not_h) << 17)
        | ((bb & not_h) >> 15)
        | ((bb & not_a) >> 17)
        | ((bb & not_ab) << 6)
        | ((bb & not_ab) >> 10)
        | ((bb & not_gh) << 10)
        | ((bb & not_gh) >> 6)
}

const fn pawn_attacks_u64(sq: u8, white: bool) -> u64 {
    let bb = 1u64 << sq;
    let not_a = !0x0101010101010101u64;
    let not_h = !0x8080808080808080u64;
    if white {
        ((bb & not_a) << 7) | ((bb & not_h) << 9)
    } else {
        ((bb & not_a) >> 9) | ((bb & not_h) >> 7)
    }
}

const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut table = [Bitboard(0); 64];
    let mut sq = 0u8;
    while sq < 64 {
        table[sq as usize] = Bitboard(knight_attacks_u64(sq));
        sq += 1;
    }
    table
};

const PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
    let mut table = [[Bitboard(0); 64]; 2];
    let mut sq = 0u8;
    while sq < 64 {
        table[0][sq as usize] = Bitboard(pawn_attacks_u64(sq, true));
        table[1][sq as usize] = Bitboard(pawn_attacks_u64(sq, false));
        sq += 1;
    }
    table
};

pub struct MoveGen;

impl MoveGen {
    pub fn knight_attacks(square: u8) -> Bitboard {
        KNIGHT_ATTACKS[square as usize]
    }

    pub fn pawn_attacks(square: u8, color: Color) -> Bitboard {
        PAWN_ATTACKS[color as usize][square as usize]
    }
}
