use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use crate::board::Board;
use crate::search::data::{SearchData, SharedData};
use crate::threads::ThreadPool;
use crate::tools::bench::bench;
use crate::tools::datagen::generate_random_openings;
use crate::types::*;

pub fn input_loop(cli_args: String) {
    let mut data = SearchData::default();
    let mut thread_amount = 1;
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
            "setoption" => set_option(args, &mut data, &mut thread_amount),
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

                if let Some(e) = go(args, &mut data, thread_amount, false) {
                    println!("bestmove {}", e.mv);
                }
            }
            "quit" => break,
            "perft" => {
                if let Ok(depth) = args.trim().parse::<usize>() {
                    crate::tools::perft::perft(depth, &mut data.board);
                } else {
                    eprintln!("Invalid depth: {:?}", args);
                }
            }
            "d" => println!("{}", data.board),
            "bench" => {
                let (total_node_count, nps) = bench();
                println!("{} nodes {} nps", total_node_count, nps);
                break;
            }
            "genfens" => {
                genfens(args);
            }
            _ => (),
        }

        if input.contains("quit") {
            break;
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
                break;
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
            if let Ok(m) = result {
                board.make_move(m);
            } else {
                eprintln!("Illegal Move!");
                return;
            }
        }
    }
}

pub fn set_option(args: &str, data: &mut SearchData, thread_amoount: &mut usize) {
    let args = args.to_ascii_lowercase();
    let args: Vec<&str> = args.split_ascii_whitespace().collect();
    match args.as_slice() {
        ["name", "hash", "value", amount] => {
            let amount = amount.parse::<usize>().unwrap_or(16);
            data.shared.tt.resize(amount);
            println!("info string Resized TT to {amount} mb");
        }
        ["name", "threads", "value", amount] => {
            let amount = amount.parse::<usize>().unwrap_or(1);
            *thread_amoount = amount;
        }
        ["name", "clear", "hash"] => {
            data.shared.tt.clear();
            println!("info string TT cleared");
        }
        _ => eprintln!("Unkown option"),
    }
}

pub fn go(
    args: &str,
    data: &mut SearchData,
    thread_amoount: usize,
    mute: bool,
) -> Option<MoveEntry> {
    let (command, args) = args.split_once(" ").unwrap_or((args, ""));
    if args.is_empty() {
        let mut thread_pool = ThreadPool::new(
            data.board.clone(),
            data.time.clone(),
            data.shared.clone(),
            thread_amoount,
        );
        return thread_pool.start(data.shared.clone(), mute);
    }

    match command.trim() {
        "depth" => {
            let (depth, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().depth = depth.trim().parse().unwrap_or(MAX_DEPTH - 1);
            go(args, data, thread_amoount, mute)
        }
        "wtime" => {
            //Example: go wtime 900000 btime 900000 winc 0 binc 0
            let (wtime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().wtime = wtime.trim().parse().unwrap_or(500);
            go(args, data, thread_amoount, mute)
        }
        "btime" => {
            let (btime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().btime = btime.trim().parse().unwrap_or(500);
            go(args, data, thread_amoount, mute)
        }
        "winc" => {
            let (winc, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().winc = winc.trim().parse().unwrap_or(0);
            go(args, data, thread_amoount, mute)
        }
        "binc" => {
            let (binc, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().binc = binc.trim().parse().unwrap_or(0);
            go(args, data, thread_amoount, mute)
        }
        "movestogo" => {
            let (movestogo, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().movestogo = movestogo.trim().parse().unwrap_or(0);
            go(args, data, thread_amoount, mute)
        }
        "movetime" => {
            let (movetime, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().movetime = movetime.trim().parse().unwrap_or(0);
            go(args, data, thread_amoount, mute)
        }
        "nodes" => {
            let (nodes, args) = args.split_once(" ").unwrap_or((args, ""));
            data.get_time_settings().nodes = nodes.trim().parse().unwrap_or(0);
            data.time.set_nodes_limit();
            go(args, data, thread_amoount, mute)
        }
        _ => go(args, data, thread_amoount, mute),
    }
}

pub fn uci() {
    println!("id name JustBot 0.2.0");
    println!("id author Hasan Fakih");
    println!("option name Threads type spin default 1 min 1 max 32");
    println!("option name Hash type spin default 16 min 1 max 512");
    println!("option name Clear Hash type button");
    println!("uciok");
}

pub fn genfens(args: &str) {
    let args = args.to_ascii_lowercase();
    let args: Vec<&str> = args.split_ascii_whitespace().collect();
    let mut amount = 0;
    let mut seed = 0;

    match args.as_slice() {
        [n, "seed", s, ..] => {
            amount = n.parse::<usize>().unwrap_or(0);
            seed = s.parse::<u64>().unwrap_or(0);
        }
        [n, ..] => {
            amount = n.parse::<usize>().unwrap_or(0);
        }
        _ => (),
    }

    let book = generate_random_openings(amount, 8, seed);
    for opening in book {
        println!("info string genfens {}", opening);
    }
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
            1,
            false,
        );
    }

    #[test]
    fn test_parse_go() {
        let mut data = SearchData::default();
        let bm = go(
            "wtime 5000 btime 5000 winc 5 binc 8 movetime 100",
            &mut data,
            1,
            false,
        );
        println!(
            "{:?}\nBestmove: {}",
            data.get_time_settings(),
            bm.unwrap().mv
        );
    }

    #[test]
    fn test_set_option() {
        let mut data = SearchData::default();
        let mut thread_amount = 1;
        set_option("name Hash value 32", &mut data, &mut thread_amount);
    }
}
