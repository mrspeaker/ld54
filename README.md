# LD54

![soupchunk](https://github.com/mrspeaker/ld54/assets/129330/d2e12d52-34d0-42ad-9d8d-56f0dd034ced)

## See last deployed version:

[LD54](https://mrspeaker.github.io/ld54/)

## Setup
Install [`rustup`](https://rustup.rs/).

Get the `nightly` toolchain and some of the tools.
```sh
# Nightly toolchain for unstable Rust features
rustup toolchain install nightly

# Linter
rustup component add clippy

# LSP with nightly compiler support
rustup component add rust-analyzer --toolchain nightly
```

For web builds:
```sh
rustup target add wasm32-unknown-unknown
```
and get [`trunk`](https://trunkrs.dev/).

Finally, I recommend [configuring rust-analyzer to show clippy lints](https://averylarsen.com/posts/enable-clippy-with-rust-analyzer/).

### Git Hook
Add the following git hook to `.git/hooks/pre-commit`:
```sh
#!/bin/sh
set -e
cargo fmt --check
cargo check
cargo clippy -- -D warnings -A clippy::pedantic
```

## Build

Run `./check` to lint and format code.

Choose one:
```sh
# Play the Native Debug Build
cargo run

# Play the Web Debug Build
trunk serve

# Build for Native Release
cargo build --release

# Build for Web Release
trunk build --release
```

## debug println!

I added a (debug print plugin)[https://github.com/nicopap/bevy-debug-text-overlay] which will draw text on the screen temporarily. Handy for debugging stuff rather than trying to read stdout.

Use it like `println!` except you can optionally pass it a `col:Color` for colours and `sec:u32` for how long to show it.

## Other repo I started messing with bevy

with lots of sprites:
https://github.com/mrspeaker/beaves
