use std::usize;

use crate::{
    eval::evaluate_for_white,
    movegen::{MagicTable, Move, MoveFlags},
    position::{Color, PieceType, Position},
};

pub fn negamax(
    position: Position,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
) -> i32 {
    if depth == 0 {
        return evaluate_for_white(&position)
            * if position.side_to_move == Color::White {
                1
            } else {
                -1
            };
    }
    let mut legal_moves = position.all_moves(table);
    if legal_moves.is_empty() {
        if position.king_under_attack(table) {
            return -1000000;
        } else {
            return 0;
        }
    }
    legal_moves.sort_by_key(|mv| -move_score(&position, mv));
    for mv in legal_moves {
        let score = -negamax(position.make_move(mv), depth - 1, -beta, -alpha, table);
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    alpha
}

pub fn best_move(position: Position, depth: u32, table: &MagicTable) -> Option<Move> {
    if depth == 0 {
        return None;
    }
    let mut result: Option<Move> = None;
    let mut legal_moves = position.all_moves(table);
    let mut highest_score = -1000000;
    legal_moves.sort_by_key(|mv| -move_score(&position, mv));
    for mv in legal_moves {
        let score = -negamax(
            position.make_move(mv),
            depth - 1,
            -1000000i32,
            -highest_score,
            table,
        );
        if score > highest_score {
            highest_score = score;
            result = Some(mv);
        }
    }
    result
}

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub fn move_score(position: &Position, mv: &Move) -> i32 {
    let base = 10000i32;
    let from = mv.from();
    let to = mv.to();
    match mv.flags() {
        MoveFlags::CAPTURE => {
            base + PIECE_VALUES[(position.piece_on[to as usize] % 6) as usize]
                - PIECE_VALUES[(position.piece_on[from as usize] % 6) as usize] as i32
        }
        MoveFlags::KNIGHT_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(position.piece_on[to as usize] % 6) as usize]
                - PIECE_VALUES[0] as i32
        }
        MoveFlags::BISHOP_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(position.piece_on[to as usize] % 6) as usize]
                - PIECE_VALUES[0] as i32
        }
        MoveFlags::ROOK_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(position.piece_on[to as usize] % 6) as usize]
                - PIECE_VALUES[0] as i32
        }
        MoveFlags::QUEEN_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(position.piece_on[to as usize] % 6) as usize]
                - PIECE_VALUES[0] as i32
        }
        MoveFlags::EN_PASSANT => base + PIECE_VALUES[0] - PIECE_VALUES[0] as i32,
        _ => 0i32,
    }
}
