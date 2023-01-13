use std::{
    collections::VecDeque,
    str::FromStr,
    sync::{mpsc::Sender, Arc, Mutex}
};

use yea_fen::{chess_engines::minimax, moves::MoveType, Colored, GameState, Piece, Pos};
fn make_info(info: &str) -> String {
    info.lines().map(|s| format!("# info {s}\n")).collect::<String>().trim().to_owned()
}
fn main() {
    // wait until the "uci" command is received
    let mut input = String::new();
    // let mut done = String::new();
    let mut gs = GameState::new();
    let mut best_move: Option<(String)> = None;
    // let (tx_go, rx_go) = std::sync::mpsc::channel::<Option<(MoveType<Pos, Colored<Piece>>, Option<Colored<Piece>>)>>();
    // let (tx_set, rx_set) = std::sync::mpsc::channel();
    // let mut set = false;
    // let mut go = false;
    let res = std::io::stdin();
    let queue: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let queue2 = Arc::clone(&queue);
    std::thread::spawn(move || {
        let mut move_list: Vec<String> = Vec::new();
        loop {
            let input: String = if let Ok(mut q) = queue2.lock() {
                if let Some(i) = q.pop_front() {
                    println!("{}", make_info(&i));
                    i
                } else {
                    continue;
                }
            } else {
                continue;
            };
            if input.trim() == "uci" {
                println!("id name Flawed Formula");
                println!("id author mendelsshop");
                // print set options
                println!("uciok");
            } else if input.trim() == "isready" {
                println!("readyok");
            } else if input.trim() == "ucinewgame" {
                gs = NEW_STATE;
            } else if input.trim() == "stop" {
                // if let Some(moves) = best_move.clone() {
                //     println!("{}", moves);
                // } else {
                //     println!("# info error: no best move found");
                // }
            }
            else if input.trim() == "quit" {
                std::process::exit(0);
            }
            else if input.trim().starts_with("position") {
                set_position(input.trim(), &mut move_list, &mut gs);
                println!("# info set position");
                println!("{}", make_info(format!("board: \n{}", gs.get_board().to_string()).as_str()));

                println!("{}", make_info(format!("gs fen {}", gs.to_string()).as_str()));
            } else if input.trim().starts_with("go") {
                if let Some((m, p)) = minimax::minimax(&mut gs, 3) {
                    let p = if let Some(p) = p {
                        match p {
                            Colored::White(Piece::Queen) => Some('q'),
                            Colored::White(Piece::Rook) => Some('r'),
                            Colored::White(Piece::Bishop) => Some('b'),
                            Colored::White(Piece::Knight) => Some('n'),
                            Colored::Black(Piece::Queen) => Some('Q'),
                            Colored::Black(Piece::Rook) => Some('R'),
                            Colored::Black(Piece::Bishop) => Some('B'),
                            Colored::Black(Piece::Knight) => Some('N'),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let moves = format!("bestmove {}{}{}",
                    m.from().0,
                    m.to(),
                    match p {
                        Some(p) => p,
                        None => ' ',
                    });
                    best_move = Some(moves.clone());
                    println!(
                        "{moves}"
                    );
                } else {
                    println!("# info error: could not find best move");
                }
            }

        }
    });
    loop {
        input.clear();

        res.read_line(&mut input).unwrap();

        println!("{}", make_info(format!("input: {}", input).as_str()));
        queue.lock().unwrap().push_back(input.clone());
        // push(input.clone());
    }
}
const NEW_STATE: GameState = GameState::new();
fn set_position(pos_str: &str, current_moves: &mut Vec<String>, gs: &mut GameState) {
    // remove the "position " from the str
    if let Some(pos_str) = pos_str.strip_prefix("position ") {
        let mut split_str = pos_str.split(" ");
        if let Some(r#type) = split_str.next() {
            println!("{}", make_info(format!("type {}", r#type).as_str()));
            match r#type {
                "startpos" => {
                    *gs = NEW_STATE;
                    if let Some("moves") = split_str.next() {
                        // do nothing
                    } else {
                        println!("# info error \"moves\" not found after \"startpos\"");
                        // return;
                    }
                }
                "fen" => {
                    let mut fen = String::new();
                    while let Some(s) = split_str.next() {
                        if s == "moves" {
                            println!("{}", make_info(&fen));
                            println!("{}", make_info(&format!("fen: {}", fen)));
                            break;
                        }
                        fen.push_str(s);
                        fen.push(' ');
                    }

                    *gs = match GameState::from_str(&fen) {
                        Ok(new_gs) => new_gs,
                        Err(_) => {
                            println!("# info error invalid fen");
                            return;
                        }
                    }
                }
                _ => {
                    println!("# info error invalid position type");
                    return;
                }
            }
            // let new_moves = split_str.clone().collect::<Vec<&str>>();
            // let mut move_iter = current_moves.iter();
            // let mut new_move_iter = split_str.clone();
            // loop {
            //     let nxt_new = new_move_iter.next();
            //     let nxt_old = move_iter.next();
            //     if let (Some(new_mover), Some(old_mover)) = (nxt_new, nxt_old) {
            //         if new_mover != old_mover {
            //             break;
            //         }
            //     } else if nxt_new.is_some() {
            //         split_str = new_move_iter.clone();
            //         break;
            //     } 
            //     else {
            //         break;
            //     }
            // }
            for r#move in split_str {
                if let Some(m) = get_move(r#move, gs) {
                    if gs.do_move(m.0, m.1) {
                        // println!("# info move {} done", r#move);
                        println!("{}", make_info(format!("move {} done", r#move).as_str()));
                    } else {
                        // println!("# info error move {} failed", r#move);
                        println!("{}", make_info(format!("move {} failed", r#move).as_str()));
                    }
                }
            }
        }
    }
}

fn thread_find_move(
    gs: &GameState,
    tx: Sender<Option<(MoveType<Pos, Colored<Piece>>, Option<Colored<Piece>>)>>,
) {
    let mut gs = gs.clone();
    std::thread::spawn(move || {
        let best_move = minimax::minimax(&mut gs, 3);
        tx.send(best_move).unwrap();
    });
}

fn thread_set_move(gs: GameState, tx: Sender<GameState>, pos_str: String) {
    let mut gs = gs.clone();
    std::thread::spawn(move || {
        set_position(&pos_str, &mut vec![], &mut gs);
        println!("position set thread");
        tx.send(gs).unwrap();
    });
}

fn get_move(
    move_str: &str,
    gs: &mut GameState,
) -> Option<(MoveType<Pos, Colored<Piece>>, Option<Colored<Piece>>)> {
    // split the move at the first 2 chars (the start pos) the second 2 chars (the end pos) and optionaly the 3rd char (the promotion piece)
    let (start, end) = move_str.split_at(2);
    let (end, promotion) = end.split_at(2);

    // if there is a promotion piece
    let promotion = if promotion.is_empty() {
        None
    } else {
        match promotion.chars().next().unwrap() {
            'q' => Some(Colored::White(Piece::Queen)),
            'r' => Some(Colored::White(Piece::Rook)),
            'b' => Some(Colored::White(Piece::Bishop)),
            'n' => Some(Colored::White(Piece::Knight)),
            'Q' => Some(Colored::Black(Piece::Queen)),
            'R' => Some(Colored::Black(Piece::Rook)),
            'B' => Some(Colored::Black(Piece::Bishop)),
            'N' => Some(Colored::Black(Piece::Knight)),
            _ => None,
        }
    };
    let start_pos = Pos::from_str(start).ok()?;
    let end_pos = Pos::from_str(end).ok()?;

    // get all the moves
    let moves = gs.new_all_valid_moves(gs.get_active_color());

    // find the move that .to() = end_pos, and .from() = start_pos
    moves
        .into_iter()
        .find(|m| m.to() == end_pos && m.from().0 == start_pos)
        .map(|m| (m, promotion))
}

mod tests {
    #[test]
    fn start_uci() {
        std::thread::spawn(|| {
            crate::main();
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("uci");
        std::thread::sleep(std::time::Duration::from_secs(50))
    }
}
