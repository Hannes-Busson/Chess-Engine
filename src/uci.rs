use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::{ops::Add, time::Instant, u64};

use crate::{
    file_to_number,
    movegen::{MagicTable, Move, MoveList},
    position::{Color, Position},
    search::{self, best_move},
    tt::TranspositionTable,
};

pub fn run() {
    let num_threads = 4 as usize;
    let table = Arc::new(MagicTable::init());
    let mut game = Position::start();
    let mut line = String::new();
    let mut t_table = Arc::new(TranspositionTable::new());

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
                t_table = Arc::new(TranspositionTable::new());
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
            "go" => {
                let mut stop = Arc::new(AtomicBool::new(false));
                match tokens[1] {
                    "depth" => {
                        let depth: u32 = tokens[2].parse().unwrap();
                        let best_mv =
                            best_move(&mut game, depth, &table, &t_table, u64::MAX, &stop, None);
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
                            let best_mv = mult_best_move(
                                num_threads,
                                &table,
                                &t_table,
                                &mut game,
                                100,
                                time_limit_ms,
                            );
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
                            let best_mv = mult_best_move(
                                num_threads,
                                &table,
                                &t_table,
                                &mut game,
                                100,
                                time_limit_ms,
                            );
                            if let Some(mv) = best_mv {
                                println!("bestmove {}", mv_to_string(mv));
                            }
                        }
                    },
                    _ => {
                        let best_mv =
                            best_move(&mut game, 100, &table, &t_table, 5000, &stop, None);
                        if let Some(mv) = best_mv {
                            println!("bestmove {}", mv_to_string(mv));
                        }
                    }
                }
            }
            "benchmt" => {
                search::reset_stats();
                let bench_start = Instant::now();
                let positions = [
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                        .to_string(),
                    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(),
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
                    "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9"
                        .to_string(),
                    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(),
                    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
                        .to_string(),
                    "3r3r/1p4pp/2nb1k2/pP3p2/8/PB2PN2/p4PPP/R4RK1 b - - 0 1".to_string(),
                ];
                let shared_nodes = Arc::new(AtomicU64::new(0));
                for p in positions {
                    let mut stop = Arc::new(AtomicBool::new(false));
                    let mut pos = Position::from_fen(&p);
                    let mut handler: Vec<JoinHandle<()>> = Vec::new();
                    for i in 0..num_threads - 1 {
                        let helper_shared_nodes = Arc::clone(&shared_nodes);
                        let helper_table = Arc::clone(&table);
                        let helper_t_table = Arc::clone(&t_table);
                        let mut helper_position = pos.clone();
                        let helper_stop = Arc::clone(&stop);
                        handler.push(std::thread::spawn(move || {
                            let best_mv = best_move(
                                &mut helper_position,
                                10,
                                &helper_table,
                                &helper_t_table,
                                u64::MAX,
                                &helper_stop,
                                Some(&helper_shared_nodes),
                            );
                        }));
                    }
                    let best_mv = best_move(
                        &mut pos,
                        10,
                        &table,
                        &t_table,
                        u64::MAX,
                        &stop,
                        Some(&shared_nodes),
                    );
                    stop.store(true, Ordering::Relaxed);
                    for h in handler {
                        h.join().unwrap();
                    }
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
                let nodes = shared_nodes.load(Ordering::Relaxed);
                let nps = nodes * 1000 / bench_start.elapsed().as_millis().max(1) as u64;
                println!("combined bench: {} nodes {} nps", nodes, nps);
            }
            "benchst" => {
                search::reset_stats();
                let bench_start = Instant::now();
                let positions = [
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
                        .to_string(),
                    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(),
                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
                    "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9"
                        .to_string(),
                    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(),
                    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
                        .to_string(),
                    "3r3r/1p4pp/2nb1k2/pP3p2/8/PB2PN2/p4PPP/R4RK1 b - - 0 1".to_string(),
                ];
                let shared_nodes = Arc::new(AtomicU64::new(0));
                for p in positions {
                    t_table = Arc::new(TranspositionTable::new());
                    let mut stop = Arc::new(AtomicBool::new(false));
                    let mut pos = Position::from_fen(&p);
                    let best_mv = best_move(
                        &mut pos,
                        10,
                        &table,
                        &t_table,
                        u64::MAX,
                        &stop,
                        Some(&shared_nodes),
                    );
                    stop.store(true, Ordering::Relaxed);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
                let nodes = shared_nodes.load(Ordering::Relaxed);
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

pub fn mult_best_move(
    num_threads: usize,
    table: &Arc<MagicTable>,
    t_table: &Arc<TranspositionTable>,
    game: &mut Position,
    depth: u32,
    time_limit_ms: u64,
) -> Option<Move> {
    let mut stop = Arc::new(AtomicBool::new(false));
    let mut handler: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..num_threads - 1 {
        let helper_table = Arc::clone(&table);
        let helper_t_table = Arc::clone(&t_table);
        let mut helper_position = game.clone();
        let helper_stop = Arc::clone(&stop);
        handler.push(std::thread::spawn(move || {
            let best_mv = best_move(
                &mut helper_position,
                depth,
                &helper_table,
                &helper_t_table,
                u64::MAX,
                &helper_stop,
                None,
            );
        }));
    }
    let best_mv = best_move(game, depth, &table, &t_table, time_limit_ms, &stop, None);
    stop.store(true, Ordering::Relaxed);
    for h in handler {
        h.join().unwrap();
    }
    best_mv
}
