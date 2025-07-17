use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Result, Write};
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
    New {
        /// Create a single line task
        head: Option<String>,
        body: Option<String>,
    },

    /// List all tasks heads
    List,

    /// Get && Update a task
    Get {
        /// Task Id
        id: u64,
    },

    /// Delete task(s) by their id
    Done {
        /// Task id(s) to delete.
        indices: Vec<u64>,
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

fn load_from_storage(storage: &PathBuf) -> BTreeMap<u64, Task> {
    if !storage.exists() {
        let _ = File::create(storage).map_err(|err| eprintln!("ERROR: {err}"));
    }

    match fs::read(storage) {
        Ok(data) => {
            if data.is_empty() {
                return BTreeMap::new();
            }

            let data: BTreeMap<u64, Task> = match bincode2::deserialize(&data) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    BTreeMap::new()
                }
            };

            data
        }
        Err(err) => {
            eprintln!("ERROR: {err}");
            BTreeMap::new()
        }
    }
}

fn save_to_storage(storage: &PathBuf, tasks: &BTreeMap<u64, Task>) -> Result<()> {
    match bincode2::serialize(&tasks) {
        Ok(encoded) => fs::write(storage, encoded),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialise tasks: {err}"),
        )),
    }
}

fn add_one(head: Option<String>, body: Option<String>, tasks: &mut BTreeMap<u64, Task>) {
    let new_id = tasks.keys().next_back().map_or(1, |&id| id + 1);
    let head = head.unwrap_or_default();
    let body = body.unwrap_or_default();
    let new_task = Task {
        id: new_id,
        head,
        body,
    };
    tasks.insert(new_id, new_task);
    println!("Task {new_id} added!");
}

fn add_new(tasks: &mut BTreeMap<u64, Task>) -> Result<()> {
    let file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = file.path().to_path_buf();

    let editor = std::env::var("EDITOR").unwrap_or("nvim".to_string());

    let status = process::Command::new(&editor).arg(&temp_path).status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "{editor} exited with non zero status"
        )));
    }

    let content = fs::read_to_string(&temp_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        println!("New Task aborted!");
        return Ok(());
    }

    let head = lines[0].to_string();

    let body = if lines.len() > 1 {
        lines[1..].join("\n")
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
    println!("Task {new_id} Added!");
    Ok(())
}

fn list_all(tasks: &BTreeMap<u64, Task>) {
    if tasks.is_empty() {
        println!("No Tasks");
    } else {
        for task in tasks.values() {
            if task.body.is_empty() {
                println!("{}. {}", task.id, task.head);
            } else {
                println!("{}. HEAD: {}", task.id, task.head);
            }
        }
    }
}

fn delete_todos(indices: &[u64], tasks: &mut BTreeMap<u64, Task>) {
    let mut deleted_any = false;
    for id in indices {
        if tasks.remove(id).is_some() {
            println!("Marked task {id} as done!");
            deleted_any = true;
        } else {
            println!("Task {id} not found!");
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

fn get_task(index: u64, tasks: &mut BTreeMap<u64, Task>) -> Result<()> {
    if !tasks.contains_key(&index) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Task with id {index} not found"),
        ));
    }

    let current_task = tasks.get(&index).unwrap();

    let mut temp_file = tempfile::NamedTempFile::new()?;
    writeln!(temp_file, "{}", current_task.head)?;
    write!(temp_file, "{}", current_task.body)?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_path_buf();

    let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());

    let status = process::Command::new(&editor).arg(&temp_path).status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "{editor} exited with non zero status"
        )));
    }

    let content = fs::read_to_string(&temp_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        delete_todos(&[index], tasks);
        return Ok(());
    }

    let new_head = lines[0].to_string();

    let new_body = if lines.len() > 1 {
        lines[1..].join("\n")
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

fn main() -> Result<()> {
    let args = Cli::parse();
    let storage = get_storage();
    let mut tasks = load_from_storage(&storage);

    match args.command {
        Commands::List => {
            list_all(&tasks);
            return Ok(());
        }
        Commands::Get { id } => get_task(id, &mut tasks)?,
        Commands::New { head, body } => {
            if head.is_none() && body.is_none() {
                add_new(&mut tasks)?;
            } else {
                add_one(head, body, &mut tasks);
            }
        }
        Commands::Done { indices } => {
            delete_todos(&indices, &mut tasks);
        }
    };

    save_to_storage(&storage, &tasks)?;

    Ok(())
}
