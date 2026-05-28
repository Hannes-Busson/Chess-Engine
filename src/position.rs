use crate::bitboard::Bitboard;
use crate::movegen::{MagicTable, Move, MoveFlags, MoveGen};
use crate::zobrist;

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
    pub hash: u64,
    pub occ: Bitboard,
    pub occ_by: [Bitboard; 2],
}

impl Position {
    pub fn start() -> Self {
        let mut hash = 0u64;
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
                let index = bb.pop_lsb() as usize;
                piece_on[index] = idx as u8;
                hash = hash ^ zobrist::keys()[idx * 64 + index];
            }
        }

        for i in 0..4 {
            hash = hash ^ zobrist::keys()[769 + i]
        }

        let mut occ = Bitboard(0);
        for p in &pieces {
            occ = occ | *p;
        }
        let mut occ_white = Bitboard(0);
        for i in 0..6 {
            occ_white = occ_white | pieces[i];
        }
        let occ_by = [occ_white, occ & !occ_white];

        Position {
            pieces,
            piece_on,
            side_to_move: Color::White,
            castling: CastlingRights::ALL,
            en_passant: 64,
            hash,
            occ,
            occ_by,
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

    pub fn display(&self) {
        const PIECE_CHARS: [char; 6] = ['P', 'N', 'B', 'R', 'Q', 'K'];
        println!("  A B C D E F G H");
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let square = rank * 8 + file;
                let idx = self.piece_on[square as usize];
                if idx == 64 {
                    print!(". ")
                } else {
                    let ch = PIECE_CHARS[(idx % 6) as usize];
                    if idx / 6 == 0 {
                        print!("{} ", ch.to_ascii_uppercase());
                    } else {
                        print!("{} ", ch.to_ascii_lowercase());
                    }
                }
            }
            print!("/r");
            println!();
        }
        println!();
    }

    pub fn make_move(self, to_move: Move) -> Self {
        let mut result = self;
        result.en_passant = 64;
        let color = self.side_to_move;
        let from = to_move.from();
        let to = to_move.to();
        let flag = to_move.flags();
        let piece_idx = self.piece_on[from as usize] as usize;
        let opponent_idx = self.piece_on[to as usize] as usize;
        result.castling &= CASTLING_UPDATE[from as usize];
        result.castling &= CASTLING_UPDATE[to as usize];
        match flag {
            MoveFlags::QUIET => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[piece_idx].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
            }
            MoveFlags::CAPTURE => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[opponent_idx].clear_bit(to);
                result.pieces[piece_idx].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
                result.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::KNIGHT_PROMOTION => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[(color as usize) * 6 + 1].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 1;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 1) * 64 + to as usize];
            }
            MoveFlags::BISHOP_PROMOTION => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[(color as usize) * 6 + 2].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 2;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 2) * 64 + to as usize];
            }
            MoveFlags::ROOK_PROMOTION => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[(color as usize) * 6 + 3].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 3;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 3) * 64 + to as usize];
            }
            MoveFlags::QUEEN_PROMOTION => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[(color as usize) * 6 + 4].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 4;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 4) * 64 + to as usize];
            }
            MoveFlags::KNIGHT_PROMOTION_CAPTURE => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[opponent_idx].clear_bit(to);
                result.pieces[(color as usize) * 6 + 1].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 1;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 1) * 64 + to as usize];
                result.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::BISHOP_PROMOTION_CAPTURE => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[opponent_idx].clear_bit(to);
                result.pieces[(color as usize) * 6 + 2].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 2;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 2) * 64 + to as usize];
                result.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::ROOK_PROMOTION_CAPTURE => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[opponent_idx].clear_bit(to);
                result.pieces[(color as usize) * 6 + 3].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 3;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 3) * 64 + to as usize];
                result.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::QUEEN_PROMOTION_CAPTURE => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[opponent_idx].clear_bit(to);
                result.pieces[(color as usize) * 6 + 4].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = (color as u8) * 6 + 4;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[((color as usize) * 6 + 4) * 64 + to as usize];
                result.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::KINGSIDE_CASTLE => {
                result.pieces[(color as usize * 6) + 5].clear_bit((color as u8) * 56 + 4);
                result.pieces[(color as usize * 6) + 5].set_bit((color as u8) * 56 + 6);
                result.pieces[(color as usize * 6) + 3].clear_bit((color as u8) * 56 + 7);
                result.pieces[(color as usize * 6) + 3].set_bit((color as u8) * 56 + 5);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.piece_on[(color as usize) * 56 + 7] = 64;
                result.piece_on[(color as usize) * 56 + 5] = (color as u8 * 6) + 3;
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 4) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 6) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 7) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 5) as usize];
            }
            MoveFlags::QUEENSIDE_CASTLE => {
                result.pieces[(color as usize * 6) + 5].clear_bit((color as u8) * 56 + 4);
                result.pieces[(color as usize * 6) + 5].set_bit((color as u8) * 56 + 2);
                result.pieces[(color as usize * 6) + 3].clear_bit((color as u8) * 56 + 0);
                result.pieces[(color as usize * 6) + 3].set_bit((color as u8) * 56 + 3);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.piece_on[(color as usize) * 56 + 0] = 64;
                result.piece_on[(color as usize) * 56 + 3] = (color as u8 * 6) + 3;
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 4) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 2) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 0) as usize];
                result.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 3) as usize];
            }
            MoveFlags::EN_PASSANT => {
                result.pieces[(color as usize) * 6 + 0].clear_bit(from);
                result.pieces[(1 - (color as usize)) * 6 + 0]
                    .clear_bit(to + (color as u8) * 16 - 8);
                result.pieces[(color as usize) * 6 + 0].set_bit(to);
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.piece_on[to as usize + (color as usize) * 16 - 8] = 64;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
                result.hash ^= zobrist::keys()
                    [(1 - color as usize) * 6 * 64 + (to as usize + (color as usize) * 16 - 8)];
            }
            MoveFlags::DOUBLE_PAWN_PUSH => {
                result.pieces[piece_idx].clear_bit(from);
                result.pieces[piece_idx].set_bit(to);
                result.en_passant = from + 8 - (color as u8) * 16;
                result.piece_on[from as usize] = 64;
                result.piece_on[to as usize] = piece_idx as u8;
                result.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                result.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
            }
            6_u8..=7_u8 | 16_u8..=u8::MAX => {}
        }

        result.hash ^= zobrist::keys()[768];
        for i in 0..4 {
            if (self.castling >> i) & 1 != 0 {
                result.hash ^= zobrist::keys()[769 + i];
            }
        }
        for i in 0..4 {
            if (result.castling >> i) & 1 != 0 {
                result.hash ^= zobrist::keys()[769 + i];
            }
        }
        if self.en_passant != 64 {
            result.hash ^= zobrist::keys()[773 + (self.en_passant % 8) as usize];
        }
        if result.en_passant != 64 {
            result.hash ^= zobrist::keys()[773 + (result.en_passant % 8) as usize];
        }

        result.occ = Bitboard(0);
        for p in &result.pieces {
            result.occ = result.occ | *p;
        }
        let mut occ_white = Bitboard(0);
        for i in 0..6 {
            occ_white = occ_white | result.pieces[i];
        }
        result.occ_by = [occ_white, result.occ & !occ_white];

        result.side_to_move = self.opponent();
        result
    }

    pub fn all_moves(self, table: &MagicTable) -> Vec<Move> {
        MoveGen::generate_legal_moves(self, table)
    }

    pub fn all_captures(self, table: &MagicTable) -> Vec<Move> {
        MoveGen::generate_legal_captures(self, table)
    }

    pub fn king_under_attack(&self, table: &MagicTable) -> bool {
        MoveGen::is_attacked(
            self.pieces[(self.side_to_move as u8 * 6 + 5) as usize]
                .0
                .trailing_zeros() as u8,
            &self,
            table,
        )
    }
}
