# Todo

## Project Overview

`todo` is a lightweight command-line note management tool for Unix systems. 
It lets you track tasks and quick notes directly from your shell without a database or external dependencies.

### Main Features
- **Add Notes**  
  Create new tasks or reminders  
- **Update Notes**  
  Modify existing note text by ID  
- **Delete Notes**  
  Remove unwanted entries  
- **List Notes**  
  View all pending tasks  
- **Mark Done**  
  Flag notes as completed  
- **Built-in Help**  
  Access usage instructions with `todo help`

### Why Use `todo`
- Zero configuration: works out of the box on any Unix shell  
- Fast, text-based storage: no setup overhead  
- Simple commands: intuitive syntax for day-to-day productivity  
- GPLv3 licensed: ensures freedom to run, modify, and share

### Basic Usage Examples
Install (make sure `todo` is in your PATH), then run:

# Add a new task
todo add "Review pull requests"

# List all tasks
todo list

# Update task #2
todo update 2 "Review backend pull requests"

# Mark task #1 as done
todo done 1

# Delete task #3
todo delete 3

# View help
todo help

### License
This project is distributed under the GNU General Public License v3.0. See the LICENSE file for full terms.
## Quick Start & Installation

Get the `todo` CLI up and running in seconds: install prerequisites, build or install the binary, then run your first command.

### Prerequisites

- Rust toolchain (rustc 1.50+ and Cargo)  
- Bash (for the provided build script)  
- Git (to clone the repo)

### 1. Clone the Repository

```bash
git clone https://github.com/juanmilkah/todo.git
cd todo
```

### 2. Build & Install

#### Option A: Automated Script (Linux)

Make the script executable and run it to compile in release mode and install to `/usr/local/bin/todo`:

```bash
chmod +x build.sh
./build.sh
```

By default, `build.sh` uses:
- `cargo build --release`  
- `sudo cp target/release/todo /usr/local/bin/todo`

To install to a custom directory (e.g. `~/.local/bin`):

```bash
EX_PATH="$HOME/.local/bin/todo" ./build.sh
```

#### Option B: Cargo Install

Use Cargo to install directly into your `$CARGO_HOME/bin`:

```bash
cargo install --path .
```

This places `todo` in `~/.cargo/bin`, which you should add to your `$PATH`.

### 3. Verify Installation

```bash
todo help
```

You should see:

```
todo 0.x.x
USAGE:
    todo <SUBCOMMAND>

SUBCOMMANDS:
    add         Add a new task
    list        List all tasks
    edit        Edit an existing task
    delete      Delete a task
    help        Print this message or the help of the given subcommand(s)
```

### 4. First Commands

#### Add a Task

```bash
todo add "Buy groceries"
// Output: Added task #1: Buy groceries
```

#### List Tasks

```bash
todo list
// Output:
// 1. Buy groceries
```

#### Edit a Task

Opens your default editor (`$EDITOR`) to modify task #1:

```bash
todo edit 1
```

#### Delete a Task

```bash
todo delete 1
// Output: Deleted task #1
```

### Troubleshooting & Tips

- If you see a “permission denied” error when copying to `/usr/local/bin`, ensure you have sudo privileges.  
- Confirm your `$PATH` includes the install location (`/usr/local/bin` or `~/.cargo/bin`).  
- To rebuild after code changes, run:  
  ```bash
  cargo build --release
  ```  
- On macOS or Windows WSL, adjust `build.sh` paths or use `cargo install --path .`.
## CLI Usage

Interact with the `todo` binary via simple subcommands. All data persists in `~/.tasks.bin` by default.

Usage  
```shell
todo [OPTIONS] <COMMAND> [ARGS...]
```

Global Options  
• `-h, --help`        Show help for a command  
• `-V, --version`     Print version info  

### Commands

#### help  
Show general help or command-specific usage.  
```shell
todo help
todo help new
```

#### list  
Display all current tasks with their IDs.  
```shell
$ todo list
1  [ ] Buy groceries
2  [ ] Write blog post
```

#### new  
Open your `$EDITOR` to compose a new task.  
- First line becomes the “head”  
- Remaining lines become the “body”  
- Empty file → aborts without creating a task  

```shell
$ todo new
# Opens $EDITOR. Save & exit with:
#   Title: Plan weekend trip
#   Notes:
#     - Book hotel
#     - Pack snacks

Task 3 Added!
```

#### get <ID>  
Edit an existing task in your `$EDITOR`.  
- Modify content and save → updates task  
- Leave file blank → deletes task  

```shell
# Edit task #2
todo get 2

# After save:
# Task 2 updated successfully

# To delete: open editor, clear all text, save
# → Task 2 removed
```

#### done <ID> [<ID>...]  
Mark one or more tasks as done (removes them and reindexes remaining tasks).  
```shell
# Mark task 1 done
todo done 1
# → Task 1 marked as done

# Batch mark
todo done 3 4 5
# → Tasks 3, 4, 5 marked as done
```

### Examples of Everyday Workflows

1. Quickly add a task:
   ```shell
   $ todo new
   # Edit, save, and exit your editor
   ```

2. List all outstanding tasks:
   ```shell
   todo list | column -t
   ```

3. Update a task’s description:
   ```shell
   todo get 2
   ```

4. Complete multiple tasks at once:
   ```shell
   todo done 2 4 7
   ```

5. Inspect help for advanced flags:
   ```shell
   todo help done
   ```

### Notes & Tips

