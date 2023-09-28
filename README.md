# LD54

![soupchunk](https://github.com/mrspeaker/ld54/assets/129330/d2e12d52-34d0-42ad-9d8d-56f0dd034ced)

## See last deployed version:

[LD54](https://mrspeaker.github.io/ld54/)

## build & run

```rust
cargo install --path .
cargo build`
cargo run
```

## debug println!

I added a (debug print plugin)[https://github.com/nicopap/bevy-debug-text-overlay] which will draw text on the screen temporarily. Handy for debugging stuff rather than trying to read stdout.

Use it like `println!` except you can optionally pass it a `col:Color` for colours and `sec:u32` for how long to show it.

## Other repo I started messing with bevy

with lots of sprites:
https://github.com/mrspeaker/beaves
