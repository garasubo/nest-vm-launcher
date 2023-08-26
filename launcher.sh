#!/bin/bash

script_path=$(readlink -f "$0")
manifest_path="$(dirname "$script_path")/Cargo.toml"

RUSTFLAGS=-Awarnings cargo run -q --release --manifest-path=$manifest_path -- "$@"
