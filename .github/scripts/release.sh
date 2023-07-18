#!/bin/bash

echo "Building..."
cargo build --release
echo "Generating documentation..."
cargo doc --no-deps
echo "Publishing..."
cargo publish --token "${CRATES_IO_TOKEN}"
