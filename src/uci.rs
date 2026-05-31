use std::{ops::Add, time::Instant, u64};

use crate::{
    file_to_number,
    movegen::{MagicTable, Move, MoveList},
    position::{Color, Position},
    search::{self, best_move, get_nodes},
    tt::TransponationTable,
};

pub fn run() {
    let table = MagicTable::init();
    let mut game = Position::start();
    let mut line = String::new();
    let mut t_table = TransponationTable::new();

    loop {
        line.clear();
        let bytes = std::io::stdin().read_line(&mut line).unwrap();
        if bytes == 0 {
            break;
        }
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "uci" => println!("id name SAYA\nid author Tec\nuciok"),
            "isready" => println!("readyok"),
            "ucinewgame" => {
                game = Position::start();
                t_table = TransponationTable::new();
            }
            "quit" => break,
            "position" => match tokens[1] {
                "startpos" => {
                    game = Position::start();
                    for i in 3..tokens.len() {
                        game = do_move_from_coordinates(&mut game, tokens[i], &table);
                    }
                }
                "fen" => {
                    let mut string = String::new();
                    string = string.add(tokens[2]);
                    for i in 3..8 {
                        string = string.add(" ").add(tokens[i]);
                    }
                    game = Position::from_fen(&string);
                    if tokens.get(8) == Some(&"moves") {
                        for i in 9..tokens.len() {
                            game = do_move_from_coordinates(&mut game, tokens[i], &table);
                        }
                    }
                }
                _ => println!("Command unknown"),
            },
            "go" => match tokens[1] {
                "depth" => {
                    let depth: u32 = tokens[2].parse().unwrap();
                    let best_mv = best_move(&mut game, depth, &table, &mut t_table, u64::MAX);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
                "wtime" => match game.side_to_move {
                    Color::White => {
                        let mut time_limit_ms = tokens[2].parse::<u64>().unwrap();
                        if time_limit_ms < 10000 {
                            time_limit_ms /= 10;
                        } else {
                            time_limit_ms /= 30;
                            if let Some(t) = tokens.get(6) {
                                let time_increment = t.parse::<u64>().unwrap();
                                time_limit_ms += time_increment * 3 / 4;
                            }
                        }
                        let best_mv =
                            best_move(&mut game, 100, &table, &mut t_table, time_limit_ms);
                        if let Some(mv) = best_mv {
                            println!("bestmove {}", mv_to_string(mv));
                        }
                    }
                    Color::Black => {
                        let mut time_limit_ms = tokens[4].parse::<u64>().unwrap();
                        if time_limit_ms < 10000 {
                            time_limit_ms /= 10;
                        } else {
                            time_limit_ms /= 30;
                            if let Some(t) = tokens.get(8) {
                                let time_increment = t.parse::<u64>().unwrap();
                                time_limit_ms += time_increment * 3 / 4;
                            }
                        }
                        let best_mv =
                            best_move(&mut game, 100, &table, &mut t_table, time_limit_ms);
                        if let Some(mv) = best_mv {
                            println!("bestmove {}", mv_to_string(mv));
                        }
                    }
                },
                _ => {
                    let best_mv = best_move(&mut game, 100, &table, &mut t_table, 5000);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
            },
            "bench" => {
                search::reset_stats();
                let bench_start = Instant::now();
                let positions = [
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                        .to_string(),
                    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(),
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
                ];
                for p in positions {
                    let mut pos = Position::from_fen(&p);
                    let best_mv = best_move(&mut pos, 8, &table, &mut t_table, u64::MAX);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
                let nodes = get_nodes();
                let nps = nodes * 1000 / bench_start.elapsed().as_millis().max(1) as u64;
                println!("bench: {} nodes {} nps", nodes, nps);
            }
            _ => println!("Command unknown"),
        }
    }
}

pub fn mv_to_string(mv: Move) -> String {
    let mut coordinates: [char; 4] = [' '; 4];
    let from = mv.from();
    let to = mv.to();
    let flag = mv.flags();
    coordinates[0] = (b'a' + from % 8) as char;
    coordinates[1] = (b'1' + from / 8) as char;
    coordinates[2] = (b'a' + to % 8) as char;
    coordinates[3] = (b'1' + to / 8) as char;
    let mut move_string = String::new();
    for c in coordinates {
        move_string.push(c);
    }
    if flag >= 8 {
        match flag as usize {
            8 | 12 => move_string.push('n'),
            9 | 13 => move_string.push('b'),
            10 | 14 => move_string.push('r'),
            11 | 15 => move_string.push('q'),
            _ => println!("Wrong flag -> Promotion append"),
        }
    }
    move_string
}

pub fn do_move_from_coordinates(game: &mut Position, token: &str, table: &MagicTable) -> Position {
    let mut coordinates = [' '; 4];
    let mut legal_moves = MoveList::new();
    game.all_moves(&table, &mut legal_moves);
    for i in 0..4 {
        coordinates[i] = token.as_bytes()[i] as char;
    }
    let file_from = file_to_number(coordinates[0]);
    let file_to = file_to_number(coordinates[2]);
    let from = file_from + (coordinates[1] as u8 - b'1') * 8;
    let to = file_to + (coordinates[3] as u8 - b'1') * 8;
    if from < 64 {
        if to < 64 {
            let prop_move = if token.len() == 5 {
                let promo = token.as_bytes()[4] as char;
                let promo_idx = match promo {
                    'n' => 0u8,
                    'b' => 1,
                    'r' => 2,
                    _ => 3,
                };
                legal_moves.as_slice().iter().find(|m| {
                    m.from() == from
                        && m.to() == to
                        && m.flags() >= 8
                        && (m.flags() & 3) == promo_idx
                })
            } else {
                legal_moves
                    .as_slice()
                    .iter()
                    .find(|m| m.from() == from && m.to() == to)
            };

            if let Some(mv) = prop_move {
                let _ = game.make_move(*mv);
            } else {
                println!("Invalid move.")
            }
        } else {
            println!("Invalid target square");
        }
    } else {
        println!("Invalid start square.");
    }
    *game
}
