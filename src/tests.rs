#[cfg(test)]
mod tests {
    use crate::bitboard::Bitboard;
    use crate::movegen::{MagicTable, Move, MoveFlags, MoveGen};
    use crate::position::{CastlingRights, Color, PieceType, Position};

    fn empty(side: Color) -> Position {
        Position {
            pieces: [Bitboard(0); 12],
            piece_on: [64u8; 64],
            side_to_move: side,
            castling: CastlingRights::NONE,
            en_passant: 64,
        }
    }

    fn place(pos: &mut Position, color: Color, piece: PieceType, sq: u8) {
        let idx = color as usize * 6 + piece as usize;
        pos.pieces[idx].set_bit(sq);
        pos.piece_on[sq as usize] = idx as u8;
    }

    fn has(moves: &[Move], from: u8, to: u8, flag: u8) -> bool {
        moves
            .iter()
            .any(|m| m.from() == from && m.to() == to && m.flags() == flag)
    }

    // file: 0=a..7=h  rank: 0=rank1..7=rank8
    fn sq(file: u8, rank: u8) -> u8 {
        rank * 8 + file
    }

    fn magic() -> MagicTable {
        MagicTable::init()
    }

    // ── Starting position ────────────────────────────────────────────────────

    #[test]
    fn starting_position_move_count() {
        let moves = MoveGen::generate_moves(Position::start(), &magic());
        assert_eq!(moves.len(), 20, "expected 20, got {}", moves.len());
    }

    // ── Pawn pushes ──────────────────────────────────────────────────────────

