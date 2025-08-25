# CSIL Arena Wasm POC

Seeing what it would look like to use Wasm for CSIL Arena. It's a pretty clean solution,
but we could only support compiled langauges which excludes Python and JS/TS devs.

## Building a Player

In the `player` crate, you can modify `player/src/player.rs` to change your bot's behavior.
To compile the Wasm module, run

```
cargo build --target wasm32-wasip2 --release
```

The compiled file is at `target/wasm32-wasip2/release/player.wasm`. Copy this into the
`runner/players` directory.

## Running the Game

To run the game with the default players (`greedy.wasm` and `rr.wasm`), run

```
cargo run --release
```

in the `runner` directory. To specify paths to your own `.wasm` files, use

```
cargo run --release -- [player1] [player2]
```
