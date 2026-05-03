use macroquad::prelude::*;
use crate::board::BitBoard;

pub mod board;

#[macroquad::main("MyGame")]
async fn main() {
    let board_texture = load_texture("assets/board.png").await.unwrap();
    let bb_tex = load_texture("assets/bb.svg").await.unwrap();
    let bit_board = BitBoard::new();

    loop {
        draw_texture(&board_texture, 0.0, 0.0, WHITE);
        request_new_screen_size(768.0, 768.0);
        draw_texture(&bb_tex, 0.0, 0.0, WHITE);
        next_frame().await
    }
}
