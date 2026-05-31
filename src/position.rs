use crate::bitboard::Bitboard;
use crate::movegen::{MagicTable, Move, MoveFlags, MoveGen, MoveList};
use crate::zobrist;

pub struct UndoInfo {
    pub moving_piece: u8,
    pub captured: u8,
    pub captured_sq: u8,
    pub en_passant: u8,
    pub castling: u8,
    pub hash: u64,
    pub occ: Bitboard,
    pub occ_by: [Bitboard; 2],
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

    pub fn from_fen(fen: &str) -> Self {
        let mut hash = 0u64;
        let mut pieces = [Bitboard(0); 12];
        let mut piece_on = [64u8; 64];
        let mut square = 0;

        let space_splitted: Vec<&str> = fen.split(' ').collect();
        let slash_splitted: Vec<&str> = space_splitted[0].rsplit('/').collect();

        for s in slash_splitted {
            for i in 0..s.len() {
                let c = s.as_bytes()[i] as char;
                if c.is_alphabetic() {
                    match c {
                        'P' => {
                            pieces[0].set_bit(square);
                            piece_on[square as usize] = 0;
                            hash ^= zobrist::keys()[0 * 64 + square as usize]
                        }
                        'N' => {
                            pieces[1].set_bit(square);
                            piece_on[square as usize] = 1;
                            hash ^= zobrist::keys()[1 * 64 + square as usize]
                        }
                        'B' => {
                            pieces[2].set_bit(square);
                            piece_on[square as usize] = 2;
                            hash ^= zobrist::keys()[2 * 64 + square as usize]
                        }
                        'R' => {
                            pieces[3].set_bit(square);
                            piece_on[square as usize] = 3;
                            hash ^= zobrist::keys()[3 * 64 + square as usize]
                        }
                        'Q' => {
                            pieces[4].set_bit(square);
                            piece_on[square as usize] = 4;
                            hash ^= zobrist::keys()[4 * 64 + square as usize]
                        }
                        'K' => {
                            pieces[5].set_bit(square);
                            piece_on[square as usize] = 5;
                            hash ^= zobrist::keys()[5 * 64 + square as usize]
                        }
                        'p' => {
                            pieces[6].set_bit(square);
                            piece_on[square as usize] = 6;
                            hash ^= zobrist::keys()[6 * 64 + square as usize]
                        }
                        'n' => {
                            pieces[7].set_bit(square);
                            piece_on[square as usize] = 7;
                            hash ^= zobrist::keys()[7 * 64 + square as usize]
                        }
                        'b' => {
                            pieces[8].set_bit(square);
                            piece_on[square as usize] = 8;
                            hash ^= zobrist::keys()[8 * 64 + square as usize]
                        }
                        'r' => {
                            pieces[9].set_bit(square);
                            piece_on[square as usize] = 9;
                            hash ^= zobrist::keys()[9 * 64 + square as usize]
                        }
                        'q' => {
                            pieces[10].set_bit(square);
                            piece_on[square as usize] = 10;
                            hash ^= zobrist::keys()[10 * 64 + square as usize]
                        }
                        'k' => {
                            pieces[11].set_bit(square);
                            piece_on[square as usize] = 11;
                            hash ^= zobrist::keys()[11 * 64 + square as usize]
                        }
                        _ => println!("FEN parsing error"),
                    }
                    square += 1;
                } else {
                    square += c as u8 - b'0';
                }
            }
        }

        let side_to_move = match space_splitted[1] {
            "b" => {
                hash ^= zobrist::keys()[768];
                Color::Black
            }
            _ => Color::White,
        };

        let mut castling = 0u8;
        if space_splitted[2].contains('K') {
            castling ^= 0b00000001;
            hash ^= zobrist::keys()[769]
        }
        if space_splitted[2].contains('Q') {
            castling ^= 0b00000010;
            hash ^= zobrist::keys()[770];
        }
        if space_splitted[2].contains('k') {
            castling ^= 0b00000100;
            hash ^= zobrist::keys()[771];
        }
        if space_splitted[2].contains('q') {
            castling ^= 0b00001000;
            hash ^= zobrist::keys()[772];
        }

        let en_passant = if space_splitted[3] == "-" {
            64
        } else {
            (space_splitted[3].as_bytes()[0] as u8 - b'a'
                + (space_splitted[3].as_bytes()[1] as u8 - b'1') * 8) as u8
        };
        if en_passant != 64 {
            hash ^= zobrist::keys()[773 + (en_passant % 8) as usize];
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
            pieces: pieces,
            piece_on: piece_on,
            side_to_move: side_to_move,
            castling: castling,
            en_passant: en_passant,
            hash: hash,
            occ: occ,
            occ_by: occ_by,
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

    pub fn make_move(&mut self, mv: Move) -> UndoInfo {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flags();
        let color = self.side_to_move;
        let ep_captured_sq = if flag == MoveFlags::EN_PASSANT {
            to + color as u8 * 16 - 8
        } else {
            to
        };
        let undo = UndoInfo {
            moving_piece: self.piece_on[from as usize],
            captured: self.piece_on[ep_captured_sq as usize],
            captured_sq: ep_captured_sq,
            en_passant: self.en_passant,
            castling: self.castling,
            hash: self.hash,
            occ: self.occ,
            occ_by: self.occ_by,
        };

        self.en_passant = 64;
        let opponent = self.opponent();
        let piece_idx = self.piece_on[from as usize] as usize;
        let opponent_idx = self.piece_on[to as usize] as usize;
        self.castling &= CASTLING_UPDATE[from as usize];
        self.castling &= CASTLING_UPDATE[to as usize];
        match flag {
            MoveFlags::QUIET => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[piece_idx].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
            }
            MoveFlags::CAPTURE => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[opponent_idx].clear_bit(to);
                self.pieces[piece_idx].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
                self.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::KNIGHT_PROMOTION => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[(color as usize) * 6 + 1].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 1;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 1) * 64 + to as usize];
            }
            MoveFlags::BISHOP_PROMOTION => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[(color as usize) * 6 + 2].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 2;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 2) * 64 + to as usize];
            }
            MoveFlags::ROOK_PROMOTION => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[(color as usize) * 6 + 3].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 3;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 3) * 64 + to as usize];
            }
            MoveFlags::QUEEN_PROMOTION => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[(color as usize) * 6 + 4].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 4;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 4) * 64 + to as usize];
            }
            MoveFlags::KNIGHT_PROMOTION_CAPTURE => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[opponent_idx].clear_bit(to);
                self.pieces[(color as usize) * 6 + 1].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 1;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 1) * 64 + to as usize];
                self.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::BISHOP_PROMOTION_CAPTURE => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[opponent_idx].clear_bit(to);
                self.pieces[(color as usize) * 6 + 2].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 2;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 2) * 64 + to as usize];
                self.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::ROOK_PROMOTION_CAPTURE => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[opponent_idx].clear_bit(to);
                self.pieces[(color as usize) * 6 + 3].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 3;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 3) * 64 + to as usize];
                self.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::QUEEN_PROMOTION_CAPTURE => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[opponent_idx].clear_bit(to);
                self.pieces[(color as usize) * 6 + 4].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = (color as u8) * 6 + 4;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[((color as usize) * 6 + 4) * 64 + to as usize];
                self.hash ^= zobrist::keys()[opponent_idx * 64 + to as usize];
            }
            MoveFlags::KINGSIDE_CASTLE => {
                self.pieces[(color as usize * 6) + 5].clear_bit((color as u8) * 56 + 4);
                self.pieces[(color as usize * 6) + 5].set_bit((color as u8) * 56 + 6);
                self.pieces[(color as usize * 6) + 3].clear_bit((color as u8) * 56 + 7);
                self.pieces[(color as usize * 6) + 3].set_bit((color as u8) * 56 + 5);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.piece_on[(color as usize) * 56 + 7] = 64;
                self.piece_on[(color as usize) * 56 + 5] = (color as u8 * 6) + 3;
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 4) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 6) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 7) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 5) as usize];
            }
            MoveFlags::QUEENSIDE_CASTLE => {
                self.pieces[(color as usize * 6) + 5].clear_bit((color as u8) * 56 + 4);
                self.pieces[(color as usize * 6) + 5].set_bit((color as u8) * 56 + 2);
                self.pieces[(color as usize * 6) + 3].clear_bit((color as u8) * 56 + 0);
                self.pieces[(color as usize * 6) + 3].set_bit((color as u8) * 56 + 3);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.piece_on[(color as usize) * 56 + 0] = 64;
                self.piece_on[(color as usize) * 56 + 3] = (color as u8 * 6) + 3;
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 4) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 5) * 64 + ((color as u8) * 56 + 2) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 0) as usize];
                self.hash ^= zobrist::keys()
                    [((color as usize) * 6 + 3) * 64 + ((color as u8) * 56 + 3) as usize];
            }
            MoveFlags::EN_PASSANT => {
                self.pieces[(color as usize) * 6 + 0].clear_bit(from);
                self.pieces[(1 - (color as usize)) * 6 + 0].clear_bit(to + (color as u8) * 16 - 8);
                self.pieces[(color as usize) * 6 + 0].set_bit(to);
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.piece_on[to as usize + (color as usize) * 16 - 8] = 64;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
                self.hash ^= zobrist::keys()
                    [(1 - color as usize) * 6 * 64 + (to as usize + (color as usize) * 16 - 8)];
            }
            MoveFlags::DOUBLE_PAWN_PUSH => {
                self.pieces[piece_idx].clear_bit(from);
                self.pieces[piece_idx].set_bit(to);
                self.en_passant = from + 8 - (color as u8) * 16;
                self.piece_on[from as usize] = 64;
                self.piece_on[to as usize] = piece_idx as u8;
                self.hash ^= zobrist::keys()[piece_idx * 64 + from as usize];
                self.hash ^= zobrist::keys()[piece_idx * 64 + to as usize];
            }
            6_u8..=7_u8 | 16_u8..=u8::MAX => {}
        }

        self.hash ^= zobrist::keys()[768];
        for i in 0..4 {
            if (undo.castling >> i) & 1 != 0 {
                self.hash ^= zobrist::keys()[769 + i];
            }
        }
        for i in 0..4 {
            if (self.castling >> i) & 1 != 0 {
                self.hash ^= zobrist::keys()[769 + i];
            }
        }
        if undo.en_passant != 64 {
            self.hash ^= zobrist::keys()[773 + (undo.en_passant % 8) as usize];
        }
        if self.en_passant != 64 {
            self.hash ^= zobrist::keys()[773 + (self.en_passant % 8) as usize];
        }

        self.occ.clear_bit(from);
        self.occ_by[color as usize].clear_bit(from);
        if undo.captured != 64 {
            self.occ_by[opponent as usize].clear_bit(undo.captured_sq);
        }
        if undo.captured_sq != to {
            self.occ.clear_bit(undo.captured_sq);
        }
        self.occ.set_bit(to);
        self.occ_by[color as usize].set_bit(to);
        if flag == MoveFlags::KINGSIDE_CASTLE {
            self.occ.clear_bit(color as u8 * 56 + 7);
            self.occ.set_bit(color as u8 * 56 + 5);
            self.occ_by[color as usize].clear_bit(color as u8 * 56 + 7);
            self.occ_by[color as usize].set_bit(color as u8 * 56 + 5);
        }
        if flag == MoveFlags::QUEENSIDE_CASTLE {
            self.occ.clear_bit(color as u8 * 56 + 0);
            self.occ.set_bit(color as u8 * 56 + 3);
            self.occ_by[color as usize].clear_bit(color as u8 * 56 + 0);
            self.occ_by[color as usize].set_bit(color as u8 * 56 + 3);
        }

        self.side_to_move = opponent;
        undo
    }

    pub fn unmake_move(&mut self, mv: Move, undo: UndoInfo) {
        self.side_to_move = self.opponent();
        self.en_passant = undo.en_passant;
        self.castling = undo.castling;
        self.hash = undo.hash;
        let color = self.side_to_move;
        let from = mv.from();
        let to = mv.to();
        self.pieces[undo.moving_piece as usize].set_bit(from);
        self.pieces[self.piece_on[to as usize] as usize].clear_bit(to);
        self.piece_on[from as usize] = undo.moving_piece;
        self.piece_on[to as usize] = 64;
        if undo.captured != 64 {
            self.pieces[undo.captured as usize].set_bit(undo.captured_sq);
            self.piece_on[undo.captured_sq as usize] = undo.captured;
        }
        let flag = mv.flags();
        if flag == MoveFlags::KINGSIDE_CASTLE {
            self.pieces[color as usize * 6 + 3].clear_bit(color as u8 * 56 + 5);
            self.pieces[color as usize * 6 + 3].set_bit(color as u8 * 56 + 7);
            self.piece_on[color as usize * 56 + 5] = 64;
            self.piece_on[color as usize * 56 + 7] = color as u8 * 6 + 3;
        }
        if flag == MoveFlags::QUEENSIDE_CASTLE {
            self.pieces[color as usize * 6 + 3].clear_bit(color as u8 * 56 + 3);
            self.pieces[color as usize * 6 + 3].set_bit(color as u8 * 56 + 0);
            self.piece_on[color as usize * 56 + 3] = 64;
            self.piece_on[color as usize * 56 + 0] = color as u8 * 6 + 3;
        }

        self.occ = undo.occ;
        self.occ_by = undo.occ_by;
    }

    pub fn all_legal_moves(&mut self, table: &MagicTable, list: &mut MoveList) {
        MoveGen::generate_legal_moves(self, table, list)
    }

    pub fn all_legal_captures(&mut self, table: &MagicTable, list: &mut MoveList) {
        MoveGen::generate_legal_captures(self, table, list)
    }

    pub fn all_moves(&mut self, table: &MagicTable, list: &mut MoveList) {
        MoveGen::generate_moves(self, table, list)
    }

    pub fn all_captures(&mut self, table: &MagicTable, list: &mut MoveList) {
        MoveGen::generate_captures(self, table, list)
    }

    pub fn king_under_attack(&self, table: &MagicTable) -> bool {
        MoveGen::is_attacked(
            self.side_to_move,
            self.pieces[(self.side_to_move as u8 * 6 + 5) as usize]
                .0
                .trailing_zeros() as u8,
            &self,
            table,
        )
    }
}
