
# Five in a row bot

This repo contains my xo / five in a row / ristinolla / gomoku bots. There are a few versions.

 - Rust: Monte carlo tree search. Works reasonably well.
 - Python: Machine learning self play. Does not work (yet hopefully)

Build debug versrion
```
cargo run
```

Build fast version
```
cargo build --release && time ./target/release/five-in-a-row
```

