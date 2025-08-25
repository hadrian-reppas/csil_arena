use std::collections::HashSet;

use crate::game::{Cell, Color};

pub struct Player {
    // Keep whatever state here
}

impl Player {
    pub fn new(_: &[[Cell; 16]; 16]) -> Self {
        Player { /* ... */ }
    }

    pub fn play(&mut self, board: &[[Cell; 16]; 16]) -> Color {
        let (score, color) = Color::ALL
            .iter()
            .map(|c| (score_color(board, *c), *c))
            .max()
            .unwrap();
        println!("Color {color:?} has score {score}");
        color
    }
}

fn score_color(board: &[[Cell; 16]; 16], color: Color) -> u32 {
    let mut seen = HashSet::new();
    let mut stack = Vec::new();
    for row in 0..16 {
        for col in 0..16 {
            if board[row][col] == Cell::Me {
                seen.insert((row, col));
                stack.push((row, col));
            }
        }
    }

    macro_rules! visit {
        ($row:expr, $col:expr) => {
            if (0..16).contains(&$row)
                && (0..16).contains(&$col)
                && !seen.contains(&($row, $col))
                && board[$row][$col] == Cell::Color(color)
            {
                seen.insert(($row, $col));
                stack.push(($row, $col));
            }
        };
    }

    let mut count = 0;
    while let Some((row, col)) = stack.pop() {
        if board[row][col] == Cell::Color(color) {
            count += 1;
        }
        visit!(row - 1, col);
        visit!(row + 1, col);
        visit!(row, col - 1);
        visit!(row, col + 1);
    }
    count
}