- Editor defaults to `nvim` if `$EDITOR` is unset; set it via `export EDITOR=vim`.  
- Tasks always reindex on deletion/mark-done to keep IDs sequential.  
- Errors reading corrupted storage fall back to an empty list; saving errors bubble up immediately.
## Development Guide

This guide helps you navigate the codebase, storage format, dependency configuration, and contribution workflow for the `juanmilkah/todo` CLI task manager.

### Code Layout

All application logic resides in `src/main.rs`. Key sections:

1. Imports and type definitions  
2. `Command` enum: CLI actions  
3. `main()` function: argument parsing, storage setup, command dispatch, and persistence  
4. Handler functions: add, list, edit, delete, reindex, editor integration  

#### File Skeleton

```rust
use std::{collections::BTreeMap, fs, io, path::PathBuf};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    description: String,
}

#[derive(Parser)]
#[command(name = "todo", about = "A Commandline Tasks Manager")]
enum Command {
    New { description: String },
    List,
    Edit { id: u64 },
    Done { id: u64 },
    Clean,
}

fn main() -> io::Result<()> {
    let cmd = Command::parse();
    let storage = get_storage();
    let mut tasks = load_from_storage(&storage);

    match cmd {
        Command::New { description } => add_task(&mut tasks, description),
        Command::List => list_tasks(&tasks),
        Command::Edit { id } => edit_task(&mut tasks, id)?,
        Command::Done { id } => delete_task(&mut tasks, id),
        Command::Clean => reindex_tasks(&mut tasks),
    }

    save_to_storage(&storage, &tasks)
}
```

#### Extending Commands

1. Add a variant to `Command`.  
2. In `main()`, add a `match` arm for the variant.  
3. Implement a handler:

```rust
fn your_command(tasks: &mut BTreeMap<u64, Task>, /* args */) -> io::Result<()> {
    // your logic
    Ok(())
}
```

### Persistent Storage

Tasks persist in a binary file (`.tasks.bin`) using `bincode2`. This ensures fast (de)serialization and sorted task IDs via `BTreeMap<u64, Task>`.

#### Storage Path (`get_storage`)

```rust
fn get_storage() -> PathBuf {
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".tasks.bin")
}
```

- Uses the `home` crate to locate the user’s home directory.  
- Falls back to the current directory.

#### Loading Tasks (`load_from_storage`)

```rust
fn load_from_storage(storage: &PathBuf) -> BTreeMap<u64, Task> {
    if !storage.exists() {
        let _ = fs::File::create(storage).map_err(|e| eprintln!("ERROR: {e}"));
        return BTreeMap::new();
    }

    match fs::read(storage) {
        Ok(data) if data.is_empty() => BTreeMap::new(),
        Ok(data) => bincode2::deserialize(&data).unwrap_or_default(),
        Err(err) => {
            eprintln!("ERROR: {err}");
            BTreeMap::new()
        }
    }
}
```

- Creates file on first run.  
- Returns an empty map on I/O errors, empty data, or deserialization failures.

#### Saving Tasks (`save_to_storage`)

```rust
fn save_to_storage(storage: &PathBuf, tasks: &BTreeMap<u64, Task>) -> io::Result<()> {
    let encoded = bincode2::serialize(tasks)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("{err}")))?;
    fs::write(storage, encoded)
}
```

- Returns a `Result` for upstream error handling.  
- Use `?` in `main()` to propagate I/O errors.

#### Usage Pattern

```rust
let storage = get_storage();
let mut tasks = load_from_storage(&storage);
// … mutate tasks …
save_to_storage(&storage, &tasks)?;
```

### Cargo.toml Configuration

`Cargo.toml` defines package metadata, dependencies, and release optimizations.

#### Package Metadata

```toml
[package]
name        = "todo"
version     = "0.3.0"
edition     = "2024"
description = "A Commandline Tasks Manager"
authors     = ["Juan Milkah <…>"]
repository  = "https://github.com/juanmilkah/todo"
license     = "GPL-3.0-or-later"
```

- Update `version` with `cargo set-version`.  
- Ensure `license` matches your LICENSE file (SPDX ID).

#### Dependencies

```toml
[dependencies]
clap     = { version = "4.5", features = ["derive"] }
serde    = { version = "1.0", features = ["derive"] }
bincode2 = "2.0"
home     = "0.5"
tempfile = "3.19"
```

- Add new crates via `cargo add <crate> --features <flags>`.  
- Enable derive macros for `serde` and `clap`.

#### Release Profile

```toml
[profile.release]
opt-level     = "s"
lto           = "thin"
codegen-units = 1
strip         = true
panic         = "abort"
```

- Produces a small, fast CLI binary.  
- Use `cargo build --release` and verify size with `ls -lh target/release/todo`.

### Contributing

Follow these steps to set up your environment, ensure code quality, and submit changes.

1. Clone the repo  
   ```bash
   git clone https://github.com/juanmilkah/todo.git
   cd todo
   ```
2. Install Rust toolchain (Rust 2024 edition)  
   ```bash
   rustup override set stable
   ```
3. Format and lint  
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   ```
4. Run (and add) tests  
   ```bash
   cargo test
   ```
5. Create a feature branch  
   ```bash
   git checkout -b feature/your-feature
   ```
6. Commit conventionally  
   ```
   feat: add export-to-json command
   ```
7. Push and open a pull request against `main`.  

Key guidelines:
- Write clear commit messages.  
- Ensure `cargo fmt` and `clippy` pass.  
- Update documentation when you add or change features.  
- Discuss breaking changes in an issue before implementation.
