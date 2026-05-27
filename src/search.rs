use std::usize;

use crate::{
    eval::evaluate_for_white,
    movegen::{MagicTable, Move, MoveFlags},
    position::{Color, PieceType, Position},
    tt::TransponationTable,
};

pub fn negamax(
    position: Position,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
) -> i32 {
    if depth == 0 {
        return quienscence(position, alpha, beta, table);
    }
    if let Some(t) = t_table.lookup(position.hash, depth as u8, alpha, beta) {
        return t;
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
    let original_alpha = alpha;
    for mv in legal_moves {
        let score = -negamax(
            position.make_move(mv),
            depth - 1,
            -beta,
            -alpha,
            table,
            t_table,
        );
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    if alpha >= beta {
        t_table.store(position.hash, alpha, depth as u8, 2);
    } else if alpha > original_alpha {
        t_table.store(position.hash, alpha, depth as u8, 0);
    } else {
        t_table.store(position.hash, alpha, depth as u8, 1);
    }
    alpha
}

pub fn best_move(
    position: Position,
    depth: u32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
) -> Option<Move> {
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
            t_table,
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
    let piece_on_to = position.piece_on[to as usize] % 6;
    match mv.flags() {
        MoveFlags::CAPTURE => {
            base + PIECE_VALUES[(piece_on_to) as usize]
                - PIECE_VALUES[(position.piece_on[from as usize] % 6) as usize] as i32
        }
        MoveFlags::KNIGHT_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(piece_on_to) as usize] - PIECE_VALUES[0] as i32
        }
        MoveFlags::BISHOP_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(piece_on_to) as usize] - PIECE_VALUES[0] as i32
        }
        MoveFlags::ROOK_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(piece_on_to) as usize] - PIECE_VALUES[0] as i32
        }
        MoveFlags::QUEEN_PROMOTION_CAPTURE => {
            base + PIECE_VALUES[(piece_on_to) as usize] - PIECE_VALUES[0] as i32
        }
        MoveFlags::EN_PASSANT => base + PIECE_VALUES[0] - PIECE_VALUES[0] as i32,
        _ => 0i32,
    }
}

pub fn quienscence(position: Position, mut alpha: i32, beta: i32, table: &MagicTable) -> i32 {
    let stand_pat = evaluate_for_white(&position)
        * if position.side_to_move == Color::White {
            1
        } else {
            -1
        };
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let all_captures = position
        .all_moves(table)
        .into_iter()
        .filter(|mv| mv.flags() >= 4 && mv.flags() <= 15);
    for c in all_captures {
        let score = -quienscence(position.make_move(c), -beta, -alpha, table);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}
