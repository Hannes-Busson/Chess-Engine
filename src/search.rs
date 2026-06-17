use crate::{
    eval::evaluate_for_white,
    movegen::{MagicTable, Move, MoveFlags, MoveGen, MoveList},
    position::{Color, Position},
    tt::{self, TranspositionTable},
    zobrist,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

// Stat variables

static NEGAMAX_CALLS: AtomicU64 = AtomicU64::new(0);
static TT_HITS: AtomicU64 = AtomicU64::new(0);
static TT_COLLISIONS: AtomicU64 = AtomicU64::new(0);
static BETA_CUTOFFS: AtomicU64 = AtomicU64::new(0);
static NULL_MOVE_CUTOFFS: AtomicU64 = AtomicU64::new(0);
static QUIESCENCE_CALLS: AtomicU64 = AtomicU64::new(0);
static STORE_CALLS: AtomicU64 = AtomicU64::new(0);

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub const STOP_SCORE: i32 = i32::MAX;

// search algorithm - brute force

pub fn negamax(
    position: &mut Position,
    depth: u32,
    ply: u32,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    t_table: &TranspositionTable,
    killers: &mut [[u16; 2]; 64],
    history: &mut [[i32; 64]; 64],
    stop: &Arc<AtomicBool>,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;
    // leaf check via quiescence: checks for captures on depth 0
    if depth == 0 {
        return quienscence(position, alpha, beta, table, 4, t_table, stop, nodes);
    }
    if unsafe { *t_table.vault[position.hash as usize & tt::SHIFT].get() }.hash != position.hash {
        TT_COLLISIONS.fetch_add(1, Ordering::Relaxed);
    }
    // checks for table entry
    if let Some(t) = t_table.lookup(position.hash, depth as u8, alpha, beta) {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return t;
    }
    let in_check = position.king_under_attack(table);
    // null move: checks if a position is so good that in case of passing moves still keep the lead
    if depth > 2 && !in_check {
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
            history,
            stop,
            nodes,
        );
        position.side_to_move = saved_side;
        position.en_passant = saved_ep;
        position.hash = saved_hash;
        if null_score >= beta {
            NULL_MOVE_CUTOFFS.fetch_add(1, Ordering::Relaxed);
            if !stop.load(Ordering::Relaxed) {
                STORE_CALLS.fetch_add(1, Ordering::Relaxed);
                t_table.store(position.hash, beta, depth as u8, 2, 0);
            }

            return beta;
        }
    }
    // collect moves and sort them with move_score
    let mut legal_moves = MoveList::new();
    position.all_moves(table, &mut legal_moves);
    let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
    legal_moves
        .as_mut_slice()
        .sort_by_key(|mv| -move_score(&position, mv, tt_move, killers, ply, history));
    let original_alpha = alpha;
    let mut best_move = 0;
    let mut move_index = 0;
    let mut legal_move_count = 0;
    // run loop for sorted moves with make/unmake move logic for checking if move legal
    for mv in legal_moves.as_slice() {
        let undo = position.make_move(*mv);
        if MoveGen::is_attacked(
            position.opponent(),
            position.pieces[position.opponent() as usize * 6 + 5]
                .0
                .trailing_zeros() as u8,
            &position,
            table,
        ) {
            position.unmake_move(*mv, undo);
            continue;
        } else {
            legal_move_count += 1;
        }
        let mut score;
        if move_index >= 2 && depth >= 3 && !in_check && mv.flags() < MoveFlags::CAPTURE {
            score = -negamax(
                position,
                depth - 2,
                ply + 1,
                -beta,
                -alpha,
                table,
                t_table,
                killers,
                history,
                stop,
                nodes,
            );
            if score > alpha {
                score = -negamax(
                    position,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    table,
                    t_table,
                    killers,
                    history,
                    stop,
                    nodes,
                );
            }
        } else {
            score = -negamax(
                position,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                table,
                t_table,
                killers,
                history,
                stop,
                nodes,
            );
        }
        position.unmake_move(*mv, undo);
        // updates alpha if checked move is better
        if score > alpha {
            alpha = score;
            best_move = mv.value;
        }
        // Beta cutoff when move is too good that the oppenent allow it
        if alpha >= beta {
            BETA_CUTOFFS.fetch_add(1, Ordering::Relaxed);
            break;
        }
        move_index += 1;
    }
    // check for checkmate and draw if no move found
    if legal_move_count == 0 {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        if position.king_under_attack(table) {
            t_table.store(position.hash, -1000000, depth as u8, 0, 0);
            return -1000000;
        } else {
            t_table.store(position.hash, 0, depth as u8, 0, 0);
            return 0;
        }
    }
    if alpha >= beta && !stop.load(Ordering::Relaxed) {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        if best_move != 0 {
            let flag = Move { value: best_move }.flags();
            let from = Move { value: best_move }.from();
            let to = Move { value: best_move }.to();
            if flag < MoveFlags::CAPTURE {
                if killers[ply as usize][0] != best_move {
                    killers[ply as usize][1] = killers[ply as usize][0];
                    killers[ply as usize][0] = best_move;
                }
                history[from as usize][to as usize] += depth as i32 * depth as i32;
            }
        }
        t_table.store(position.hash, alpha, depth as u8, 2, best_move);
    } else if alpha > original_alpha && !stop.load(Ordering::Relaxed) {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, alpha, depth as u8, 0, best_move);
    } else {
        if !stop.load(Ordering::Relaxed) {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, alpha, depth as u8, 1, best_move);
        }
    }
    alpha
}

