use std::io::Write;

use crate::movegen::{Move, MoveFlags};
use crate::position::{Color, Position};

pub struct Game {
    pub position: Position,
    pub history: Vec<Position>,
    pub half_move_clock: u32,
    pub full_move_number: u32,
    pub white_time_ms: u64,
    pub black_time_ms: u64,
    pub increment_ms: u64,
}

impl Game {
    pub fn new(white_time_ms: u64, black_time_ms: u64, increment_ms: u64) -> Self {
        let history: Vec<Position> = Vec::new();
        Game {
            position: Position::start(),
            history: history,
            half_move_clock: 0u32,
            full_move_number: 1u32,
            white_time_ms: white_time_ms,
            black_time_ms: black_time_ms,
            increment_ms: increment_ms,
        }
    }

    pub fn make_move(&mut self, mv: Move, elapsed_ms: u64) -> &mut Self {
        let mut new_game_state = self;
        let is_pawn_move = new_game_state.position.piece_on[mv.from() as usize] % 6 == 0;
        new_game_state.history.push(new_game_state.position);
        new_game_state.position = new_game_state.position.make_move(mv);
        let flag = mv.flags();
        if flag == MoveFlags::CAPTURE || is_pawn_move {
            new_game_state.half_move_clock = 0;
        } else {
            new_game_state.half_move_clock += 1;
        }
        if new_game_state.position.side_to_move == Color::Black {
            new_game_state.white_time_ms -= elapsed_ms;
            new_game_state.white_time_ms += new_game_state.increment_ms;
        } else {
            new_game_state.black_time_ms -= elapsed_ms;
            new_game_state.black_time_ms += new_game_state.increment_ms;
            new_game_state.full_move_number += 1;
        }
        new_game_state
    }

    pub fn is_draw(&self) -> bool {
        self.half_move_clock >= 100
            || self.history.iter().filter(|p| **p == self.position).count() >= 2
    }
}