    #[test]
    fn white_pawn_single_and_double_push() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 1)); // e2
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 1), sq(4, 2), MoveFlags::QUIET),
            "e2-e3 missing"
        );
        assert!(
            has(&moves, sq(4, 1), sq(4, 3), MoveFlags::DOUBLE_PAWN_PUSH),
            "e2-e4 missing"
        );
    }

    #[test]
    fn black_pawn_single_and_double_push() {
        let mut pos = empty(Color::Black);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::Black, PieceType::Pawn, sq(4, 6)); // e7
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 6), sq(4, 5), MoveFlags::QUIET),
            "e7-e6 missing"
        );
        assert!(
            has(&moves, sq(4, 6), sq(4, 4), MoveFlags::DOUBLE_PAWN_PUSH),
            "e7-e5 missing"
        );
    }

    #[test]
    fn pawn_blocked_at_rank3_no_pushes() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 1)); // e2
        place(&mut pos, Color::Black, PieceType::Pawn, sq(4, 2)); // e3 blocks
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            !moves.iter().any(|m| m.from() == sq(4, 1)),
            "blocked pawn should have no pushes"
        );
    }

    #[test]
    fn pawn_rank4_blocked_allows_single_only() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 1)); // e2
        place(&mut pos, Color::Black, PieceType::Pawn, sq(4, 3)); // e4 blocks double push
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 1), sq(4, 2), MoveFlags::QUIET),
            "e2-e3 should be allowed"
        );
        assert!(
            !moves
                .iter()
                .any(|m| m.from() == sq(4, 1) && m.to() == sq(4, 3)),
            "e2-e4 should be blocked"
        );
    }

    // ── Pawn captures ────────────────────────────────────────────────────────

    #[test]
    fn white_pawn_captures() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 3)); // e4
        place(&mut pos, Color::Black, PieceType::Pawn, sq(3, 4)); // d5
        place(&mut pos, Color::Black, PieceType::Pawn, sq(5, 4)); // f5
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 3), sq(3, 4), MoveFlags::CAPTURE),
            "e4xd5 missing"
        );
        assert!(
            has(&moves, sq(4, 3), sq(5, 4), MoveFlags::CAPTURE),
            "e4xf5 missing"
        );
    }

    // ── En passant ───────────────────────────────────────────────────────────

    #[test]
    fn en_passant_white_generate_and_make() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 4)); // e5
        place(&mut pos, Color::Black, PieceType::Pawn, sq(3, 4)); // d5
        pos.en_passant = sq(3, 5); // d6
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 4), sq(3, 5), MoveFlags::EN_PASSANT),
            "e5xd6 ep missing"
        );
        let ep_move = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::EN_PASSANT)
            .unwrap();
        let new_pos = pos.make_move(ep_move);
        assert!(
            !new_pos
                .get_piece_bitboard(Color::Black, PieceType::Pawn)
                .get_bit(sq(3, 4)),
            "black pawn d5 should be removed"
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::Pawn)
                .get_bit(sq(3, 5)),
            "white pawn should be on d6"
        );
        assert_eq!(
            new_pos.piece_on[sq(3, 4) as usize],
            64,
            "piece_on d5 should be empty"
        );
        assert_eq!(
            new_pos.piece_on[sq(3, 5) as usize],
            0 * 6 + 0,
            "piece_on d6 should be white pawn"
        );
    }

    #[test]
    fn en_passant_black_generate_and_make() {
        let mut pos = empty(Color::Black);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::Black, PieceType::Pawn, sq(3, 3)); // d4
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 3)); // e4
        pos.en_passant = sq(4, 2); // e3
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(3, 3), sq(4, 2), MoveFlags::EN_PASSANT),
            "d4xe3 ep missing"
        );
        let ep_move = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::EN_PASSANT)
            .unwrap();
        let new_pos = pos.make_move(ep_move);
        assert!(
            !new_pos
                .get_piece_bitboard(Color::White, PieceType::Pawn)
                .get_bit(sq(4, 3)),
            "white pawn e4 should be removed"
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::Black, PieceType::Pawn)
                .get_bit(sq(4, 2)),
            "black pawn should be on e3"
        );
        assert_eq!(
            new_pos.piece_on[sq(4, 3) as usize],
            64,
            "piece_on e4 should be empty"
        );
    }

    #[test]
    fn white_double_push_sets_ep_square() {
        let pos = Position::start();
        let moves = MoveGen::generate_moves(pos, &magic());
        let dpp = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::DOUBLE_PAWN_PUSH && m.from() == sq(4, 1))
            .unwrap();
        let new_pos = pos.make_move(dpp);
        assert_eq!(
            new_pos.en_passant,
            sq(4, 2),
            "ep square after e2-e4 should be e3"
        );
    }

    #[test]
    fn black_double_push_sets_ep_square() {
        let mut pos = empty(Color::Black);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::Black, PieceType::Pawn, sq(4, 6)); // e7
        let moves = MoveGen::generate_moves(pos, &magic());
        let dpp = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::DOUBLE_PAWN_PUSH)
            .unwrap();
        let new_pos = pos.make_move(dpp);
        assert_eq!(
            new_pos.en_passant,
            sq(4, 5),
            "ep square after e7-e5 should be e6"
        );
    }

    // ── Promotions ───────────────────────────────────────────────────────────

    #[test]
    fn white_quiet_promotion_four_moves() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 6)); // e7
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(has(&moves, sq(4, 6), sq(4, 7), MoveFlags::KNIGHT_PROMOTION));
        assert!(has(&moves, sq(4, 6), sq(4, 7), MoveFlags::BISHOP_PROMOTION));
        assert!(has(&moves, sq(4, 6), sq(4, 7), MoveFlags::ROOK_PROMOTION));
        assert!(has(&moves, sq(4, 6), sq(4, 7), MoveFlags::QUEEN_PROMOTION));
        let count = moves.iter().filter(|m| m.from() == sq(4, 6)).count();
        assert_eq!(
            count, 4,
            "exactly 4 promotion moves expected, got {}",
            count
        );
    }

    #[test]
    fn white_promotion_capture_four_moves() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 6)); // e7
        place(&mut pos, Color::Black, PieceType::Rook, sq(5, 7)); // f8
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(has(
            &moves,
            sq(4, 6),
            sq(5, 7),
            MoveFlags::KNIGHT_PROMOTION_CAPTURE
        ));
        assert!(has(
            &moves,
            sq(4, 6),
            sq(5, 7),
            MoveFlags::BISHOP_PROMOTION_CAPTURE
        ));
        assert!(has(
            &moves,
            sq(4, 6),
            sq(5, 7),
            MoveFlags::ROOK_PROMOTION_CAPTURE
        ));
        assert!(has(
            &moves,
            sq(4, 6),
            sq(5, 7),
            MoveFlags::QUEEN_PROMOTION_CAPTURE
        ));
    }

    #[test]
    fn make_move_queen_promotion() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Pawn, sq(4, 6)); // e7
        let moves = MoveGen::generate_moves(pos, &magic());
        let qp = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::QUEEN_PROMOTION)
            .unwrap();
        let new_pos = pos.make_move(qp);
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::Queen)
                .get_bit(sq(4, 7)),
            "white queen should be on e8"
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::White, PieceType::Pawn)
                .get_bit(sq(4, 6)),
            "pawn should be gone from e7"
        );
        assert_eq!(
            new_pos.piece_on[sq(4, 7) as usize],
            0 * 6 + 4,
            "piece_on e8 should be white queen"
        );
    }

    #[test]
    fn black_quiet_promotion_four_moves() {
        let mut pos = empty(Color::Black);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::Black, PieceType::Pawn, sq(4, 1)); // e2
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(has(&moves, sq(4, 1), sq(4, 0), MoveFlags::KNIGHT_PROMOTION));
        assert!(has(&moves, sq(4, 1), sq(4, 0), MoveFlags::BISHOP_PROMOTION));
        assert!(has(&moves, sq(4, 1), sq(4, 0), MoveFlags::ROOK_PROMOTION));
        assert!(has(&moves, sq(4, 1), sq(4, 0), MoveFlags::QUEEN_PROMOTION));
    }

    // ── Castling ─────────────────────────────────────────────────────────────

    #[test]
    fn white_kingside_castle_generate_and_make() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::WK;
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::White, PieceType::Rook, sq(7, 0)); // h1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 0), sq(6, 0), MoveFlags::KINGSIDE_CASTLE),
            "kingside castle missing"
        );
        let castle = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::KINGSIDE_CASTLE)
            .unwrap();
        let new_pos = pos.make_move(castle);
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::King)
                .get_bit(sq(6, 0)),
            "king should be on g1"
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::Rook)
                .get_bit(sq(5, 0)),
            "rook should be on f1"
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::White, PieceType::King)
                .get_bit(sq(4, 0))
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::White, PieceType::Rook)
                .get_bit(sq(7, 0))
        );
        assert_eq!(new_pos.piece_on[sq(4, 0) as usize], 64);
        assert_eq!(new_pos.piece_on[sq(7, 0) as usize], 64);
        assert_eq!(
            new_pos.castling & (CastlingRights::WK | CastlingRights::WQ),
            0,
            "white castling rights should be cleared"
        );
    }

    #[test]
    fn white_queenside_castle_generate_and_make() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::WQ;
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::White, PieceType::Rook, sq(0, 0)); // a1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 0), sq(2, 0), MoveFlags::QUEENSIDE_CASTLE),
            "queenside castle missing"
        );
        let castle = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::QUEENSIDE_CASTLE)
            .unwrap();
        let new_pos = pos.make_move(castle);
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::King)
                .get_bit(sq(2, 0))
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::White, PieceType::Rook)
                .get_bit(sq(3, 0))
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::White, PieceType::Rook)
                .get_bit(sq(0, 0))
        );
        assert_eq!(new_pos.piece_on[sq(0, 0) as usize], 64);
    }

    #[test]
    fn black_kingside_castle_generate_and_make() {
        let mut pos = empty(Color::Black);
        pos.castling = CastlingRights::BK;
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7)); // e8
        place(&mut pos, Color::Black, PieceType::Rook, sq(7, 7)); // h8
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 7), sq(6, 7), MoveFlags::KINGSIDE_CASTLE),
            "black kingside castle missing"
        );
        let castle = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::KINGSIDE_CASTLE)
            .unwrap();
        let new_pos = pos.make_move(castle);
        assert!(
            new_pos
                .get_piece_bitboard(Color::Black, PieceType::King)
                .get_bit(sq(6, 7))
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::Black, PieceType::Rook)
                .get_bit(sq(5, 7))
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::Black, PieceType::Rook)
                .get_bit(sq(7, 7))
        );
    }

    #[test]
    fn black_queenside_castle_generate_and_make() {
        let mut pos = empty(Color::Black);
        pos.castling = CastlingRights::BQ;
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7)); // e8
        place(&mut pos, Color::Black, PieceType::Rook, sq(0, 7)); // a8
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            has(&moves, sq(4, 7), sq(2, 7), MoveFlags::QUEENSIDE_CASTLE),
            "black queenside castle missing"
        );
        let castle = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::QUEENSIDE_CASTLE)
            .unwrap();
        let new_pos = pos.make_move(castle);
        assert!(
            new_pos
                .get_piece_bitboard(Color::Black, PieceType::King)
                .get_bit(sq(2, 7))
        );
        assert!(
            new_pos
                .get_piece_bitboard(Color::Black, PieceType::Rook)
                .get_bit(sq(3, 7))
        );
        assert_eq!(
            new_pos.piece_on[sq(0, 7) as usize],
            64,
            "a8 should be empty"
        );
    }

    #[test]
    fn castle_blocked_by_piece_in_path() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::WK;
        place(&mut pos, Color::White, PieceType::King, sq(4, 0));
        place(&mut pos, Color::White, PieceType::Rook, sq(7, 0));
        place(&mut pos, Color::White, PieceType::Bishop, sq(5, 0)); // f1 blocks
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            !moves
                .iter()
                .any(|m| m.flags() == MoveFlags::KINGSIDE_CASTLE),
            "castle should be blocked by piece on f1"
        );
    }

    #[test]
    fn castle_blocked_by_attacked_square() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::WK;
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::White, PieceType::Rook, sq(7, 0)); // h1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        place(&mut pos, Color::Black, PieceType::Rook, sq(6, 7)); // g8 attacks g1
        let moves = MoveGen::generate_moves(pos, &magic());
        assert!(
            !moves
                .iter()
                .any(|m| m.flags() == MoveFlags::KINGSIDE_CASTLE),
            "castle should be blocked: g1 is attacked"
        );
    }

    // ── Castling rights ──────────────────────────────────────────────────────

    #[test]
    fn castling_rights_cleared_after_king_move() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::ALL;
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        let moves = MoveGen::generate_moves(pos, &magic());
        let km = *moves
            .iter()
            .find(|m| m.from() == sq(4, 0) && m.flags() == MoveFlags::QUIET)
            .unwrap();
        let new_pos = pos.make_move(km);
        assert_eq!(
            new_pos.castling & (CastlingRights::WK | CastlingRights::WQ),
            0,
            "white castling rights should be cleared after king move"
        );
    }

    #[test]
    fn castling_rights_cleared_after_rook_h1_move() {
        let mut pos = empty(Color::White);
        pos.castling = CastlingRights::WK;
        place(&mut pos, Color::White, PieceType::King, sq(0, 0)); // a1
        place(&mut pos, Color::White, PieceType::Rook, sq(7, 0)); // h1
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        let moves = MoveGen::generate_moves(pos, &magic());
        let rm = *moves
            .iter()
            .find(|m| m.from() == sq(7, 0) && m.flags() == MoveFlags::QUIET)
            .unwrap();
        let new_pos = pos.make_move(rm);
        assert_eq!(
            new_pos.castling & CastlingRights::WK,
            0,
            "WK should be cleared after h1 rook moves"
        );
    }

    // ── Captures ─────────────────────────────────────────────────────────────

    #[test]
    fn make_move_capture_updates_piece_on() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0));
        place(&mut pos, Color::Black, PieceType::King, sq(7, 7));
        place(&mut pos, Color::White, PieceType::Rook, sq(0, 3)); // a4
        place(&mut pos, Color::Black, PieceType::Pawn, sq(0, 6)); // a7
        let moves = MoveGen::generate_moves(pos, &magic());
        let cap = *moves
            .iter()
            .find(|m| m.flags() == MoveFlags::CAPTURE && m.from() == sq(0, 3))
            .unwrap();
        let new_pos = pos.make_move(cap);
        assert_eq!(
            new_pos.piece_on[sq(0, 6) as usize],
            0 * 6 + 3,
            "a7 should have white rook"
        );
        assert_eq!(
            new_pos.piece_on[sq(0, 3) as usize],
            64,
            "a4 should be empty"
        );
        assert!(
            !new_pos
                .get_piece_bitboard(Color::Black, PieceType::Pawn)
                .get_bit(sq(0, 6)),
            "black pawn a7 should be gone"
        );
    }

    // ── Attack detection ─────────────────────────────────────────────────────

    #[test]
    fn is_attacked_by_knight() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        place(&mut pos, Color::Black, PieceType::Knight, sq(5, 2)); // f3 attacks e1
        assert!(
            MoveGen::is_attacked(sq(4, 0), &pos, &magic()),
            "e1 should be attacked by knight on f3"
        );
        assert!(
            !MoveGen::is_attacked(sq(3, 0), &pos, &magic()),
            "d1 should not be attacked"
        );
    }

    #[test]
    fn is_attacked_by_pawn() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        place(&mut pos, Color::Black, PieceType::Pawn, sq(3, 1)); // d2 attacks e1
        assert!(
            MoveGen::is_attacked(sq(4, 0), &pos, &magic()),
            "e1 should be attacked by pawn on d2"
        );
    }

    #[test]
    fn is_attacked_by_bishop() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        place(&mut pos, Color::Black, PieceType::Bishop, sq(1, 3)); // b4 attacks e1
        assert!(
            MoveGen::is_attacked(sq(4, 0), &pos, &magic()),
            "e1 should be attacked by bishop on b4"
        );
    }

    #[test]
    fn is_attacked_by_rook() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::Black, PieceType::King, sq(4, 7));
        place(&mut pos, Color::Black, PieceType::Rook, sq(4, 5)); // e6 attacks e1
        assert!(
            MoveGen::is_attacked(sq(4, 0), &pos, &magic()),
            "e1 should be attacked by rook on e6"
        );
    }

    // ── Sliding piece attack counts ───────────────────────────────────────────

    #[test]
    fn knight_attacks_center_has_8() {
        assert_eq!(MoveGen::knight_attacks(sq(3, 3)).0.count_ones(), 8);
    }

    #[test]
    fn knight_attacks_corner_has_2() {
        assert_eq!(MoveGen::knight_attacks(sq(0, 0)).0.count_ones(), 2);
    }

    #[test]
    fn bishop_attacks_d4_empty_board() {
        let attacks = MoveGen::bishop_attacks(sq(3, 3), Bitboard(0), &magic()); // d4
        assert_eq!(
            attacks.0.count_ones(),
            13,
            "bishop d4 empty board: expected 13, got {}",
            attacks.0.count_ones()
        );
    }

    #[test]
    fn rook_attacks_a1_empty_board() {
        let attacks = MoveGen::rook_attacks(sq(0, 0), Bitboard(0), &magic());
        assert_eq!(
            attacks.0.count_ones(),
            14,
            "rook a1 empty board: expected 14, got {}",
            attacks.0.count_ones()
        );
    }

    #[test]
    fn rook_blocker_stops_ray_but_includes_capture_square() {
        let mut occ = Bitboard(0);
        occ.set_bit(sq(0, 3)); // a4 is a blocker
        let attacks = MoveGen::rook_attacks(sq(0, 0), occ, &magic()); // rook on a1
        assert!(attacks.get_bit(sq(0, 1)), "a2 reachable");
        assert!(attacks.get_bit(sq(0, 2)), "a3 reachable");
        assert!(
            attacks.get_bit(sq(0, 3)),
            "a4 (blocker) included as capture target"
        );
        assert!(!attacks.get_bit(sq(0, 4)), "a5 should be cut off");
    }

    // ── Legal move generation ────────────────────────────────────────────────

    #[test]
    fn legal_moves_king_in_check_must_resolve() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::White, PieceType::Rook, sq(0, 0)); // a1
        place(&mut pos, Color::Black, PieceType::King, sq(3, 7)); // d8
        place(&mut pos, Color::Black, PieceType::Rook, sq(4, 6)); // e7 attacks e1
        let legal = MoveGen::generate_legal_moves(pos, &magic());
        for m in &legal {
            let new_pos = pos.make_move(*m);
            let mut king_bb = *new_pos.get_piece_bitboard(Color::White, PieceType::King);
            let king_sq = king_bb.pop_lsb();
            let mut check_pos = new_pos;
            check_pos.side_to_move = Color::White;
            assert!(
                !MoveGen::is_attacked(king_sq, &check_pos, &magic()),
                "legal move left king in check: from={} to={}",
                m.from(),
                m.to()
            );
        }
        assert!(!legal.is_empty(), "should have at least one legal move");
    }

    #[test]
    fn legal_moves_pinned_piece_cannot_expose_king() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(4, 0)); // e1
        place(&mut pos, Color::White, PieceType::Rook, sq(4, 3)); // e4 — pinned on e-file
        place(&mut pos, Color::Black, PieceType::King, sq(0, 7)); // a8
        place(&mut pos, Color::Black, PieceType::Rook, sq(4, 7)); // e8 pins e4 rook
        let legal = MoveGen::generate_legal_moves(pos, &magic());
        let pinned_moves: Vec<_> = legal.iter().filter(|m| m.from() == sq(4, 3)).collect();
        for m in &pinned_moves {
            assert_eq!(m.to() % 8, 4, "pinned rook must stay on e-file");
        }
    }

    #[test]
    fn legal_moves_checkmate_returns_empty() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 0)); // a1
        place(&mut pos, Color::Black, PieceType::King, sq(5, 5)); // f6
        place(&mut pos, Color::Black, PieceType::Rook, sq(0, 7)); // a8 — checks a1 via a-file
        place(&mut pos, Color::Black, PieceType::Rook, sq(1, 7)); // b8 — covers b-file
        let legal = MoveGen::generate_legal_moves(pos, &magic());
        assert_eq!(legal.len(), 0, "back-rank mate should have 0 legal moves");
    }

    #[test]
    fn legal_moves_stalemate_returns_empty() {
        let mut pos = empty(Color::White);
        place(&mut pos, Color::White, PieceType::King, sq(0, 7)); // a8
        place(&mut pos, Color::Black, PieceType::King, sq(2, 6)); // c7
        place(&mut pos, Color::Black, PieceType::Queen, sq(1, 5)); // b6
        let legal = MoveGen::generate_legal_moves(pos, &magic());
        assert_eq!(legal.len(), 0, "stalemate should have 0 legal moves");
    }
}
