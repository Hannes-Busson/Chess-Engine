use crate::{
    eval::evaluate_for_white,
    movegen::{MagicTable, Move, MoveFlags, MoveList},
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
static STORE_CALLS: AtomicU64 = AtomicU64::new(0);

pub fn negamax(
    position: &mut Position,
    depth: u32,
    ply: u32,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
    killers: &mut [[u16; 2]; 64],
) -> i32 {
    NEGAMAX_CALLS.fetch_add(1, Ordering::Relaxed);
    if depth == 0 {
        return quienscence(position, alpha, beta, table, 4, t_table);
    }
    if t_table.vault[position.hash as usize & tt::SHIFT].hash != position.hash {
        TT_COLLISIONS.fetch_add(1, Ordering::Relaxed);
    }
    if let Some(t) = t_table.lookup(position.hash, depth as u8, alpha, beta) {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return t;
    }
    if depth > 2 && !position.king_under_attack(table) {
        let saved_side = position.side_to_move;
        let saved_ep = position.en_passant;
        let saved_hash = position.hash;
        position.side_to_move = position.opponent();
        position.en_passant = 64;
        if saved_ep != 64 {
            position.hash ^= zobrist::keys()[773 + (saved_ep % 8) as usize];
        }
        position.hash ^= zobrist::keys()[768];
        let null_score = -negamax(
            position,
            depth - 3,
            ply + 1,
            -beta,
            -beta + 1,
            table,
            t_table,
            killers,
        );
        position.side_to_move = saved_side;
        position.en_passant = saved_ep;
        position.hash = saved_hash;
        if null_score >= beta {
            NULL_MOVE_CUTOFFS.fetch_add(1, Ordering::Relaxed);
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, beta, depth as u8, 2, 0);
            return beta;
        }
    }
    let mut legal_moves = MoveList::new();
    position.all_moves(table, &mut legal_moves);
    if legal_moves.len == 0 {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        if position.king_under_attack(table) {
            t_table.store(position.hash, -1000000, depth as u8, 0, 0);
            return -1000000;
        } else {
            t_table.store(position.hash, 0, depth as u8, 0, 0);
            return 0;
        }
    }
    let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
    legal_moves
        .as_mut_slice()
        .sort_by_key(|mv| -move_score(&position, mv, tt_move, killers, ply));
    let original_alpha = alpha;
    let mut best_move = 0;
    for mv in legal_moves.as_slice() {
        let undo = position.make_move(*mv);
        let score = -negamax(
            position,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            table,
            t_table,
            killers,
        );
        position.unmake_move(*mv, undo);
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
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        if best_move != 0 {
            let flag = Move { value: best_move }.flags();
            if flag < MoveFlags::CAPTURE {
                if killers[ply as usize][0] != best_move {
                    killers[ply as usize][1] = killers[ply as usize][0];
                    killers[ply as usize][0] = best_move;
                }
            }
        }
        t_table.store(position.hash, alpha, depth as u8, 2, best_move);
    } else if alpha > original_alpha {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, alpha, depth as u8, 0, best_move);
    } else {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, alpha, depth as u8, 1, best_move);
    }
    alpha
}

pub fn best_move(
    position: &mut Position,
    depth: u32,
    table: &MagicTable,
    t_table: &mut TransponationTable,
) -> Option<Move> {
    let mut result: Option<Move> = None;
    let mut legal_moves = MoveList::new();
    position.all_moves(table, &mut legal_moves);
    let mut killers = [[0u16; 2]; 64];
    for d in 1..depth + 1 {
        let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
        legal_moves
            .as_mut_slice()
            .sort_by_key(|mv| -move_score(&position, mv, tt_move, &killers, 0));
        let mut partial_result = None;
        let mut highest_score = -1000000i32;
        for mv in legal_moves.as_slice() {
            let undo = position.make_move(*mv);
            let score = -negamax(
                position,
                d - 1,
                0,
                -1000000i32,
                -highest_score,
                table,
                t_table,
                &mut killers,
            );
            position.unmake_move(*mv, undo);

            if score > highest_score {
                highest_score = score;
                partial_result = Some(*mv);
            }
        }
        if let Some(mv) = partial_result {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
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
    eprintln!("{}", STORE_CALLS.load(Ordering::Relaxed));
    t_table.stats();
    result
}

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub fn move_score(
    position: &Position,
    mv: &Move,
    tt_move: u16,
    killers: &[[u16; 2]; 64],
    ply: u32,
) -> i32 {
    if mv.value == tt_move {
        return 20000;
    }
    if mv.value == killers[ply as usize][0] || mv.value == killers[ply as usize][1] {
        return 9000;
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
        MoveFlags::QUEEN_PROMOTION | MoveFlags::ROOK_PROMOTION => 8000,
        MoveFlags::QUEENSIDE_CASTLE | MoveFlags::KINGSIDE_CASTLE => 6000,
        MoveFlags::KNIGHT_PROMOTION | MoveFlags::BISHOP_PROMOTION => 5000,
        MoveFlags::DOUBLE_PAWN_PUSH => 100,
        _ => 0i32,
    }
}

pub fn quienscence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    qdepth: i32,
    t_table: &mut TransponationTable,
) -> i32 {
    QUIESCENCE_CALLS.fetch_add(1, Ordering::Relaxed);
    let stand_pat = evaluate_for_white(&position)
        * if position.side_to_move == Color::White {
            1
        } else {
            -1
        };
    if let Some(t) = t_table.lookup(position.hash, 0, alpha, beta) {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return t;
    }
    let original_alpha = alpha;
    if qdepth <= 0 {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, stand_pat, 0, 1, 0);
        return stand_pat;
    }
    if stand_pat >= beta {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, stand_pat, 0, 2, 0);
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let mut captures = MoveList::new();
    position.all_captures(table, &mut captures);
    for c in captures.as_slice() {
        let undo = position.make_move(*c);
        let score = -quienscence(position, -beta, -alpha, table, qdepth - 1, t_table);
        position.unmake_move(*c, undo);

        if score >= beta {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, beta, 0, 2, 0);
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    STORE_CALLS.fetch_add(1, Ordering::Relaxed);
    if alpha > original_alpha {
        t_table.store(position.hash, alpha, 0, 0, 0);
    } else {
        t_table.store(position.hash, alpha, 0, 1, 0);
    }
    alpha
}
