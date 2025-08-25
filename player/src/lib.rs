mod game;
mod player;

use std::sync::Mutex;

use game::{Cell, Color};

wit_bindgen::generate!({
    path: "../abi/wit",
    world: "arena-player"
});

fn parse_board(bytes: &[u8]) -> [[Cell; 16]; 16] {
    assert_eq!(bytes.len(), 256, "input bytes must have length 256");
    let mut board = [[Cell::Color(Color::Red); 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            let b = bytes[i * 16 + j];
            board[i][j] = Cell::from_u8(b).expect("input bytes must be in the range [0, 9]");
        }
    }
    board
}

struct PlayerWrapper(Mutex<player::Player>);

impl exports::csil::arena::api::GuestPlayer for PlayerWrapper {
    fn new(bytes: Vec<u8>) -> Self {
        let board = parse_board(&bytes);
        let player = player::Player::new(&board);
        Self(Mutex::new(player))
    }

    fn play(&self, bytes: Vec<u8>) -> i64 {
        let board = parse_board(&bytes);
        self.0.lock().unwrap().play(&board) as i64
    }
}

impl exports::csil::arena::api::Guest for PlayerWrapper {
    type Player = PlayerWrapper;
}

export!(PlayerWrapper);
