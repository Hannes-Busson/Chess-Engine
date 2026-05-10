use crate::bitboard::{self, Bitboard};
use crate::masks::Files;
use crate::masks::Ranks;
use crate::position::{self, CastlingRights, Color, PieceType, Position};

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
        let mut result = Bitboard(0);
        let mut position = Bitboard(0);
        position.set_bit(square);
        if color == Color::White {
            result = position << 8 & !occupancy;
            if square > 7 && square < 16 {
                result = result | (position << 8 & !occupancy) << 8 & !occupancy;
            }
        } else {
            result = position >> 8 & !occupancy;
            if square > 47 && square < 56 {
                result = result | (position >> 8 & !occupancy) >> 8 & !occupancy;
            }
        }
        result
    }

    pub fn bishop_attacks(square: u8, occupancy: Bitboard) -> Bitboard {
        let mut position = Bitboard(0);
        position.set_bit(square);
        let mut moveable_position = position;
        let mut result = Bitboard(0);
        // NE
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) << 9;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // SE
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) >> 7;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // SW
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) >> 9;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // NW
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) << 7;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        result
    }

    pub fn rook_attacks(square: u8, occupancy: Bitboard) -> Bitboard {
        let mut position = Bitboard(0);
        position.set_bit(square);
        let mut moveable_position = position;
        let mut result = Bitboard(0);
        // N
        loop {
            moveable_position = moveable_position << 8;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // E
        loop {
            moveable_position = (moveable_position & !Files::FILE_H) << 1;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // S
        loop {
            moveable_position = moveable_position >> 8;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        moveable_position = position;
        // W
        loop {
            moveable_position = (moveable_position & !Files::FILE_A) >> 1;
            result = result | moveable_position;
            if (moveable_position & occupancy).0 != 0 {
                break;
            }
            if moveable_position.0 == 0 {
                break;
            }
        }
        result
    }

    pub fn queen_attacks(square: u8, occupancy: Bitboard) -> Bitboard {
        MoveGen::bishop_attacks(square, occupancy) | MoveGen::rook_attacks(square, occupancy)
    }

    pub fn generate_moves(position: Position) -> Vec<Move> {
        let mut result: Vec<Move> = Vec::new();
        let color = position.side_to_move;
        let own_pieces = position.occupancy_for(color);
        let all_pieces = position.occupancy();
        let enemy_pieces = all_pieces & !own_pieces;
        // knights
        let mut knights = *position.get_piece_bitboard(color, PieceType::Knight);
        while !knights.is_empty() {
            let from = knights.pop_lsb();
            let mut attacks = MoveGen::knight_attacks(from) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    result.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    result.push(Move::new(from, to, MoveFlags::QUIET));
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
                result.push(Move::new(from, to, MoveFlags::CAPTURE));
            } else {
                result.push(Move::new(from, to, MoveFlags::QUIET));
            }
        }
        // white
        if color == Color::White {
            // kingside castle
            if position.castling & CastlingRights::WK != 0
                && !all_pieces.get_bit(5)
                && !all_pieces.get_bit(6)
                && !MoveGen::is_attacked(4, &position)
                && !MoveGen::is_attacked(5, &position)
                && !MoveGen::is_attacked(6, &position)
            {
                result.push(Move::new(4, 6, MoveFlags::KINGSIDE_CASTLE));
            }
            // queenside castle
            if position.castling & CastlingRights::WQ != 0
                && !all_pieces.get_bit(1)
                && !all_pieces.get_bit(2)
                && !all_pieces.get_bit(3)
                && !MoveGen::is_attacked(2, &position)
                && !MoveGen::is_attacked(3, &position)
                && !MoveGen::is_attacked(4, &position)
            {
                result.push(Move::new(4, 2, MoveFlags::QUEENSIDE_CASTLE));
            }
        } else {
            // kingside castle
            if position.castling & CastlingRights::BK != 0
                && !all_pieces.get_bit(61)
                && !all_pieces.get_bit(62)
                && !MoveGen::is_attacked(60, &position)
                && !MoveGen::is_attacked(61, &position)
                && !MoveGen::is_attacked(62, &position)
            {
                result.push(Move::new(60, 62, MoveFlags::KINGSIDE_CASTLE));
            }
            // queenside castle
            if position.castling & CastlingRights::BQ != 0
                && !all_pieces.get_bit(57)
                && !all_pieces.get_bit(58)
                && !all_pieces.get_bit(59)
                && !MoveGen::is_attacked(58, &position)
                && !MoveGen::is_attacked(59, &position)
                && !MoveGen::is_attacked(60, &position)
            {
                result.push(Move::new(60, 58, MoveFlags::QUEENSIDE_CASTLE));
            }
        }
        // bishops
        let mut bishops = *position.get_piece_bitboard(color, PieceType::Bishop);
        while !bishops.is_empty() {
            let from = bishops.pop_lsb();
            let mut attacks = MoveGen::bishop_attacks(from, all_pieces) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    result.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    result.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // rooks
        let mut rooks = *position.get_piece_bitboard(color, PieceType::Rook);
        while !rooks.is_empty() {
            let from = rooks.pop_lsb();
            let mut attacks = MoveGen::rook_attacks(from, all_pieces) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    result.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    result.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // queens
        let mut queens = *position.get_piece_bitboard(color, PieceType::Queen);
        while !queens.is_empty() {
            let from = queens.pop_lsb();
            let mut attacks = MoveGen::queen_attacks(from, all_pieces) & !own_pieces;
            while !attacks.is_empty() {
                let to = attacks.pop_lsb();
                if all_pieces.get_bit(to) {
                    result.push(Move::new(from, to, MoveFlags::CAPTURE));
                } else {
                    result.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        // pawns
        let ep_bb = if position.en_passant != 64 {
            let mut bb = Bitboard(0);
            bb.set_bit(position.en_passant);
            bb
        } else {
            Bitboard(0)
        };
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
                    result.push(Move::new(from, to, MoveFlags::EN_PASSANT));
                } else {
                    if promotion_rank.get_bit(to) {
                        result.push(Move::new(from, to, MoveFlags::KNIGHT_PROMOTION_CAPTURE));
                        result.push(Move::new(from, to, MoveFlags::BISHOP_PROMOTION_CAPTURE));
                        result.push(Move::new(from, to, MoveFlags::ROOK_PROMOTION_CAPTURE));
                        result.push(Move::new(from, to, MoveFlags::QUEEN_PROMOTION_CAPTURE));
                    } else {
                        result.push(Move::new(from, to, MoveFlags::CAPTURE));
                    }
                }
            }
            while !moves.is_empty() {
                let to = moves.pop_lsb();
                if promotion_rank.get_bit(to) {
                    result.push(Move::new(from, to, MoveFlags::KNIGHT_PROMOTION));
                    result.push(Move::new(from, to, MoveFlags::BISHOP_PROMOTION));
                    result.push(Move::new(from, to, MoveFlags::ROOK_PROMOTION));
                    result.push(Move::new(from, to, MoveFlags::QUEEN_PROMOTION));
                } else if to.abs_diff(from) == 16 {
                    result.push(Move::new(from, to, MoveFlags::DOUBLE_PAWN_PUSH));
                } else {
                    result.push(Move::new(from, to, MoveFlags::QUIET));
                }
            }
        }
        result
    }

    pub fn is_attacked(square: u8, position: &Position) -> bool {
        let occupancy = position.occupancy();
        let color = position.side_to_move;
        let opponent_color = position.opponent();
        let diagonals = MoveGen::bishop_attacks(square, occupancy);
        let straights = MoveGen::rook_attacks(square, occupancy);
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
}
