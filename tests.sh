#!/usr/bin/env bash

set -eux

cargo test --all-targets --all-features
cargo test --doc
