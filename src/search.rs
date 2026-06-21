use crate::{
    eval::evaluate_for_white,
    move_order::move_score,
    movegen::{MagicTable, Move, MoveFlags, MoveGen, MoveList},
    position::{Color, Position},
    pruning::null_move::try_null_move,
    tt::TranspositionTable,
};

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

pub struct SearchContext<'a> {
    pub table: &'a MagicTable,
    pub t_table: &'a TranspositionTable,
    pub killers: &'a mut [[u16; 2]; 64],
    pub history: &'a mut [[i32; 64]; 64],
    pub stop: &'a Arc<AtomicBool>,
    pub nodes: &'a mut u64,
}

// search algorithm - brute force

pub fn negamax(
    position: &mut Position,
    depth: u32,
    ply: u32,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchContext,
) -> i32 {
    *ctx.nodes += 1;
    // leaf check via quiescence: checks for captures on depth 0
    if depth == 0 {
        return quienscence(position, alpha, beta, 4, ctx);
    }
    // checks for table entry
    if let Some(t) = ctx.t_table.lookup(position.hash, depth as u8, alpha, beta) {
        return t;
    }
    let in_check = position.king_under_attack(ctx.table);
    // null move check
    if let Some(score) = try_null_move(position, depth, ply, beta, in_check, ctx) {
        return score;
    }
    // collect moves and sort them with move_score
    let mut legal_moves = MoveList::new();
    position.all_moves(ctx.table, &mut legal_moves);
    let tt_move = ctx.t_table.get_best_move(position.hash).unwrap_or(0);
    legal_moves
        .as_mut_slice()
        .sort_by_key(|mv| -move_score(&position, mv, tt_move, ply, &ctx));
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
            ctx.table,
        ) {
            position.unmake_move(*mv, undo);
            continue;
        } else {
            legal_move_count += 1;
        }
        let mut score;
        if move_index >= 2 && depth >= 3 && !in_check && mv.flags() < MoveFlags::CAPTURE {
            score = -negamax(position, depth - 2, ply + 1, -beta, -alpha, ctx);
            if score > alpha {
                score = -negamax(position, depth - 1, ply + 1, -beta, -alpha, ctx);
            }
        } else {
            score = -negamax(position, depth - 1, ply + 1, -beta, -alpha, ctx);
        }
        position.unmake_move(*mv, undo);
        // updates alpha if checked move is better
        if score > alpha {
            alpha = score;
            best_move = mv.value;
        }
        // Beta cutoff when move is too good that the oppenent allow it
        if alpha >= beta {
            break;
        }
        move_index += 1;
    }
    // check for checkmate and draw if no move found
    if legal_move_count == 0 {
        if position.king_under_attack(ctx.table) {
            ctx.t_table
                .store(position.hash, -1000000, depth as u8, 0, 0);
            return -1000000;
        } else {
            ctx.t_table.store(position.hash, 0, depth as u8, 0, 0);
            return 0;
        }
    }
    if alpha >= beta && !ctx.stop.load(Ordering::Relaxed) {
        if best_move != 0 {
            let flag = Move { value: best_move }.flags();
            let from = Move { value: best_move }.from();
            let to = Move { value: best_move }.to();
            if flag < MoveFlags::CAPTURE {
                if ctx.killers[ply as usize][0] != best_move {
                    ctx.killers[ply as usize][1] = ctx.killers[ply as usize][0];
                    ctx.killers[ply as usize][0] = best_move;
                }
                ctx.history[from as usize][to as usize] += depth as i32 * depth as i32;
            }
        }
        ctx.t_table
            .store(position.hash, alpha, depth as u8, 2, best_move);
    } else if alpha > original_alpha && !ctx.stop.load(Ordering::Relaxed) {
        ctx.t_table
            .store(position.hash, alpha, depth as u8, 0, best_move);
    } else {
        if !ctx.stop.load(Ordering::Relaxed) {
            ctx.t_table
                .store(position.hash, alpha, depth as u8, 1, best_move);
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
    let mut ctx = SearchContext {
        table,
        t_table,
        killers: &mut killers,
        history: &mut history,
        stop,
        nodes: &mut nodes,
    };
    // search for every depth ascending
    for d in 1..depth + 1 {
        let (mut alpha, mut beta) = if d == 1 {
            (-1000000, 1000000)
        } else {
            (prev_score - 50, prev_score + 50)
        };
        for i in 0..64 {
            for j in 0..64 {
                ctx.history[i][j] /= 2;
            }
        }
        let tt_move = t_table.get_best_move(position.hash).unwrap_or(0);
        legal_moves
            .as_mut_slice()
            .sort_by_key(|mv| -move_score(&position, mv, tt_move, 0, &ctx));
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
                let score = -negamax(position, d - 1, 0, -beta, -alpha, &mut ctx);

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
            t_table.store(position.hash, highest_score, d as u8, 0, mv.value);
        }
        // info print for cutechess
        let elapsed_time = start.elapsed().as_millis().max(1) as u64;
        println!(
            "info depth {} score cp {} nodes {} time {} nps {}",
            d,
            highest_score,
            *ctx.nodes,
            elapsed_time,
            *ctx.nodes * 1000 / elapsed_time
        );
        result = partial_result_mv;
    }
    if let Some(s) = shared_nodes {
        s.fetch_add(*ctx.nodes, Ordering::Relaxed);
    }
    result
}

// quiescence: check for captures on depth 0 to not miss important captures

pub fn quienscence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    qdepth: i32,
    ctx: &mut SearchContext,
) -> i32 {
    *ctx.nodes += 1;
    // check table
    if let Some(t) = ctx.t_table.lookup(position.hash, 0, alpha, beta) {
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
        if !ctx.stop.load(Ordering::Relaxed) {
            ctx.t_table.store(position.hash, stand_pat, 0, 1, 0);
        }
        return stand_pat;
    }
    if stand_pat >= beta {
        if !ctx.stop.load(Ordering::Relaxed) {
            ctx.t_table.store(position.hash, stand_pat, 0, 2, 0);
        }
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }
    let mut captures = MoveList::new();
    position.all_captures(ctx.table, &mut captures);
    // check all captures until no more captures
    for mv in captures.as_slice() {
        let undo = position.make_move(*mv);

        if MoveGen::is_attacked(
            position.opponent(),
            position.pieces[position.opponent() as usize * 6 + 5]
                .0
                .trailing_zeros() as u8,
            &position,
            ctx.table,
        ) {
            position.unmake_move(*mv, undo);
            continue;
        }

        let score = -quienscence(position, -beta, -alpha, qdepth - 1, ctx);
        position.unmake_move(*mv, undo);

        if score >= beta {
            if !ctx.stop.load(Ordering::Relaxed) {
                ctx.t_table.store(position.hash, beta, 0, 2, 0);
            }
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    if alpha > original_alpha && !ctx.stop.load(Ordering::Relaxed) {
        ctx.t_table.store(position.hash, alpha, 0, 0, 0);
    } else if !ctx.stop.load(Ordering::Relaxed) {
        ctx.t_table.store(position.hash, alpha, 0, 1, 0);
    }
    alpha
}
