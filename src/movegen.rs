use std::vec;

use crate::bitboard::Bitboard;
use crate::masks::Files;
use crate::masks::Ranks;
use crate::position::{CastlingRights, Color, PieceType, Position};

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
        (self.value & 0x3Fu16) as u8
    }
    pub fn to(&self) -> u8 {
        ((self.value >> 6) & 0x3Fu16) as u8
    }
    pub fn flags(&self) -> u8 {
        ((self.value >> 12) & 0xFu16) as u8
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
    pub const KNIGHT_PROMOTION_CAPTURE: u8 = 12;
    pub const BISHOP_PROMOTION_CAPTURE: u8 = 13;
    pub const ROOK_PROMOTION_CAPTURE: u8 = 14;
    pub const QUEEN_PROMOTION_CAPTURE: u8 = 15;
}

pub struct MoveList {
    pub moves: [Move; 256],
    pub len: u16,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move { value: 0 }; 256],
            len: 0,
        }
    }
    pub fn push(&mut self, mv: Move) {
        self.moves[self.len as usize] = mv;
        self.len += 1;
    }
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len as usize]
    }
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len as usize]
    }
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

const fn king_attacks_u64(sq: u8) -> u64 {
    let bb = 1u64 << sq;
    let not_a = !0x0101010101010101u64;
    let not_h = !0x8080808080808080u64;
    (bb << 8)
        | (bb >> 8)
        | ((bb & not_a) << 7)
        | ((bb & not_a) >> 1)
        | ((bb & not_a) >> 9)
        | ((bb & not_h) << 9)
        | ((bb & not_h) << 1)
        | ((bb & not_h) >> 7)
}

