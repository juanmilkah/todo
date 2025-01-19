#! /bin/bash
echo "building program..."
cargo build --release

# Linux build
EX_PATH="/usr/bin/todo"
sudo cp target/release/todo $EX_PATH 
echo "Copied the executable to $EX_PATH"

echo "Run .$EX_PATH"
