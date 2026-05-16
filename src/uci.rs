use crate::board::{Board, Piece, Square, constants::STARTING_FEN, moves::Move};

impl Board {
    pub fn parse_move(&self, move_string: &str) -> Result<Move, &str> {
        let from = Square::try_from(&move_string[0..2]).unwrap();
        let to = Square::try_from(&move_string[2..4]).unwrap();
        let mut promotion_piece: Option<Piece> = None;
        if move_string.len() > 4 {
            match &move_string[4..] {
                "n" => promotion_piece = Some(Piece::Knight),
                "b" => promotion_piece = Some(Piece::Bishop),
                "r" => promotion_piece = Some(Piece::Rook),
                "q" => promotion_piece = Some(Piece::Queen),
                _ => ()
            }
        }

        let move_list = self.generate_all_moves();
        if let Some(m) = move_list.iter().find(|e| e.get_from() == from && e.get_to() == to && e.get_promoted_piece() == promotion_piece) {
            Ok(*m)
        } else {
            Err("Invalid move string")
        }
    }
}

pub fn input_loop() {
    let mut board = Board::new();

    let mut input_buffer = String::new();
    loop {
        if std::io::stdin().read_line(&mut input_buffer).unwrap() == 0 {
            break;   
        }

        let (command, args) = input_buffer.split_once(" ").unwrap_or((&input_buffer, ""));

        match command {
            "position" => position(args, &mut board),
            _=> eprintln!("Not a valid command"),
        }
        
        input_buffer.clear();
    }
}

pub fn position(args: &str, board: &mut Board) {
    if args.is_empty() {
        eprintln!("Need to provide a valid argument!");
        return
    }

    let (command, args) = args.split_once(" ").unwrap_or((args, ""));
    let (args, moves) = args.split_once("moves").unwrap_or((args, ""));

    match command.trim() {
        "startpos" => {
            *board = Board::from_fen(STARTING_FEN);
            println!("{board}");
        },
        "fen" => {
            *board = Board::from_fen(args);
            println!("{board}");
        },
        _ => eprintln!("Not a valid position argument")
    }   

    if !moves.trim().is_empty() {
        for m_str in moves.split_ascii_whitespace() {
            if let Ok(m) = board.parse_move(m_str)
                && board.make_move(m).is_err() {eprintln!("Invalid move!")}
            else {
                eprintln!("Invalid move!")
            } 
        }

        println!("{board}");
    }
}

pub fn uci() {

}

#[cfg(test)]
pub mod tests {
    use crate::board::constants::STARTING_FEN;
    use super::*;

    #[test]
    fn test_parse_move() {
        let board = Board::from_fen(STARTING_FEN);
        if let Ok(m) = board.parse_move("e2e4") {
            println!("{m}");
        }
    }
}