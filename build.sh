#! /bin/bash
echo "building program..."
cargo build --release

EX_PATH="$HOME/todo"
cp target/release/todo $EX_PATH 
echo "Copied the executable to $EX_PATH"

echo "Run .$EX_PATH"
