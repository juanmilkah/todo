#! /bin/bash
echo "building program..."
cargo build --release

# Linux build
EX_PATH="/usr/local/bin/todo"
sudo cp target/release/todo $EX_PATH 
echo "Copied the executable to $EX_PATH"

echo "RUN: todo help"
