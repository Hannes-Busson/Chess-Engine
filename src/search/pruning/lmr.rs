use crate::{
    board::position::Position,
    movegen::movegen::MoveFlags,
    search::search::{SearchContext, negamax},
};

pub fn try_lmr(
    position: &mut Position,
    depth: u32,
    ply: u32,
    alpha: i32,
    beta: i32,
    move_index: u32,
    in_check: bool,
    flag: u8,
    ctx: &mut SearchContext,
) -> i32 {
    let mut score;
    if move_index >= 2 && depth >= 3 && !in_check && flag < MoveFlags::CAPTURE {
        score = -negamax(position, depth - 2, ply + 1, -beta, -alpha, ctx);
        if score > alpha {
            score = -negamax(position, depth - 1, ply + 1, -beta, -alpha, ctx);
        }
    } else {
        score = -negamax(position, depth - 1, ply + 1, -beta, -alpha, ctx);
    }
    score
}
