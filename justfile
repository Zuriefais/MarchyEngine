# Run engine (uses engine.toml or shows module picker)
run:
    RUST_LOG=info cargo run --package zu_core

# Run engine in release mode
run_release:
    RUST_LOG=info cargo run --package zu_core --release

# Run with specific module
run_mod MODULE:
    RUST_LOG=info cargo run --package zu_core -- "{{MODULE}}"

# Build engine
build:
    cargo build --release --package zu_core

build_windows:
    cargo build --release --package zu_core --target x86_64-pc-windows-gnu

# Build WASM mod for wasm32-wasip2 target (Rust)
wasm:
    cargo build --package vampire_like_demo --target wasm32-wasip2 --release

# Build Go WASM demo
wasm_go:
    #!/usr/bin/env bash
    set -e
    cd games/go_demo
    echo "Building Go WASM..."
    tinygo build -o go_demo_nosched.wasm -target=wasip1 -scheduler=none -no-debug .
    echo "Creating Component Model..."
    wasm-tools component embed ../../crates/zurie_scripting/zurie_engine.wit go_demo_nosched.wasm --world zurie-mod -o go_demo_embedded.wasm
    wasm-tools component new go_demo_embedded.wasm --adapt wasi_snapshot_preview1=wasi_snapshot_preview1.reactor.wasm -o go_demo.wasm
    rm -f go_demo_nosched.wasm go_demo_embedded.wasm
    echo "Done! Run: just go"

# Build and run Go Snake demo
go: wasm_go
    RUST_LOG=info cargo run --package zu_core --release -- games/go_demo/go_demo.wasm

# Build and run Rust Vampire demo
rust: wasm
    RUST_LOG=info cargo run --package zu_core --release -- ./target/wasm32-wasip2/release/vampire_like_demo.wasm

# Build Python WASM demo
wasm_python:
    #!/usr/bin/env bash
    set -e
    cd games/python_flappy
    echo "Building Python WASM..."
    ~/.local/bin/componentize-py -d ../../crates/zurie_scripting/zurie_engine.wit -w zurie-mod componentize app -o flappy.wasm
    echo "Done! Run: just python"

# Build and run Python Flappy Bird demo
python: wasm_python
    RUST_LOG=info cargo run --package zu_core --release -- games/python_flappy/flappy.wasm

# Build WASM then run engine (legacy)
run_full: wasm
    RUST_LOG=info cargo run --package zu_core

# Build WASM then run engine in release mode (legacy)
run_full_release: wasm
    RUST_LOG=info cargo run --package zu_core --release
