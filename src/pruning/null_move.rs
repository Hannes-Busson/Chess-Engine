// src/pruning/null_move.rs

use crate::position::Position;
use crate::search::{SearchContext, negamax};
use crate::zobrist;
use std::sync::atomic::Ordering;

pub fn try_null_move(
    position: &mut Position,
    depth: u32,
    ply: u32,
    beta: i32,
    in_check: bool,
    ctx: &mut SearchContext,
) -> Option<i32> {
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
        let null_score = -negamax(position, depth - 3, ply + 1, -beta, -beta + 1, ctx);
        position.side_to_move = saved_side;
        position.en_passant = saved_ep;
        position.hash = saved_hash;
        if null_score >= beta {
            if !ctx.stop.load(Ordering::Relaxed) {
                ctx.t_table.store(position.hash, beta, depth as u8, 2, 0);
            }
            return Some(beta);
        }
    }
    None
}
