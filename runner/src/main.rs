use std::collections::HashSet;
use std::time::Instant;

use anyhow::Result;
use wasmtime::{Config, Engine, Store, component::*};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView, p2::pipe::MemoryOutputPipe};

wasmtime::component::bindgen!({
    path: "../abi/wit",
    world: "arena-player"
});

struct Ctx {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl WasiView for Ctx {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            table: &mut self.table,
            ctx: &mut self.wasi,
        }
    }
}

struct Player {
    component: Component,
    store: Store<Ctx>,
    instance: ArenaPlayer,
    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
    player: Option<ResourceAny>,
}

const INITIAL_BOARD: [u8; 256] = [
    8, 5, 5, 1, 1, 5, 6, 4, 6, 0, 2, 6, 0, 2, 7, 7, 7, 3, 6, 3, 1, 2, 2, 6, 0, 3, 7, 7, 6, 2, 1, 7,
    4, 2, 1, 5, 4, 2, 7, 3, 2, 7, 6, 4, 2, 0, 7, 6, 2, 2, 0, 2, 4, 6, 4, 5, 3, 0, 1, 3, 2, 1, 2, 7,
    6, 1, 1, 7, 1, 2, 3, 5, 2, 1, 1, 3, 2, 1, 4, 3, 2, 4, 5, 0, 0, 0, 7, 7, 0, 5, 7, 4, 7, 5, 6, 7,
    7, 2, 5, 0, 7, 5, 4, 7, 7, 7, 3, 3, 6, 2, 1, 4, 2, 1, 4, 5, 2, 5, 3, 1, 5, 6, 2, 7, 1, 3, 3, 2,
    1, 7, 6, 3, 7, 7, 2, 4, 5, 6, 2, 6, 1, 2, 5, 2, 3, 0, 0, 6, 0, 0, 7, 7, 3, 4, 2, 7, 2, 7, 5, 6,
    0, 5, 4, 3, 1, 3, 1, 4, 3, 2, 4, 1, 5, 7, 6, 5, 6, 4, 6, 0, 4, 7, 5, 0, 1, 5, 3, 7, 1, 0, 6, 6,
    2, 0, 6, 4, 4, 2, 5, 4, 2, 3, 5, 5, 1, 2, 0, 7, 6, 1, 3, 2, 7, 4, 0, 7, 7, 7, 3, 1, 5, 4, 3, 1,
    5, 5, 5, 4, 3, 2, 6, 3, 2, 4, 0, 3, 6, 5, 6, 2, 2, 2, 2, 6, 3, 6, 1, 6, 3, 6, 2, 7, 2, 6, 5, 9,
];
const COLOR_ANSI_CODES: &[&str] = &[
    "\x1b[31m\x1b[41m",
    "\x1b[38;5;9m\x1b[48;5;9m",
    "\x1b[38;5;11m\x1b[48;5;11m",
    "\x1b[32m\x1b[42m",
    "\x1b[38;5;14m\x1b[48;5;14m",
    "\x1b[34m\x1b[44m",
    "\x1b[35m\x1b[45m",
    "\x1b[38;5;13m\x1b[48;5;13m",
    "\x1b[37m\x1b[47m",
    "\x1b[48;5;8m",
];
const COLOR_NAMES: &[&str] = &[
    "red", "orange", "yellow", "green", "cyan", "blue", "purple", "pink",
];

