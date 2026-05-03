use macroquad::prelude::*;
use crate::board::{BitBoard, Square, print_board};

pub mod board;

#[macroquad::main("MyGame")]
async fn main() {
    let board_texture = load_texture("assets/board.png").await.unwrap();
    let bb_texture = load_texture("assets/bb.png").await.unwrap();

    let bit_board = BitBoard::new();
    print_board(&bit_board.white_king);

    loop {
        draw_texture(&board_texture, 0.0, 0.0, WHITE);
        request_new_screen_size(768.0, 768.0);
        draw_piece(0.0, 0.0, &bb_texture);
        next_frame().await
    }
}


fn draw_piece(x: f32, y: f32, texture: &Texture2D) {
    let params = DrawTextureParams {
        dest_size: Some(vec2(96.0, 96.0)),
        ..Default::default()
    };

    draw_texture_ex(texture, x, y, WHITE, params);
}