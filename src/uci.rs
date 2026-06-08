use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Instant;

use crate::bench::bench;
use crate::board::Board;
use crate::search::data::{SearchData, SharedData};
use crate::search::search_runner;
use crate::types::*;

pub fn input_loop(cli_args: String) {
    let mut data = SearchData::default();
    let rx = listen(data.shared.clone());

    let mut input = if !cli_args.is_empty() {
        cli_args
    } else {
        String::new()
    };

    loop {
        if input.is_empty() {
            if let Ok(s) = rx.recv() {
                input = s;
            } else {
                data.shared.status.stop();
                break;
            }
        }

        let (command, args) = input.split_once(" ").unwrap_or((&input, ""));

        match command.trim() {
            "position" => position(args, &mut data.board),
            "uci" => uci(),
            "isready" => println!("readyok"),
            "ucinewgame" => {
                data.shared.tt.clear();
                data = SearchData {
                    shared: data.shared,
                    ..Default::default()
                };
            }
            "go" => {
                data.time.clear_settings();
                data.shared.status.run();

                if let Some(e) = go(args, &mut data) {
                    println!("bestmove {}", e.mv);
                }
            }
            "quit" => break,
            "perft" => {
                if let Ok(depth) = args.trim().parse::<usize>() {
                    let clock = Instant::now();
                    let nodes_count = crate::perft::perft(depth, &mut data.board);
                    println!(
                        "Number of nodes: {nodes_count}\nTime: {}ms",
                        clock.elapsed().as_millis()
                    );
                } else {
                    eprintln!("Invalid depth: {:?}", args);
                }
            }
            "d" => println!("{}", data.board),
            "bench" => bench(),
            _ => (),
        }

        input.clear();
    }
}

pub fn listen(shared: Arc<SharedData>) -> Receiver<String> {
    let (tx, rx) = channel::<String>();
    let mut input_buffer = String::new();

    thread::spawn(move || {
        loop {
            if std::io::stdin().read_line(&mut input_buffer).unwrap() == 0 {
                shared.status.stop();
            };

            match input_buffer.trim() {
                "quit" => {
                    shared.status.stop();
                    break;
                }
                "stop" => {
                    shared.status.stop();
                }
                _ => (),
            }

            let _ = tx.send(input_buffer.clone());
            input_buffer.clear();
        }
    });

    rx
}

pub fn position(args: &str, board: &mut Board) {
    if args.trim().is_empty() {
        eprintln!("Need to provide a valid argument!");
        return;
    }

    let (command, args) = args.split_once(" ").unwrap_or((args, ""));
    let (args, moves) = args.split_once("moves").unwrap_or((args, ""));

    match command.trim() {
        "startpos" => {
            *board = Board::from_fen(STARTING_FEN).unwrap();
        }
        "fen" => {
            if args.trim().is_empty() {
                eprintln!("Please provide a fen string");
                return;
            }
            if let Ok(b) = Board::from_fen(args) {
                *board = b;
            } else {
                eprintln!("Invalid FEN: {:?}", args.trim_end());
            }
        }
        _ => eprintln!("Not a valid position argument!"),
    }

    if !moves.trim().is_empty() {
        for m_str in moves.split_ascii_whitespace() {
            let result = board.parse_move(m_str);
            if let Ok(m) = result
                && board.make_move(m).is_err()
            {
                eprintln!("Illegal Move! {m}");
                return;
            }
        }
    }
}

pub fn go(args: &str, data: &mut SearchData) -> Option<MoveEntry> {
    let (command, args) = args.split_once(" ").unwrap_or((args, ""));
    if args.is_empty() {
        return search_runner(data);
    }

    match command.trim() {
        "depth" => {
            let (depth, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().depth = depth.trim().parse().unwrap_or(MAX_DEPTH - 1);
            go(args, data)
        }
        "wtime" => {
            //Example: go wtime 900000 btime 900000 winc 0 binc 0
            let (wtime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().wtime = wtime.trim().parse().unwrap_or(500);
            go(args, data)
        }
        "btime" => {
            let (btime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().btime = btime.trim().parse().unwrap_or(500);
            go(args, data)
        }
        "winc" => {
            let (winc, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().winc = winc.trim().parse().unwrap_or(0);
            go(args, data)
        }
        "binc" => {
            let (binc, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().binc = binc.trim().parse().unwrap_or(0);
            go(args, data)
        }
        "movestogo" => {
            let (movestogo, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().movestogo = movestogo.trim().parse().unwrap_or(0);
            go(args, data)
        }
        "movetime" => {
            let (movetime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().movetime = movetime.trim().parse().unwrap_or(0);
            go(args, data)
        }
        _ => go(args, data),
    }
}

pub fn uci() {
    println!("id name JustBot 1.0");
    println!("id author Hasan Fakih");
    println!("uciok");
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::types::constants::STARTING_FEN;

    #[test]
    fn test_parse_move() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        if let Ok(m) = board.parse_move("e2e4") {
            println!("bestmove {m}");
        }
    }

    #[test]
    fn test_parse_times() {
        go(
            "wtime 5000 btime 5000 winc 0 binc 0",
            &mut SearchData::default(),
        );
    }

    #[test]
    fn test_parse_go() {
        let mut data = SearchData::default();
        let bm = go(
            "wtime 5000 btime 5000 winc 5 binc 8 movetime 100",
            &mut data,
        );
        println!(
            "{:?}\nBestmove: {}",
            data.get_time_settings(),
            bm.unwrap().mv
        );
    }
}
