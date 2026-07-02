use crate::board::bitboard::Bitboard;
use std::sync::OnceLock;

// returns bitboards with file set to be ones
pub struct Files;
impl Files {
    pub const FILE_A: Bitboard = Bitboard(0x0101010101010101);
    pub const FILE_B: Bitboard = Bitboard(0x0202020202020202);
    pub const FILE_C: Bitboard = Bitboard(0x0404040404040404);
    pub const FILE_D: Bitboard = Bitboard(0x0808080808080808);
    pub const FILE_E: Bitboard = Bitboard(0x1010101010101010);
    pub const FILE_F: Bitboard = Bitboard(0x2020202020202020);
    pub const FILE_G: Bitboard = Bitboard(0x4040404040404040);
    pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);
}

// returns bitboards with rank set to be ones
pub struct Ranks;
impl Ranks {
    pub const RANK_1: Bitboard = Bitboard(0x00000000000000FF);
    pub const RANK_2: Bitboard = Bitboard(0x000000000000FF00);
    pub const RANK_3: Bitboard = Bitboard(0x0000000000FF0000);
    pub const RANK_4: Bitboard = Bitboard(0x00000000FF000000);
    pub const RANK_5: Bitboard = Bitboard(0x000000FF00000000);
    pub const RANK_6: Bitboard = Bitboard(0x0000FF0000000000);
    pub const RANK_7: Bitboard = Bitboard(0x00FF000000000000);
    pub const RANK_8: Bitboard = Bitboard(0xFF00000000000000);
}

// Once lock only computes the table once and then just a look up
static BETWEEN: OnceLock<[[Bitboard; 64]; 64]> = OnceLock::new();

// creates a bitboard with the squares between a and b set to be ones
fn build_between_table() -> [[Bitboard; 64]; 64] {
    let mut result = [[Bitboard(0); 64]; 64];
    for a in 0..64 {
        for b in 0..64 {
            if a == b {
                continue;
            }
            let (af, ar) = (a % 8, a / 8);
            let (bf, br) = (b % 8, b / 8);
            let df = bf - af as i32;
            let dr = br - ar as i32;
            let step: i32 = if dr == 0 {
                df.signum()
            } else if df == 0 {
                dr.signum() * 8
            } else if df == dr {
                df.signum() * 9
            } else if df == -dr {
                df.signum() * -7
            } else {
                0i32
            };
            if step == 0 {
                continue;
            }
            let mut bb = Bitboard(0);
            let mut next = a + step;
            while next != b {
                bb.set_bit(next as u8);
                next = next + step;
            }
            result[a as usize][b as usize] = bb;
        }
    }
    result
}

// one build then look up
pub fn between(a: u8, b: u8) -> Bitboard {
    BETWEEN.get_or_init(build_between_table)[a as usize][b as usize]
}
