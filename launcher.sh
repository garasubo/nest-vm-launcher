#!/bin/bash

script_path=$(readlink -f "$0")
manifest_path="$(dirname "$script_path")/Cargo.toml"

RUST_BACKTRACE=1 RUSTFLAGS=-Awarnings cargo run -q --release --manifest-path=$manifest_path -- "$@"
