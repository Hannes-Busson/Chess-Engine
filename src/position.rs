use crate::bitboard::Bitboard;
use crate::movegen::{Move, MoveFlags};
use crate::position;
use std::iter::Enumerate;
use std::result;

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
    Rook = 3,
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

pub const CASTLING_UPDATE: [u8; 64] = {
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
    pub piece_on: [u8; 64],
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
        pieces[Color::White as usize * 6 + PieceType::Rook as usize] = Bitboard(0x0000000000000081);
        pieces[Color::Black as usize * 6 + PieceType::Rook as usize] = Bitboard(0x8100000000000000);
        pieces[Color::White as usize * 6 + PieceType::Queen as usize] =
            Bitboard(0x0000000000000008);
        pieces[Color::Black as usize * 6 + PieceType::Queen as usize] =
            Bitboard(0x0800000000000000);
        pieces[Color::White as usize * 6 + PieceType::King as usize] = Bitboard(0x0000000000000010);
        pieces[Color::Black as usize * 6 + PieceType::King as usize] = Bitboard(0x1000000000000000);

        let mut piece_on = [64u8; 64];

        for (idx, p) in pieces.iter().enumerate() {
            let mut bb = *p;
            while bb.0 != 0 {
                piece_on[bb.pop_lsb() as usize] = idx as u8;
            }
        }
        Position {
            pieces,
            piece_on,
            side_to_move: Color::White,
            castling: CastlingRights::ALL,
            en_passant: 64,
        }
    }

    pub fn get_piece_bitboard(&self, color: Color, piece: PieceType) -> &Bitboard {
        &self.pieces[color as usize * 6 + piece as usize]
    }

    pub fn get_piece_bitboard_mut(&mut self, color: Color, piece: PieceType) -> &mut Bitboard {
        &mut self.pieces[color as usize * 6 + piece as usize]
    }

    pub fn occupancy(&self) -> Bitboard {
        let mut value = Bitboard(0);
        for s in self.pieces {
            value = value | s;
        }
        value
    }

    pub fn occupancy_for(&self, color: Color) -> Bitboard {
        let mut value = Bitboard(0);
        for i in color as usize * 6..color as usize * 6 + 6 {
            value = value | self.pieces[i];
        }
        value
    }

    pub fn opponent(self) -> Color {
        match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn make_move(self, to_move: Move) -> Self {
        let mut result = self;
        result.en_passant = 64;
        let color = self.side_to_move;
        let from = to_move.from();
        let to = to_move.to();
        let flag = to_move.flags();
        result.castling &= CASTLING_UPDATE[from as usize];
        let piece_idx = self.piece_on[from as usize] as usize;
        let opponent_start = (1 - color as usize) * 6;
        match flag {
            MoveFlags::QUIET => 
        }
        if color == Color::White {
            if flag == MoveFlags::QUIET || flag == MoveFlags::CAPTURE {
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Pawn, &mut result);
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Knight, &mut result);
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Bishop, &mut result);
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Rook, &mut result);
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Queen, &mut result);
                self.quiet_or_capture_move(to_move, Color::White, PieceType::King, &mut result);
            }
            if flag == MoveFlags::KNIGHT_PROMOTION || flag == MoveFlags::KNIGHT_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::White, PieceType::Knight, &mut result);
            }
            if flag == MoveFlags::BISHOP_PROMOTION || flag == MoveFlags::BISHOP_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::White, PieceType::Bishop, &mut result);
            }
            if flag == MoveFlags::ROOK_PROMOTION || flag == MoveFlags::ROOK_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::White, PieceType::Rook, &mut result);
            }
            if flag == MoveFlags::QUEEN_PROMOTION || flag == MoveFlags::QUEEN_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::White, PieceType::Queen, &mut result);
            }
            if flag == MoveFlags::KINGSIDE_CASTLE {
                result.pieces[5].clear_bit(4);
                result.pieces[5].set_bit(6);
                result.pieces[3].clear_bit(7);
                result.pieces[3].set_bit(5);
            }
            if flag == MoveFlags::QUEENSIDE_CASTLE {
                result.pieces[5].clear_bit(4);
                result.pieces[5].set_bit(2);
                result.pieces[3].clear_bit(0);
                result.pieces[3].set_bit(3);
            }
            if flag == MoveFlags::EN_PASSANT {
                result.pieces[0].clear_bit(from);
                result.pieces[6].clear_bit(to - 8);
                result.pieces[0].set_bit(to);
            }
            if flag == MoveFlags::DOUBLE_PAWN_PUSH {
                self.quiet_or_capture_move(to_move, Color::White, PieceType::Pawn, &mut result);
                result.en_passant = from + 8;
            }
        } else {
            if flag == MoveFlags::QUIET || flag == MoveFlags::CAPTURE {
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Pawn, &mut result);
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Knight, &mut result);
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Bishop, &mut result);
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Rook, &mut result);
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Queen, &mut result);
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::King, &mut result);
            }
            if flag == MoveFlags::KNIGHT_PROMOTION || flag == MoveFlags::KNIGHT_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::Black, PieceType::Knight, &mut result);
            }
            if flag == MoveFlags::BISHOP_PROMOTION || flag == MoveFlags::BISHOP_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::Black, PieceType::Bishop, &mut result);
            }
            if flag == MoveFlags::ROOK_PROMOTION || flag == MoveFlags::ROOK_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::Black, PieceType::Rook, &mut result);
            }
            if flag == MoveFlags::QUEEN_PROMOTION || flag == MoveFlags::QUEEN_PROMOTION_CAPTURE {
                self.promotion_move(to_move, Color::Black, PieceType::Queen, &mut result);
            }
            if flag == MoveFlags::KINGSIDE_CASTLE {
                result.pieces[11].clear_bit(60);
                result.pieces[11].set_bit(62);
                result.pieces[9].clear_bit(63);
                result.pieces[9].set_bit(61);
            }
            if flag == MoveFlags::QUEENSIDE_CASTLE {
                result.pieces[11].clear_bit(60);
                result.pieces[11].set_bit(58);
                result.pieces[9].clear_bit(56);
                result.pieces[9].set_bit(59);
            }
            if flag == MoveFlags::EN_PASSANT {
                result.pieces[6].clear_bit(from);
                result.pieces[0].clear_bit(to + 8);
                result.pieces[6].set_bit(to);
            }
            if flag == MoveFlags::DOUBLE_PAWN_PUSH {
                self.quiet_or_capture_move(to_move, Color::Black, PieceType::Pawn, &mut result);
                result.en_passant = from - 8;
            }
        }

        result.side_to_move = self.opponent();
        result
    }
    pub fn quiet_or_capture_move(
        &self,
        to_move: Move,
        color: Color,
        piece: PieceType,
        result: &mut Position,
    ) {
        let from = to_move.from();
        let to = to_move.to();
        let occupancy = self.get_piece_bitboard(color, piece);
        if occupancy.get_bit(from) {
            result.pieces[color as usize * 6 + piece as usize].clear_bit(from);
            result.pieces[color as usize * 6 + piece as usize].set_bit(to);
        }
        if to_move.flags() == MoveFlags::CAPTURE {
            for i in 0..6 {
                result.pieces[(1 - color as usize) * 6 + i].clear_bit(to);
            }
        }
    }
    pub fn promotion_move(
        &self,
        to_move: Move,
        color: Color,
        piece: PieceType,
        result: &mut Position,
    ) {
        let from = to_move.from();
        let to = to_move.to();
        let occupancy = self.get_piece_bitboard(color, PieceType::Pawn);
        if occupancy.get_bit(from) {
            result.pieces[color as usize * 6 + PieceType::Pawn as usize].clear_bit(from);
            result.pieces[color as usize * 6 + piece as usize].set_bit(to);
        }
        if to_move.flags() >= MoveFlags::KNIGHT_PROMOTION_CAPTURE {
            for i in 0..6 {
                result.pieces[(1 - color as usize) * 6 + i].clear_bit(to);
            }
        }
    }
}
