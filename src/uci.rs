use crate::{
    file_to_number,
    movegen::MagicTable,
    position::{self, Position},
    search::best_move,
};

pub fn run() {
    let table = MagicTable::init();
    let mut game = Position::start();
    let mut line = String::new();

    loop {
        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "uci" => println!("id name SAYA\nid author Tec\nuciok"),
            "isready" => println!("readyok"),
            "ucinewgame" => game = Position::start(),
            "quit" => break,
            "position" => match tokens[1] {
                "startpos" => {
                    game = Position::start();
                    for t in 3..tokens.len() {
                        let mut coordinates = [' '; 4];
                        let legal_moves = game.all_moves(&table);
                        for i in 0..4 {
                            coordinates[i] = tokens[t].chars().nth(i).unwrap();
                        }
                        let file_from = file_to_number(coordinates[0]);
                        let file_to = file_to_number(coordinates[2]);
                        let from = file_from + (coordinates[1] as u8 - b'1') * 8;
                        let to = file_to + (coordinates[3] as u8 - b'1') * 8;
                        if from < 64 {
                            if to < 64 {
                                let prop_move = legal_moves
                                    .iter()
                                    .find(|m| m.from() == from && m.to() == to);
                                if let Some(mv) = prop_move {
                                    game = game.make_move(*mv);
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
                    let best_mv = best_move(game, depth, &table);
                    let mut coordinates: [char; 4] = [' '; 4];
                    if let Some(mv) = best_mv {
                        let from = mv.from();
                        let to = mv.to();
                        coordinates[0] = (b'a' + from % 8) as char;
                        coordinates[1] = (b'1' + from / 8) as char;
                        coordinates[2] = (b'a' + to % 8) as char;
                        coordinates[3] = (b'1' + to / 8) as char;
                        let mut move_string = String::new();
                        for c in coordinates {
                            move_string.push(c);
                        }
                        println!("bestmove {}", move_string);
                    }
                }
                _ => {
                    // let depth: u32 = tokens[2].parse().unwrap();
                    let best_mv = best_move(game, 8, &table);
                    let mut coordinates: [char; 4] = [' '; 4];
                    if let Some(mv) = best_mv {
                        let from = mv.from();
                        let to = mv.to();
                        coordinates[0] = (b'a' + from % 8) as char;
                        coordinates[1] = (b'1' + from / 8) as char;
                        coordinates[2] = (b'a' + to % 8) as char;
                        coordinates[3] = (b'1' + to / 8) as char;
                        let mut move_string = String::new();
                        for c in coordinates {
                            move_string.push(c);
                        }
                        println!("bestmove {}", move_string);
                    }
                }
            },
            _ => println!("Command unknown"),
        }
    }
}
