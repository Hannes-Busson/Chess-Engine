use crate::{
    eval::evaluate_for_white,
    movegen::{MagicTable, Move, MoveFlags},
    position::{Color, Position},
    tt::{self, TransponationTable},
    zobrist,
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEGAMAX_CALLS: AtomicU64 = AtomicU64::new(0);
static TT_HITS: AtomicU64 = AtomicU64::new(0);
static TT_COLLISIONS: AtomicU64 = AtomicU64::new(0);
static BETA_CUTOFFS: AtomicU64 = AtomicU64::new(0);
static NULL_MOVE_CUTOFFS: AtomicU64 = AtomicU64::new(0);
static QUIESCENCE_CALLS: AtomicU64 = AtomicU64::new(0);

pub fn negamax(
    position: Position,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
) -> i32 {
    NEGAMAX_CALLS.fetch_add(1, Ordering::Relaxed);
    if depth == 0 {
        return quienscence(position, alpha, beta, table, 4);
    }
    if t_table.vault[position.hash as usize & tt::SHIFT].hash != position.hash {
        TT_COLLISIONS.fetch_add(1, Ordering::Relaxed);
    }
    if let Some(t) = t_table.lookup(position.hash, depth as u8, alpha, beta) {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return t;
    }
    if depth > 2 && !position.king_under_attack(table) {
        let mut null_pos = position;
        null_pos.side_to_move = position.opponent();
        null_pos.en_passant = 64;
        if position.en_passant != 64 {
            null_pos.hash ^= zobrist::keys()[773 + (position.en_passant % 8) as usize];
        }
        null_pos.hash ^= zobrist::keys()[768];
        if -negamax(null_pos, depth - 3, -beta, -beta + 1, table, t_table) >= beta {
            NULL_MOVE_CUTOFFS.fetch_add(1, Ordering::Relaxed);
            return beta;
        }
    }
    let mut legal_moves = position.all_moves(table);
    if legal_moves.is_empty() {
        if position.king_under_attack(table) {
            return -1000000;
        } else {
            return 0;
        }
    }
    let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
    legal_moves.sort_by_key(|mv| -move_score(&position, mv, tt_move));
    let original_alpha = alpha;
    let mut best_move = 0;
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
            best_move = mv.value;
        }
        if alpha >= beta {
            BETA_CUTOFFS.fetch_add(1, Ordering::Relaxed);
            break;
        }
    }
    if alpha >= beta {
        t_table.store(position.hash, alpha, depth as u8, 2, best_move);
    } else if alpha > original_alpha {
        t_table.store(position.hash, alpha, depth as u8, 0, best_move);
    } else {
        t_table.store(position.hash, alpha, depth as u8, 1, best_move);
    }
    alpha
}

pub fn best_move(
    position: Position,
    depth: u32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
) -> Option<Move> {
    let mut result: Option<Move> = None;
    let mut legal_moves = position.all_moves(table);
    for d in 1..depth + 1 {
        let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
        legal_moves.sort_by_key(|mv| -move_score(&position, mv, tt_move));
        let mut partial_result = None;
        let mut highest_score = -1000000;
        for mv in &legal_moves {
            let score = -negamax(
                position.make_move(*mv),
                d - 1,
                -1000000i32,
                -highest_score,
                table,
                t_table,
            );
            if score > highest_score {
                highest_score = score;
                partial_result = Some(*mv);
            }
        }
        if let Some(mv) = partial_result {
            t_table.store(position.hash, highest_score, d as u8, 0, mv.value);
        }
        result = partial_result;
    }
    eprintln!("{}", NEGAMAX_CALLS.load(Ordering::Relaxed));
    eprintln!("{}", TT_HITS.load(Ordering::Relaxed));
    eprintln!("{}", TT_COLLISIONS.load(Ordering::Relaxed));
    eprintln!("{}", BETA_CUTOFFS.load(Ordering::Relaxed));
    eprintln!("{}", NULL_MOVE_CUTOFFS.load(Ordering::Relaxed));
    eprintln!("{}", QUIESCENCE_CALLS.load(Ordering::Relaxed));
    result
}

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub fn move_score(position: &Position, mv: &Move, tt_move: u16) -> i32 {
    if mv.value == tt_move {
        return 20000;
    }
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

pub fn quienscence(
    position: Position,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    qdepth: i32,
) -> i32 {
    QUIESCENCE_CALLS.fetch_add(1, Ordering::Relaxed);
    let stand_pat = evaluate_for_white(&position)
        * if position.side_to_move == Color::White {
            1
        } else {
            -1
        };
    if qdepth <= 0 {
        return stand_pat;
    }
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let all_captures = position.all_captures(table);
    for c in all_captures {
        let score = -quienscence(position.make_move(c), -beta, -alpha, table, qdepth - 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}
