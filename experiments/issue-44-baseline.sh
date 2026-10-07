#!/usr/bin/env bash
# Run from the repository root. Before the fix, disabling default features
# still enabled CLI/codec std dependencies. Now only percent-encoding + alloc
# belongs to the production dependency graph.
set -eu
cargo tree --manifest-path rust/Cargo.toml --no-default-features -e normal,features
cargo build --manifest-path rust/Cargo.toml --lib --no-default-features --target wasm32-unknown-unknown
cargo test --manifest-path rust/Cargo.toml --test language --no-default-features
