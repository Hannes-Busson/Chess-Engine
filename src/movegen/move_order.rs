// move_score for ordering move evaluation for improving beta cutoffs

use crate::board::position::{PIECE_VALUES, Position};
use crate::movegen::movegen::{Move, MoveFlags};
use crate::search::search::SearchContext;

pub fn move_score(
    position: &Position,
    mv: &Move,
    tt_move: u16,
    ply: u32,
    ctx: &SearchContext,
) -> i32 {
    // check for best move in table
    if mv.value == tt_move {
        return 20000;
    }
    // check for killer moves
    if mv.value == ctx.killers[ply as usize][0] || mv.value == ctx.killers[ply as usize][1] {
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
        _ => ctx.history[from as usize][to as usize].min(8500),
    }
}

pub fn update_killers_history(best_move: u16, ply: u32, depth: u32, ctx: &mut SearchContext) {
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
}