// best_move kickoff for search with negamax calls

pub fn best_move(
    position: &mut Position,
    depth: u32,
    table: &MagicTable,
    t_table: &TranspositionTable,
    time_limit_ms: u64,
    stop: &Arc<AtomicBool>,
    shared_nodes: Option<&Arc<AtomicU64>>,
) -> Option<Move> {
    let mut nodes = 0u64;
    let start = Instant::now();
    let mut time_up = false;
    let mut result: Option<Move> = None;
    let mut legal_moves = MoveList::new();
    position.all_moves(table, &mut legal_moves);
    let mut killers = [[0u16; 2]; 64];
    let mut history = [[0i32; 64]; 64];
    let mut prev_score = 0;
    // search for every depth ascending
    for d in 1..depth + 1 {
        let (mut alpha, mut beta) = if d == 1 {
            (-1000000, 1000000)
        } else {
            (prev_score - 50, prev_score + 50)
        };
        for i in 0..64 {
            for j in 0..64 {
                history[i][j] /= 2;
            }
        }
        let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
        legal_moves
            .as_mut_slice()
            .sort_by_key(|mv| -move_score(&position, mv, tt_move, &killers, 0, &history));
        let mut partial_result_mv = None;
        let mut highest_score = -1000000i32;
        let mut legal_move_count = 0;
        // loop for aspiration window first small if fails big
        loop {
            let original_alpha = alpha;
            for mv in legal_moves.as_slice() {
                let undo = position.make_move(*mv);
                if MoveGen::is_attacked(
                    position.opponent(),
                    position.pieces[position.opponent() as usize * 6 + 5]
                        .0
                        .trailing_zeros() as u8,
                    &position,
                    table,
                ) {
                    position.unmake_move(*mv, undo);
                    continue;
                } else {
                    legal_move_count += 1;
                }
                let score = -negamax(
                    position,
                    d - 1,
                    0,
                    -beta,
                    -alpha,
                    table,
                    t_table,
                    &mut killers,
                    &mut history,
                    stop,
                    &mut nodes,
                );

                position.unmake_move(*mv, undo);

                if start.elapsed().as_millis() as u64 >= time_limit_ms {
                    stop.store(true, Ordering::Relaxed);
                    time_up = true;
                    break;
                }

                if score > highest_score {
                    alpha = alpha.max(score);
                    highest_score = score;
                    partial_result_mv = Some(*mv);
                }
            }
            //check time
            if time_up || stop.load(Ordering::Relaxed) {
                break;
            }
            // check for rerun or continue
            if highest_score <= original_alpha || highest_score >= beta {
                alpha = -1000000;
                beta = 1000000;
                highest_score = -1000000;
                partial_result_mv = None;
                continue;
            } else {
                prev_score = highest_score;
                break;
            }
        }
        if time_up || stop.load(Ordering::Relaxed) {
            break;
        }
        if let Some(mv) = partial_result_mv
            && !stop.load(Ordering::Relaxed)
        {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, highest_score, d as u8, 0, mv.value);
        }
        // info print for cutechess
        let elapsed_time = start.elapsed().as_millis().max(1) as u64;
        println!(
            "info depth {} score cp {} nodes {} time {} nps {}",
            d,
            highest_score,
            nodes,
            elapsed_time,
            nodes * 1000 / elapsed_time
        );
        result = partial_result_mv;
    }
    // eprintln!("{}", NEGAMAX_CALLS.load(Ordering::Relaxed));
    // eprintln!("{}", TT_HITS.load(Ordering::Relaxed));
    // eprintln!("{}", TT_COLLISIONS.load(Ordering::Relaxed));
    // eprintln!("{}", BETA_CUTOFFS.load(Ordering::Relaxed));
    // eprintln!("{}", NULL_MOVE_CUTOFFS.load(Ordering::Relaxed));
    // eprintln!("{}", QUIESCENCE_CALLS.load(Ordering::Relaxed));
    // eprintln!("{}", STORE_CALLS.load(Ordering::Relaxed));
    // t_table.stats();
    if let Some(s) = shared_nodes {
        s.fetch_add(nodes, Ordering::Relaxed);
    }
    result
}

