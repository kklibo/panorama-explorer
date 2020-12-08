#!/bin/sh

wasm-pack build --target web --out-name web
cargo run --release