pub fn bishop_mask(sq: u8) -> Bitboard {
    let mut position = Bitboard(0);
    position.set_bit(sq);
    let mut moveable = position;
    let mut ne = Bitboard(0);
    let mut se = Bitboard(0);
    let mut sw = Bitboard(0);
    let mut nw = Bitboard(0);
    // NE
    loop {
        moveable = (moveable & !Files::FILE_H) << 9;
        ne = ne | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // SE
    loop {
        moveable = (moveable & !Files::FILE_H) >> 7;
        se = se | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // SW
    loop {
        moveable = (moveable & !Files::FILE_A) >> 9;
        sw = sw | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // NW
    loop {
        moveable = (moveable & !Files::FILE_A) << 7;
        nw = nw | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    ne & !(Ranks::RANK_8 | Files::FILE_H)
        | se & !(Files::FILE_H | Ranks::RANK_1)
        | sw & !(Ranks::RANK_1 | Files::FILE_A)
        | nw & !(Files::FILE_A | Ranks::RANK_8)
}

pub fn rook_mask(sq: u8) -> Bitboard {
    let mut position = Bitboard(0);
    position.set_bit(sq);
    let mut moveable = position;
    let mut north = Bitboard(0);
    let mut east = Bitboard(0);
    let mut south = Bitboard(0);
    let mut west = Bitboard(0);
    // N
    loop {
        moveable = moveable << 8;
        north = north | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // E
    loop {
        moveable = (moveable & !Files::FILE_H) << 1;
        east = east | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // S
    loop {
        moveable = moveable >> 8;
        south = south | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    moveable = position;
    // W
    loop {
        moveable = (moveable & !Files::FILE_A) >> 1;
        west = west | moveable;
        if moveable.0 == 0 {
            break;
        }
    }
    north & !Ranks::RANK_8 | east & !Files::FILE_H | south & !Ranks::RANK_1 | west & !Files::FILE_A
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

const KING_ATTACKS: [Bitboard; 64] = {
    let mut table = [Bitboard(0); 64];
    let mut sq = 0u8;
    while sq < 64 {
        table[sq as usize] = Bitboard(king_attacks_u64(sq));
        sq += 1;
    }
    table
};

const BISHOP_MAGICS: [u64; 64] = [
    0xFFEDF9FD7CFCFFFF,
    0xFC0962854A77F576,
    0x5822022042000000,
    0x2CA804A100200020,
    0x0204042200000900,
    0x2002121024000002,
    0xFC0A66C64A7EF576,
    0x7FFDFDFCBD79FFFF,
    0xFC0846A64A34FFF6,
    0xFC087A874A3CF7F6,
    0x1001080204002100,
    0x1810080489021800,
    0x0062040420010A00,
    0x5028043004300020,
    0xFC0864AE59B4FF76,
    0x3C0860AF4B35FF76,
    0x73C01AF56CF4CFFB,
    0x41A01CFAD64AAFFC,
    0x040C0422080A0598,
    0x4228020082004050,
    0x0200800400E00100,
    0x020B001230021040,
    0x7C0C028F5B34FF76,
    0xFC0A028E5AB4DF76,
    0x0020208050A42180,
    0x001004804B280200,
    0x2048020024040010,
    0x0102C04004010200,
    0x020408204C002010,
    0x02411100020080C1,
    0x102A008084042100,
    0x0941030000A09846,
    0x0244100800400200,
    0x4000901010080696,
    0x0000280404180020,
    0x0800042008240100,
    0x0220008400088020,
    0x04020182000904C9,
    0x0023010400020600,
    0x0041040020110302,
    0xDCEFD9B54BFCC09F,
    0xF95FFA765AFD602B,
    0x1401210240484800,
    0x0022244208010080,
    0x1105040104000210,
    0x2040088800C40081,
    0x43FF9A5CF4CA0C01,
    0x4BFFCD8E7C587601,
    0xFC0FF2865334F576,
    0xFC0BF6CE5924F576,
    0x80000B0401040402,
    0x0020004821880A00,
    0x8200002022440100,
    0x0009431801010068,
    0xC3FFB7DC36CA8C89,
    0xC3FF8A54F4CA2C89,
    0xFFFFFCFCFD79EDFF,
    0xFC0863FCCB147576,
    0x040C000022013020,
    0x2000104000420600,
    0x0400000260142410,
    0x0800633408100500,
    0xFC087E8E4BB2F736,
    0x43FF9E4EF4CA2C89,
];

const ROOK_MAGICS: [u64; 64] = [
    0xA180022080400230,
    0x0040100040022000,
    0x0080088020001002,
    0x0080080280841000,
    0x4200042010460008,
    0x04800A0003040080,
    0x0400110082041008,
    0x008000A041000880,
    0x10138001A080C010,
    0x0000804008200480,
    0x00010011012000C0,
    0x0022004128102200,
    0x000200081201200C,
    0x202A001048460004,
    0x0081000100420004,
    0x4000800380004500,
    0x0000208002904001,
    0x0090004040026008,
    0x0208808010002001,
    0x2002020020704940,
    0x8048010008110005,
    0x6820808004002200,
    0x0A80040008023011,
    0x00B1460000811044,
    0x4204400080008EA0,
    0xB002400180200184,
    0x2020200080100380,
    0x0010080080100080,
    0x2204080080800400,
    0x0000A40080360080,
    0x02040604002810B1,
    0x008C218600004104,
    0x8180004000402000,
    0x488C402000401001,
    0x4018A00080801004,
    0x1230002105001008,
    0x8904800800800400,
    0x0042000C42003810,
    0x008408110400B012,
    0x0018086182000401,
    0x2240088020C28000,
    0x001001201040C004,
    0x0A02008010420020,
    0x0010003009010060,
    0x0004008008008014,
    0x0080020004008080,
    0x0282020001008080,
    0x50000181204A0004,
    0x48FFFE99FECFAA00,
    0x48FFFE99FECFAA00,
    0x497FFFADFF9C2E00,
    0x613FFFDDFFCE9200,
    0xFFFFFFE9FFE7CE00,
    0xFFFFFFF5FFF3E600,
    0x0010301802830400,
    0x510FFFF5F63C96A0,
    0xEBFFFFB9FF9FC526,
    0x61FFFEDDFEEDAEAE,
    0x53BFFFEDFFDEB1A2,
    0x127FFFB9FFDFB5F6,
    0x411FFFDDFFDBF4D6,
    0x0801000804000603,
    0x0003FFEF27EEBE74,
    0x7645FFFECBFEA79E,
];

pub struct MagicTable {
    pub bishop_masks: [Bitboard; 64],
    pub bishop_magics: [u64; 64],
    pub bishop_attacks: Box<[[Bitboard; 512]]>,
    pub bishop_shifts: [u8; 64],
    pub rook_masks: [Bitboard; 64],
    pub rook_magics: [u64; 64],
    pub rook_attacks: Box<[[Bitboard; 4096]]>,
    pub rook_shifts: [u8; 64],
}

impl MagicTable {
    pub fn init() -> Self {
        let mut bishop_masks = [Bitboard(0); 64];
        let mut rook_masks = [Bitboard(0); 64];
        let mut bishop_shifts = [0u8; 64];
        let mut rook_shifts = [0u8; 64];
        for i in 0..64 {
            bishop_masks[i] = bishop_mask(i as u8);
        }
        for i in 0..64 {
            rook_masks[i] = rook_mask(i as u8);
        }
        let mut bishop_attacks = vec![[Bitboard(0); 512]; 64].into_boxed_slice();
        for i in 0..64 {
            let mask = bishop_masks[i].0;
            let mut subset = 0u64;
            let shift = 64 - mask.count_ones();
            loop {
                let attack = MoveGen::bishop_attacks_slow(i as u8, Bitboard(subset));
                bishop_attacks[i][(subset.wrapping_mul(BISHOP_MAGICS[i]) >> shift) as usize] =
                    attack;
                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0 {
                    break;
                }
            }
        }
        let mut rook_attacks = vec![[Bitboard(0); 4096]; 64].into_boxed_slice();
        for i in 0..64 {
            let mask = rook_masks[i].0;
            let mut subset = 0u64;
            let shift = 64 - mask.count_ones();
            loop {
                let attack = MoveGen::rook_attacks_slow(i as u8, Bitboard(subset));
                rook_attacks[i][(subset.wrapping_mul(ROOK_MAGICS[i]) >> shift) as usize] = attack;
                subset = subset.wrapping_sub(mask) & mask;
                if subset == 0 {
                    break;
                }
            }
        }
        for i in 0..bishop_shifts.len() {
            bishop_shifts[i] = (64 - bishop_masks[i].0.count_ones()) as u8;
        }
        for i in 0..rook_shifts.len() {
            rook_shifts[i] = (64 - rook_masks[i].0.count_ones()) as u8;
        }

        let table = MagicTable {
            bishop_masks: bishop_masks,
            bishop_magics: BISHOP_MAGICS,
            bishop_attacks: bishop_attacks,
            bishop_shifts: bishop_shifts,
            rook_masks: rook_masks,
            rook_magics: ROOK_MAGICS,
            rook_attacks: rook_attacks,
            rook_shifts: rook_shifts,
        };
        table
    }
}

pub struct MoveGen;

impl MoveGen {
    pub fn knight_attacks(square: u8) -> Bitboard {
        KNIGHT_ATTACKS[square as usize]
    }

    pub fn pawn_attacks(square: u8, color: Color) -> Bitboard {
        PAWN_ATTACKS[color as usize][square as usize]
    }

    pub fn king_attacks(square: u8) -> Bitboard {
        KING_ATTACKS[square as usize]
    }

    pub fn generate_pawn_moves(square: u8, occupancy: Bitboard, color: Color) -> Bitboard {
        let mut result;
        let mut position = Bitboard(0);
        position.set_bit(square);
        if color == Color::White {
            result = position << 8 & !occupancy;
            if square > 7 && square < 16 {
                result = result | result << 8 & !occupancy;
            }
        } else {
            result = position >> 8 & !occupancy;
            if square > 47 && square < 56 {
                result = result | result >> 8 & !occupancy;
            }
        }
        result
    }

    pub fn bishop_attacks_slow(square: u8, occupancy: Bitboard) -> Bitboard {
        let mut position = Bitboard(0);
        position.set_bit(square);
        let mut moveable_position = position;
        let mut result = Bitboard(0);
        // NE
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) << 9;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // SE
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) >> 7;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // SW
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) >> 9;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // NW
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) << 7;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        result
    }

    pub fn bishop_attacks(square: u8, occupancy: Bitboard, table: &MagicTable) -> Bitboard {
        let mut result = occupancy & table.bishop_masks[square as usize];
        let index = (result.0.wrapping_mul(table.bishop_magics[square as usize])
            >> table.bishop_shifts[square as usize]) as usize;
        result = table.bishop_attacks[square as usize][index];
        result
    }

    pub fn rook_attacks_slow(square: u8, occupancy: Bitboard) -> Bitboard {
        let mut position = Bitboard(0);
        position.set_bit(square);
        let mut moveable_position = position;
        let mut result = Bitboard(0);
        // N
        loop {
            moveable_position = moveable_position << 8;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // E
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) << 1;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // S
        loop {
            moveable_position = moveable_position >> 8;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        moveable_position = position;
        // W
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) >> 1;
            result = result | moveable_position;
            if moveable_position.0 == 0 {
                break;
            }
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
        }
        result
    }

    pub fn rook_attacks(square: u8, occupancy: Bitboard, table: &MagicTable) -> Bitboard {
        let mut result = occupancy & table.rook_masks[square as usize];
        let index = (result.0.wrapping_mul(table.rook_magics[square as usize])
            >> table.rook_shifts[square as usize]) as usize;
        result = table.rook_attacks[square as usize][index];
        result
    }

    pub fn queen_attacks(square: u8, occupancy: Bitboard, table: &MagicTable) -> Bitboard {
        MoveGen::bishop_attacks(square, occupancy, table)
            | MoveGen::rook_attacks(square, occupancy, table)
    }

    pub fn generate_moves(position: &Position, table: &MagicTable, list: &mut MoveList) {
        let color = position.side_to_move;
        let own_pieces = position.occ_by[color as usize];
        let all_pieces = position.occ;
        let enemy_pieces = all_pieces & !own_pieces;
        // knights
        let mut knights = *position.get_piece_bitboard(color, PieceType::Knight);
        while !knights.is_empty() {
            let from = knights.pop_lsb();
            let mut attacks = MoveGen::knight_attacks(from) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    list.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    list.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // king
        let mut king = *position.get_piece_bitboard(color, PieceType::King);
        let from = king.pop_lsb();
        let mut attacks = MoveGen::king_attacks(from) & !own_pieces;
        while !attacks.is_empty() {
            let to = attacks.pop_lsb();
            if all_pieces.get_bit(to) {
                list.push(Move::new(from, to, MoveFlags::CAPTURE));
            } else {
                list.push(Move::new(from, to, MoveFlags::QUIET));
            }
        }
        // white
        if color == Color::White {
            // kingside castle
            if position.castling & CastlingRights::WK != 0
                && !all_pieces.get_bit(5)
                && !all_pieces.get_bit(6)
                && !MoveGen::is_attacked(color, 4, &position, table)
                && !MoveGen::is_attacked(color, 5, &position, table)
                && !MoveGen::is_attacked(color, 6, &position, table)
            {
                list.push(Move::new(4, 6, MoveFlags::KINGSIDE_CASTLE));
            }
            // queenside castle
            if position.castling & CastlingRights::WQ != 0
                && !all_pieces.get_bit(1)
                && !all_pieces.get_bit(2)
                && !all_pieces.get_bit(3)
                && !MoveGen::is_attacked(color, 2, &position, table)
                && !MoveGen::is_attacked(color, 3, &position, table)
                && !MoveGen::is_attacked(color, 4, &position, table)
            {
                list.push(Move::new(4, 2, MoveFlags::QUEENSIDE_CASTLE));
            }
        } else {
            // kingside castle
            if position.castling & CastlingRights::BK != 0
                && !all_pieces.get_bit(61)
                && !all_pieces.get_bit(62)
                && !MoveGen::is_attacked(color, 60, &position, table)
                && !MoveGen::is_attacked(color, 61, &position, table)
                && !MoveGen::is_attacked(color, 62, &position, table)
            {
                list.push(Move::new(60, 62, MoveFlags::KINGSIDE_CASTLE));
            }
            // queenside castle
            if position.castling & CastlingRights::BQ != 0
                && !all_pieces.get_bit(57)
                && !all_pieces.get_bit(58)
                && !all_pieces.get_bit(59)
                && !MoveGen::is_attacked(color, 58, &position, table)
                && !MoveGen::is_attacked(color, 59, &position, table)
                && !MoveGen::is_attacked(color, 60, &position, table)
            {
                list.push(Move::new(60, 58, MoveFlags::QUEENSIDE_CASTLE));
            }
        }
        // bishops
        let mut bishops = *position.get_piece_bitboard(color, PieceType::Bishop);
        while !bishops.is_empty() {
            let from = bishops.pop_lsb();
            let mut attacks = MoveGen::bishop_attacks(from, all_pieces, table) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    list.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    list.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // rooks
        let mut rooks = *position.get_piece_bitboard(color, PieceType::Rook);
        while !rooks.is_empty() {
            let from = rooks.pop_lsb();
            let mut attacks = MoveGen::rook_attacks(from, all_pieces, table) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    list.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    list.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // queens
        let mut queens = *position.get_piece_bitboard(color, PieceType::Queen);
        while !queens.is_empty() {
            let from = queens.pop_lsb();
            let mut attacks = MoveGen::queen_attacks(from, all_pieces, table) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    list.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    list.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // pawns
        let ep_bb = Bitboard(if position.en_passant != 64 {
            1u64 << position.en_passant
        } else {
            0
        });
        let promotion_rank = if color == Color::White {
            Ranks::RANK_8
        } else {
            Ranks::RANK_1
        };
        let mut pawns = *position.get_piece_bitboard(color, PieceType::Pawn);
        while !pawns.is_empty() {
            let from = pawns.pop_lsb();
            let mut attacks = MoveGen::pawn_attacks(from, color) & (enemy_pieces | ep_bb);
            let mut moves = MoveGen::generate_pawn_moves(from, all_pieces, color);
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if ep_bb.0 != 0 && to == position.en_passant {
                    list.push(Move::new(from, to, MoveFlags::EN_PASSANT));
                } else {
                    if promotion_rank.get_bit(to) {
                        list.push(Move::new(from, to, MoveFlags::KNIGHT_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::BISHOP_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::ROOK_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::QUEEN_PROMOTION_CAPTURE));
                    } else {
                        list.push(Move::new(from, to, MoveFlags::CAPTURE));
                    }
                }
            }
            while !moves.is_empty() {
                let to = moves.pop_lsb();
                if promotion_rank.get_bit(to) {
                    list.push(Move::new(from, to, MoveFlags::KNIGHT_PROMOTION));
                    list.push(Move::new(from, to, MoveFlags::BISHOP_PROMOTION));
                    list.push(Move::new(from, to, MoveFlags::ROOK_PROMOTION));
                    list.push(Move::new(from, to, MoveFlags::QUEEN_PROMOTION));
                } else if to.abs_diff(from) == 16 {
                    list.push(Move::new(from, to, MoveFlags::DOUBLE_PAWN_PUSH));
                } else {
                    list.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
    }

    pub fn generate_captures(position: &Position, table: &MagicTable, list: &mut MoveList) {
        let color = position.side_to_move;
        let own_pieces = position.occ_by[color as usize];
        let all_pieces = position.occ;
        let enemy_pieces = all_pieces & !own_pieces;
        // knights
        let mut knights = *position.get_piece_bitboard(color, PieceType::Knight);
        while !knights.is_empty() {
            let from = knights.pop_lsb();
            let mut attacks = MoveGen::knight_attacks(from) & enemy_pieces;
            while !attacks.is_empty() {
                list.push(Move::new(from, attacks.pop_lsb(), MoveFlags::CAPTURE));
            }
        }
        // king
        let mut king = *position.get_piece_bitboard(color, PieceType::King);
        let from = king.pop_lsb();
        let mut attacks = MoveGen::king_attacks(from) & enemy_pieces;
        while !attacks.is_empty() {
            list.push(Move::new(from, attacks.pop_lsb(), MoveFlags::CAPTURE));
        }
        // whites
        // bishops
        let mut bishops = *position.get_piece_bitboard(color, PieceType::Bishop);
        while !bishops.is_empty() {
            let from = bishops.pop_lsb();
            let mut attacks = MoveGen::bishop_attacks(from, all_pieces, table) & enemy_pieces;
            while !attacks.is_empty() {
                list.push(Move::new(from, attacks.pop_lsb(), MoveFlags::CAPTURE));
            }
        }
        // rooks
        let mut rooks = *position.get_piece_bitboard(color, PieceType::Rook);
        while !rooks.is_empty() {
            let from = rooks.pop_lsb();
            let mut attacks = MoveGen::rook_attacks(from, all_pieces, table) & enemy_pieces;
            while !attacks.is_empty() {
                list.push(Move::new(from, attacks.pop_lsb(), MoveFlags::CAPTURE));
            }
        }
        // queens
        let mut queens = *position.get_piece_bitboard(color, PieceType::Queen);
        while !queens.is_empty() {
            let from = queens.pop_lsb();
            let mut attacks = MoveGen::queen_attacks(from, all_pieces, table) & enemy_pieces;
            while !attacks.is_empty() {
                list.push(Move::new(from, attacks.pop_lsb(), MoveFlags::CAPTURE));
            }
        }
        // pawns
        let ep_bb = Bitboard(if position.en_passant != 64 {
            1u64 << position.en_passant
        } else {
            0
        });
        let promotion_rank = if color == Color::White {
            Ranks::RANK_8
        } else {
            Ranks::RANK_1
        };
        let mut pawns = *position.get_piece_bitboard(color, PieceType::Pawn);
        while !pawns.is_empty() {
            let from = pawns.pop_lsb();
            let mut attacks = MoveGen::pawn_attacks(from, color) & (enemy_pieces | ep_bb);
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if ep_bb.0 != 0 && to == position.en_passant {
                    list.push(Move::new(from, to, MoveFlags::EN_PASSANT));
                } else {
                    if promotion_rank.get_bit(to) {
                        list.push(Move::new(from, to, MoveFlags::KNIGHT_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::BISHOP_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::ROOK_PROMOTION_CAPTURE));
                        list.push(Move::new(from, to, MoveFlags::QUEEN_PROMOTION_CAPTURE));
                    } else {
                        list.push(Move::new(from, to, MoveFlags::CAPTURE));
                    }
                }
            }
        }
    }

    pub fn is_attacked(color: Color, square: u8, position: &Position, table: &MagicTable) -> bool {
        let occupancy = position.occ;
        let opponent_color = match color {
            Color::Black => Color::White,
            Color::White => Color::Black,
        };
        let diagonals = MoveGen::bishop_attacks(square, occupancy, table);
        let straights = MoveGen::rook_attacks(square, occupancy, table);
        let pawns_attacks = MoveGen::pawn_attacks(square, color);
        let king_attacks = MoveGen::king_attacks(square);
        let knight_attacks = MoveGen::knight_attacks(square);
        let enemy_pawns = *position.get_piece_bitboard(opponent_color, PieceType::Pawn);
        let enemy_king = *position.get_piece_bitboard(opponent_color, PieceType::King);
        let enemy_knight = *position.get_piece_bitboard(opponent_color, PieceType::Knight);
        let enemy_bishops = *position.get_piece_bitboard(opponent_color, PieceType::Bishop);
        let enemy_rooks = *position.get_piece_bitboard(opponent_color, PieceType::Rook);
        let enemy_queens = *position.get_piece_bitboard(opponent_color, PieceType::Queen);
        (diagonals & (enemy_bishops | enemy_queens)).0 != 0
            || (straights & (enemy_rooks | enemy_queens)).0 != 0
            || (pawns_attacks & enemy_pawns).0 != 0
            || (king_attacks & enemy_king).0 != 0
            || (knight_attacks & enemy_knight).0 != 0
    }

    pub fn generate_legal_moves(position: &mut Position, table: &MagicTable, list: &mut MoveList) {
        let start = list.len;
        let color = position.side_to_move;
        MoveGen::generate_moves(position, table, list);
        let mut write = start;
        for read in start..list.len {
            let mv = list.moves[read as usize];
            let undo = position.make_move(mv);
            let king_sq = position
                .get_piece_bitboard(color, PieceType::King)
                .0
                .trailing_zeros() as u8;
            let legal = !MoveGen::is_attacked(color, king_sq, position, table);
            position.unmake_move(mv, undo);
            if legal {
                list.moves[write as usize] = list.moves[read as usize];
                write += 1;
            }
        }
        list.len = write
    }

    pub fn generate_legal_captures(
        position: &mut Position,
        table: &MagicTable,
        list: &mut MoveList,
    ) {
        let start = list.len;
        let color = position.side_to_move;
        MoveGen::generate_captures(position, table, list);
        let mut write = start;
        for read in start..list.len {
            let mv = list.moves[read as usize];
            let undo = position.make_move(mv);
            let king_sq = position
                .get_piece_bitboard(color, PieceType::King)
                .0
                .trailing_zeros() as u8;
            let legal = !MoveGen::is_attacked(color, king_sq, position, table);
            position.unmake_move(mv, undo);
            if legal {
                list.moves[write as usize] = list.moves[read as usize];
                write += 1;
            }
        }
        list.len = write
    }
}
