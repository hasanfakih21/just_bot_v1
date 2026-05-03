use macroquad::prelude::*;
use crate::board::{BitBoard, print_board};

pub mod board;

#[macroquad::main("MyGame")]
async fn main() {
    let board_texture = load_texture("assets/board.png").await.unwrap();
    let bit_board = BitBoard::new();
    print_board(&bit_board.white_king);

    loop {
        draw_texture(&board_texture, 0.0, 0.0, WHITE);
        request_new_screen_size(768.0, 768.0);
        next_frame().await
    }
}
