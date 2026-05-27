mod bitboard;
mod eval;
mod game;
mod masks;
mod movegen;
mod position;
mod search;
#[cfg(test)]
mod tests;
mod tt;
mod uci;
mod zobrist;

use std::io;
use std::time::Instant;

use crate::game::Game;
use bitboard::Bitboard;
use movegen::{Move, MoveFlags, MoveGen};
use position::{Color, PieceType, Position};

use crate::movegen::MagicTable;

fn main() {
    uci::run();
}

pub fn game() {
    let table = MagicTable::init();
    let mut game = Game::new(300000, 300000, 5000);
    let mut move_string = String::new();
    let start_time = Instant::now();
    let mut time_passed = 0u64;
    loop {
        let moves = game.position.all_moves(&table);
        if game.is_draw() {
            print!("It's a draw.");
            break;
        }
        if moves.is_empty() {
            if game.position.king_under_attack(&table) {
                match game.position.side_to_move {
                    Color::White => print!("Winner is Black"),
                    Color::Black => print!("Winner is White"),
                }
            } else {
                print!("It's a draw.");
            }
            break;
        }
        game.position.display();
        let mut coordinates = [' '; 4];
        loop {
            move_string.clear();
            io::stdin().read_line(&mut move_string);
            if move_string.len() == 5 {
                for i in 0..4 {
                    coordinates[i] = move_string.as_bytes()[i] as char;
                }
                let file_from = file_to_number(coordinates[0]);
                let file_to = file_to_number(coordinates[2]);
                let from = file_from + (coordinates[1] as u8 - b'1') * 8;
                let to = file_to + (coordinates[3] as u8 - b'1') * 8;
                if from < 64 {
                    if to < 64 {
                        let prop_move = moves.iter().find(|m| m.from() == from && m.to() == to);
                        if let Some(mv) = prop_move {
                            let now = start_time.elapsed().as_millis() as u64;
                            game.make_move(*mv, now - time_passed);
                            time_passed += now - time_passed;
                            break;
                        } else {
                            println!("Invalid move.")
                        }
                    } else {
                        println!("Invalid target square");
                    }
                } else {
                    println!("Invalid start square.");
                }
            } else {
                println!("Wrong input length.");
            }
        }
    }
}

pub fn file_to_number(c: char) -> u8 {
    match c {
        'a' => 0u8,
        'b' => 1u8,
        'c' => 2u8,
        'd' => 3u8,
        'e' => 4u8,
        'f' => 5u8,
        'g' => 6u8,
        'h' => 7u8,
        _ => 64u8,
    }
}
