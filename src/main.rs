use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, stdin, BufRead, Result, Write};
use std::path::PathBuf;
use std::{fs, process};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(version = VERSION, about = "A Minimalistic task manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create new task.
    New,

    /// List all tasks heads
    List,

    /// Get the full task
    Get {
        /// Task Id
        id: u64,
    },

    /// Delete task(s) by their id
    Done {
        /// Task id(s) to delete.
        indices: Vec<u64>,
    },

    /// Update a task at the given index
    Update {
        /// The index of the task to update
        index: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Task {
    id: u64,
    head: String,
    body: String,
}

fn get_storage() -> PathBuf {
    let home = home::home_dir().unwrap_or(PathBuf::from("."));
    home.join(".tasks.bin")
}

fn load_storage(storage: &PathBuf) -> BTreeMap<u64, Task> {
    if !storage.exists() {
        let _ = File::create(storage).map_err(|err| eprintln!("ERROR: {}", err));
    }

    match fs::read(storage) {
        Ok(data) => {
            if data.is_empty() {
                dbg!("empty!");
                return BTreeMap::new();
            }

            let data: BTreeMap<u64, Task> = match bincode2::deserialize(&data) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("ERROR: {}", err);
                    BTreeMap::new()
                }
            };

            data
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
            BTreeMap::new()
        }
    }
}

fn save_storage(storage: &PathBuf, tasks: &BTreeMap<u64, Task>) -> Result<()> {
    match bincode2::serialize(&tasks) {
        Ok(encoded) => fs::write(storage, encoded),
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to serialise tasks: {}", err),
            ))
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let storage = get_storage();
    let mut tasks = load_storage(&storage);

    match args.command {
        Commands::List => {
            list_all(&tasks);
            return Ok(());
        }
        Commands::Get { id } => {
            get_task(id, &tasks);
            return Ok(());
        }
        Commands::New => match add_new(&mut tasks) {
            Ok(_) => save_storage(&storage, &tasks)?,
            Err(err) => {
                eprintln!("ERROR: {}", err);
            }
        },
        Commands::Done { indices } => {
            delete_todos(&indices, &mut tasks);
            save_storage(&storage, &tasks)?;
        }
        Commands::Update { index } => match update_task(index, &mut tasks) {
            Ok(_) => save_storage(&storage, &tasks)?,
            Err(err) => eprintln!("ERROR: {}", err),
        },
    };

    Ok(())
}

fn add_new(tasks: &mut BTreeMap<u64, Task>) -> Result<()> {
    println!("Create new task. Enter heading on first line, body on subsequent lines.");
    println!("End input with an empty line:");
    let mut stdin = stdin().lock();
    let mut lines = Vec::new();

    loop {
        let mut line = String::new();
        let read = stdin.read_line(&mut line)?;
        if read <= 1 || line.trim().is_empty() {
            break;
        }
        lines.push(line);
    }

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No input provided",
        ));
    }

    let head = lines[0].trim().to_string();

    let body = if lines.len() > 1 {
        lines[1..].join("")
    } else {
        String::new()
    };

    let new_id = tasks.keys().next_back().map_or(1, |&id| id + 1);

    let new_task = Task {
        id: new_id,
        head,
        body,
    };
    tasks.insert(new_id, new_task);
    Ok(())
}

fn get_task(id: u64, tasks: &BTreeMap<u64, Task>) {
    if let Some(task) = tasks.get(&id) {
        println!("ID: {}", task.id);
        println!("Heading: {}", task.head);
        println!("Body:\n{}", task.body);
    } else {
        eprintln!("Task with ID {} not found", id);
    }
}

fn list_all(tasks: &BTreeMap<u64, Task>) {
    if tasks.is_empty() {
        println!("No Tasks");
    } else {
        for task in tasks.values().into_iter() {
            println!("ID: {} HEAD: {}", task.id, task.head);
        }
    }
}

fn delete_todos(indices: &[u64], tasks: &mut BTreeMap<u64, Task>) {
    let mut deleted_any = false;
    for id in indices {
        if tasks.remove(&id).is_some() {
            println!("Marked task {} as done!", id);
            deleted_any = true;
        } else {
            println!("Task {} not found!", id);
        }
    }

    if deleted_any {
        reindex_tasks(tasks);
    }
}

fn reindex_tasks(tasks: &mut BTreeMap<u64, Task>) {
    let mut values: Vec<Task> = tasks.values().cloned().collect();
    tasks.clear();

    values.sort_by_key(|task| task.id);

    for (new_id, task) in values.iter_mut().enumerate() {
        let new_id = new_id as u64 + 1;
        task.id = new_id;
        tasks.insert(new_id, task.clone());
    }
}

fn update_task(index: u64, tasks: &mut BTreeMap<u64, Task>) -> Result<()> {
    if !tasks.contains_key(&index) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task with id {} not found", index),
        ));
    }

    let current_task = tasks.get(&index).unwrap();

    let mut temp_file = tempfile::NamedTempFile::new()?;
    writeln!(temp_file, "{}", current_task.head)?;
    write!(temp_file, "{}", current_task.body)?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_path_buf();

    let editor = std::env::var("EDITOR").unwrap_or("nvim".to_string());

    let status = process::Command::new(&editor).arg(&temp_path).status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{} exited with non zero status", editor),
        ));
    }

    let content = fs::read_to_string(&temp_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Task cannot be empty",
        ));
    }

    let new_head = lines[0].to_string();

    let new_body = if lines.len() > 1 {
        lines[1..].join("")
    } else {
        String::new()
    };

    let updated_task = Task {
        id: index,
        head: new_head,
        body: new_body,
    };

    tasks.insert(index, updated_task);
    Ok(())
}
