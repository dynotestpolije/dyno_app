#!/usr/bin/env bash

set -eux

cargo check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --  -D warnings -W clippy::all
# cargo test --all-targets --all-features
# cargo test --doc
