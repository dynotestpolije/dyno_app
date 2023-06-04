#!/usr/bin/env bash

set -eux

cargo check --all-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --  -D warnings -W clippy::all