// move_score for ordering move evaluation for improving beta cutoffs

pub fn move_score(
    position: &Position,
    mv: &Move,
    tt_move: u16,
    killers: &[[u16; 2]; 64],
    ply: u32,
    history: &[[i32; 64]; 64],
) -> i32 {
    // check for best move in table
    if mv.value == tt_move {
        return 20000;
    }
    // check for killer moves
    if mv.value == killers[ply as usize][0] || mv.value == killers[ply as usize][1] {
        return 9000;
    }
    let base = 10000i32;
    let from = mv.from();
    let to = mv.to();
    let piece_on_to = position.piece_on[to as usize] % 6;
    // general move ordering
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
        _ => history[from as usize][to as usize].min(8500),
    }
}

// quiescence: check for captures on depth 0 to not miss important captures

pub fn quienscence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    table: &MagicTable,
    qdepth: i32,
    t_table: &TranspositionTable,
    stop: &Arc<AtomicBool>,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;
    // check table
    if let Some(t) = t_table.lookup(position.hash, 0, alpha, beta) {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return t;
    }
    let stand_pat = evaluate_for_white(&position)
        * if position.side_to_move == Color::White {
            1
        } else {
            -1
        };
    let original_alpha = alpha;
    if qdepth <= 0 {
        if !stop.load(Ordering::Relaxed) {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, stand_pat, 0, 1, 0);
        }
        return stand_pat;
    }
    if stand_pat >= beta {
        if !stop.load(Ordering::Relaxed) {
            STORE_CALLS.fetch_add(1, Ordering::Relaxed);
            t_table.store(position.hash, stand_pat, 0, 2, 0);
        }
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let mut captures = MoveList::new();
    position.all_captures(table, &mut captures);
    // check all captures until no more captures
    for mv in captures.as_slice() {
        let undo = position.make_move(*mv);

        if MoveGen::is_attacked(
            position.opponent(),
            position.pieces[position.opponent() as usize * 6 + 5]
                .0
                .trailing_zeros() as u8,
            &position,
            table,
        ) {
            position.unmake_move(*mv, undo);
            continue;
        }

        let score = -quienscence(
            position,
            -beta,
            -alpha,
            table,
            qdepth - 1,
            t_table,
            stop,
            nodes,
        );
        position.unmake_move(*mv, undo);

        if score >= beta {
            if !stop.load(Ordering::Relaxed) {
                STORE_CALLS.fetch_add(1, Ordering::Relaxed);
                t_table.store(position.hash, beta, 0, 2, 0);
            }
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    if alpha > original_alpha && !stop.load(Ordering::Relaxed) {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, alpha, 0, 0, 0);
    } else if !stop.load(Ordering::Relaxed) {
        STORE_CALLS.fetch_add(1, Ordering::Relaxed);
        t_table.store(position.hash, alpha, 0, 1, 0);
    }
    alpha
}

pub fn reset_stats() {
    NEGAMAX_CALLS.store(0, Ordering::Relaxed);
}