impl Player {
    fn new(engine: &Engine, linker: &mut Linker<Ctx>, path: &str) -> Result<Self> {
        let component = Component::from_file(&engine, path)?;

        let stdout = MemoryOutputPipe::new(64 * 1024);
        let stderr = MemoryOutputPipe::new(64 * 1024);
        let wasi = WasiCtxBuilder::new()
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stderr.clone()))
            .build();

        let mut store = Store::new(
            &engine,
            Ctx {
                table: ResourceTable::new(),
                wasi,
            },
        );

        store.set_fuel(10_000_000)?; // Go needs fuel to initialize
        let instance = ArenaPlayer::instantiate(&mut store, &component, &linker)?;

        Ok(Self {
            component,
            store,
            instance,
            stdout,
            stderr,
            player: None,
        })
    }

    fn init(&mut self, bytes: &[u8], fuel: u64) -> Result<()> {
        assert!(
            self.player.is_none(),
            "Player::init() must only be called once"
        );
        self.store.set_fuel(fuel)?;
        self.player = Some(
            self.instance
                .csil_arena_api()
                .player()
                .call_constructor(&mut self.store, bytes)?,
        );
        Ok(())
    }

    fn play(&mut self, bytes: &[u8], fuel: u64) -> Result<i64> {
        self.store.set_fuel(fuel)?;
        self.instance.csil_arena_api().player().call_play(
            &mut self.store,
            self.player
                .expect("Player::init() must be called before Player::play()"),
            bytes,
        )
    }
}

fn main() -> Result<()> {
    let player1_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "players/greedy.wasm".into());
    let player2_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "players/rr.wasm".into());

    let mut board = INITIAL_BOARD;

    let engine = Engine::new(Config::new().wasm_component_model(true).consume_fuel(true))?;
    let mut linker = Linker::<Ctx>::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    let mut player1 = Player::new(&engine, &mut linker, &player1_path)?;
    let mut player2 = Player::new(&engine, &mut linker, &player2_path)?;

    player1.init(&board, 100_000_000)?;
    player2.init(&board, 100_000_000)?;

    println!("Initial board:");
    print_board(&board);

    let mut move_number = 0;
    while !game_over(&board) {
        if move_number % 2 == 0 {
            let start = Instant::now();
            let color = player1.play(&board, 100_000_000)?;
            println!(
                "Player 1 played {} after {:?}:",
                COLOR_NAMES[color as usize],
                start.elapsed()
            );
            do_move(&mut board, color as u8);
        } else {
            let mut inverted = invert_board(&board);
            let start = Instant::now();
            let color = player2.play(&inverted, 100_000_000)?;
            println!(
                "Player 2 played {} after {:?}:",
                COLOR_NAMES[color as usize],
                start.elapsed()
            );
            do_move(&mut inverted, color as u8);
            board = invert_board(&inverted);
        }

        print_board(&board);
        move_number += 1;
    }

    println!("Player 1 stdout: {:?}", player1.stdout.contents());
    println!("Player 1 stderr: {:?}", player1.stderr.contents());

    println!("Player 2 stdout: {:?}", player2.stdout.contents());
    println!("Player 2 stderr: {:?}", player2.stderr.contents());

    Ok(())
}

fn game_over(board: &[u8]) -> bool {
    board.iter().all(|b| *b >= 8)
}

fn do_move(board: &mut [u8], color: u8) {
    let mut seen = HashSet::from([0]);
    let mut stack = vec![0];
    while let Some(index) = stack.pop() {
        board[index] = 8;
        for offset in [-16, 16, -1, 1] {
            if (offset == -1 && index % 16 == 0) || (offset == 1 && index % 16 == 15) {
                continue;
            }
            let Ok(neighbor) = (index as isize + offset).try_into() else {
                continue;
            };
            if (0..256).contains(&neighbor)
                && !seen.contains(&neighbor)
                && (board[neighbor] == color || board[neighbor] == 8)
            {
                stack.push(neighbor);
                seen.insert(neighbor);
            }
        }
    }
}

fn invert_board(board: &[u8]) -> [u8; 256] {
    let mut inverted = [0; 256];
    for row in 0..16 {
        for col in 0..16 {
            let inverted_byte = match board[16 * row + col] {
                color @ 0..=7 => color,
                8 => 9,
                9 => 8,
                _ => unreachable!(),
            };
            inverted[16 * (15 - row) + (15 - col)] = inverted_byte;
        }
    }
    inverted
}

fn print_board(board: &[u8]) {
    for (i, byte) in board.iter().enumerate() {
        print!("{}  ", COLOR_ANSI_CODES[*byte as usize]);
        if i % 16 == 15 {
            println!("\x1b[m");
        }
    }
    println!();
}
