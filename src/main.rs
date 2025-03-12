use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(version = VERSION, about = "A Minimalistic task manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add one or more new tasks
    Add {
        /// Task description(s)
        tasks: Vec<String>,
    },

    /// List all tasks
    List,

    /// Delete task(s) by their indices, or pass "all" to delete everything
    Done {
        /// Task index(es) to delete, or "all" to delete all tasks
        indices: Vec<String>,
    },

    /// Update a task at the given index
    Update {
        /// The index of the task to update
        index: u32,
        /// The new task description
        new_task: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let home = home::home_dir().unwrap_or(PathBuf::from("."));
    let filepath = home.join(".tasks.txt").to_string_lossy().to_string();

    tasks_exists(&filepath)?;

    match args.command {
        Commands::List => list_all(&filepath),
        Commands::Add { tasks } => add_new(tasks, &filepath),
        Commands::Done { indices } => {
            if indices.len() == 1 && indices[0] == "all" {
                delete_all(&filepath)
            } else {
                let mut indices_vec = Vec::new();

                for idx in indices {
                    match idx.parse::<u32>() {
                        Ok(value) => indices_vec.push(value),
                        Err(e) => eprintln!("Failed to parse index: {}: {}", idx, e),
                    }
                }

                delete_todos(indices_vec, &filepath)
            }
        }
        Commands::Update { index, new_task } => update_task(index, new_task, &filepath),
    }
}

fn add_new(tasks: Vec<String>, filepath: &str) -> Result<()> {
    let content = match fs::read_to_string(filepath) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to read {filepath}: {err}");
            return Err(err);
        }
    };

    let mut count = content.lines().count();

    let mut file = BufWriter::new(OpenOptions::new().read(true).append(true).open(filepath)?);
    for task in tasks {
        writeln!(&mut file, "{task}")?;
        count += 1;
        println!("Task {count} Added");
    }

    Ok(())
}

fn list_all(filepath: &str) -> Result<()> {
    let buf = match fs::read_to_string(filepath) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to read {filepath}: {err}");
            return Err(err);
        }
    };
    if buf.is_empty() {
        println!("No Tasks!");
        return Ok(());
    }
    let mut index = 1;
    for line in buf.lines() {
        println!("{index}: {line:?}");
        index += 1;
    }

    Ok(())
}

fn delete_all(filepath: &str) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filepath)?;
    println!("All Tasks Deleted!");
    Ok(())
}

fn delete_todos(indexes: Vec<u32>, filepath: &str) -> Result<()> {
    let content = match fs::read_to_string(filepath) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to read {filepath}: {err}");
            return Err(err);
        }
    };
    let mut i = 1;
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)?,
    );

    for line in content.lines() {
        if indexes.contains(&i) {
            println!("Task {i} Deleted");
            i += 1;
            continue;
        }
        let _ = writeln!(&mut writer, "{line}").map_err(|err| {
            eprintln!("Failed to write line: {line} :{err}");
        });
        i += 1;
    }

    Ok(())
}

fn update_task(index: u32, new_task: String, filepath: &str) -> Result<()> {
    let mut i = 1;
    let buf = match fs::read_to_string(filepath) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("Failed to read {filepath}: {err}");
            return Err(err);
        }
    };
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)?,
    );

    for mut line in buf.lines() {
        if i == index {
            line = &new_task;
            println!("Task Updated");
            println!("{i}: {line}");
        }

        let _ = writeln!(&mut writer, "{line}").map_err(|err| {
            eprintln!("Failed to write line: {line} :{err}");
        });
        i += 1;
    }

    Ok(())
}

fn tasks_exists(filepath: &str) -> Result<()> {
    match fs::exists(filepath) {
        Ok(_) => return Ok(()),
        Err(_) => File::create(filepath)?,
    };

    Ok(())
}
