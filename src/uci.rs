use crate::{
    file_to_number,
    movegen::{MagicTable, Move, MoveList},
    position::Position,
    search::best_move,
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
                    for t in 3..tokens.len() {
                        let mut coordinates = [' '; 4];
                        let mut legal_moves = MoveList::new();
                        game.all_moves(&table, &mut legal_moves);
                        for i in 0..4 {
                            coordinates[i] = tokens[t].as_bytes()[i] as char;
                        }
                        let file_from = file_to_number(coordinates[0]);
                        let file_to = file_to_number(coordinates[2]);
                        let from = file_from + (coordinates[1] as u8 - b'1') * 8;
                        let to = file_to + (coordinates[3] as u8 - b'1') * 8;
                        if from < 64 {
                            if to < 64 {
                                let prop_move = if tokens[t].len() == 5 {
                                    let promo = tokens[t].as_bytes()[4] as char;
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
                    }
                }
                _ => println!("Command unknown"),
            },
            "go" => match tokens[1] {
                "depth" => {
                    let depth: u32 = tokens[2].parse().unwrap();
                    let best_mv = best_move(&mut game, depth, &table, &mut t_table);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
                _ => {
                    let best_mv = best_move(&mut game, 10, &table, &mut t_table);
                    if let Some(mv) = best_mv {
                        println!("bestmove {}", mv_to_string(mv));
                    }
                }
            },
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
