mod bitboard;
mod game;
mod masks;
mod movegen;
mod position;
#[cfg(test)]
mod tests;

use std::io;
use std::time::Instant;

use crate::game::Game;
use bitboard::Bitboard;
use movegen::{Move, MoveFlags, MoveGen};
use position::{Color, PieceType, Position};

use crate::movegen::MagicTable;

fn main() {
    game();
}

pub fn game() {
    let table = MagicTable::init();
    let mut game = Game::new(300000, 300000, 5000);
    let mut move_string = String::new();
    let start_time = Instant::now();
    loop {
        game.position.display();
        if game.is_draw() {
            print!("It's a draw.");
            break;
        }
        if game.position.all_moves(&table).is_empty() {
            match game.position.side_to_move {
                Color::White => print!("Winner is Black"),
                Color::Black => print!("Winner is White"),
            }
            break;
        }
        io::stdin().read_line(&mut move_string);
        print!("{}", move_string);
        move_string.clear();
    }
}
