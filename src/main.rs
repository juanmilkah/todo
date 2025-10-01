//! A Minimalistic task manager
/// The main crate for the task manager application.
/// It provides a command-line interface to manage tasks.
///
/// Implementation details
/// The `Storage` struct holds the state of the program
///
/// Tasks are stored in a contiguous array of intial length
/// `INTIAL_TASKS_ARRAY_LENGTH`
/// To create a new task entry, you first get a `Slot` index
/// into the tasks array.This is the position into which the
/// newly created task with be inserted at. The provided slot has
/// already been initialised with the default implementation of the
/// `Task` object.
///
/// The storage model also contains a mapping of task id's to their slot
/// indices in the tasks array. This mapping is used to retrive tasks
/// from the array in constant time. To get a task by its Id, first
/// look up the task id in the id_to_slot map, then index into the tasks
/// array at the provided slot index.
/// Deleting a task takes a similar approach to getting a task but proceeds
/// to re-index the id_to_slot map and issue new Ids to the remaining tasks.
/// The element at the removed index slot in the tasks array is replaced with
/// the default value of `Task`.
///
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{fs, process};

use clap::{Parser, Subcommand};
use flate2::Compression;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use serde::{Deserialize, Serialize};

/// The version of the application, retrieved from the Cargo.toml file.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The intial length of the tasks array in the storage
/// It is intialiased with the default values of `Task`
const INITIAL_TASKS_ARRAY_LENGTH: usize = 64;

/// The main command-line interface for the task manager.
#[derive(Parser)]
#[command(version = VERSION, about = "A Minimalistic task manager", long_about = None)]
struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    command: Commands,
}

/// The available commands for the task manager.
#[derive(Subcommand)]
enum Commands {
    /// Create new task.
    New {
        /// The Title of the task
        head: Option<String>,
        /// The Body section of the new task
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

/// An alias for the task id's type
type Id = u64;
/// An Alias for an index in the `Storage` store array of tasks
type Slot = usize;

/// A task with an id, head, and body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
struct Task {
    /// A unique identifier for the task
    id: Id,
    /// The head of the task.
    head: String,
    /// The body of the task.
    body: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Storage {
    /// An array of `Task` elements
    store: Vec<Task>,
    /// A mapping of the task id `Id` to the index slot in the
    /// tasks array
    id_to_slot: BTreeMap<Id, Slot>,
    /// The In-Memory storage has unsynched changes to the disk
    is_dirty: bool,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            store: vec![Task::default(); INITIAL_TASKS_ARRAY_LENGTH],
            id_to_slot: BTreeMap::new(),
            is_dirty: false,
        }
    }
}

/// Returns the path to the storage file.
/// If the storage file does not exist, it creates it.
fn get_storage() -> Result<PathBuf, String> {
    let home = home::home_dir().unwrap_or(PathBuf::from("."));
    // Supports development mode;
    let t_path = match std::env::var("ENVIRONMENT") {
        Ok(p) => match p.to_uppercase().as_str() {
            "DEVELOPMENT" => ".dev_tasks.bin",
            _ => ".tasks.bin",
        },
        _ => ".tasks.bin",
    };
    let storage = home.join(t_path);

    if !storage.exists() {
        File::create(&storage).map_err(|err| format!("Failed to create tasks.bin file: {err}"))?;
    }
    Ok(storage)
}

fn get_backup_path(storage_path: &Path) -> Result<PathBuf, String> {
    let mut backup_path = storage_path.to_path_buf().clone();
    backup_path.set_extension("bin.bak");
    if !backup_path.exists() {
        File::create(&backup_path).map_err(|err| format!("Failed to create backup file: {err}"))?;
    }
    Ok(backup_path)
}

/// Copies the file contents from the original storage path
/// to a backup location.
fn backup_data(storage_path: &PathBuf) {
    let backup_file = get_backup_path(storage_path)
        .map_err(|_err| eprintln!("Err Saving backup!"))
        .unwrap();

    match fs::copy(storage_path, &backup_file) {
        Err(err) => eprintln!(
            "Failed to save data to backup file: {}, {}",
            backup_file.display(),
            err
        ),
        Ok(_) => {
            println!("Data saved to a backup file: {}", backup_file.display());
        }
    }
}

/// Loads tasks from the storage file.
/// If the storage file is empty or the storage file is corrupted,
/// it returns `Storage::default()`.
fn load_from_storage(storage_path: &PathBuf) -> Storage {
    match fs::read(storage_path) {
        Ok(data) if data.is_empty() => Storage::default(),
        Ok(data) => {
            let data = decompress(&data)
                .map_err(|_| backup_data(storage_path))
                .unwrap_or_default();
            bincode2::deserialize(&data)
                .map_err(|_| backup_data(storage_path))
                .unwrap_or_default()
        }
        Err(err) => {
            eprintln!("ERROR: {err}");
            // save the old data to a backup file
            backup_data(storage_path);

            Storage::default()
        }
    }
}

/// Decompress the data from storage before deserialization
fn decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(Vec::new());
    decoder.write_all(data)?;
    decoder.finish()
}

/// Saves tasks to the storage file.
fn save_to_storage(storage_path: &PathBuf, data: &Storage) -> io::Result<()> {
    let encoded = bincode2::serialize(&data).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialise tasks: {err}"),
        )
    })?;

    let data = compress_data(&encoded)?;
    fs::write(storage_path, data)
}

/// Compress the data before saving to the storage file
fn compress_data(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

// Get the next available slot in the tasks array to insert a new entry
fn get_next_slot(data: &mut Storage) -> usize {
    let taken_slots = data.id_to_slot.values().cloned().collect::<Vec<usize>>();
    let l = data.store.len();
    let t = taken_slots.len();
    if t == l {
        let new_len = l * 2;
        data.store.resize(new_len, Task::default());
    }

    (0..data.store.len())
        .find(|i| !taken_slots.contains(i))
        .unwrap() // For Now I don't think would ever miss a slot
}

/// Adds a new task with a head and body.
fn add_one(head: Option<String>, body: Option<String>, data: &mut Storage) {
    let new_id = (data.id_to_slot.len() + 1) as u64;
    let head = head.unwrap_or_default().trim().to_string();
    let body = body.unwrap_or_default().trim().to_string();
    if head.is_empty() && body.is_empty() {
        return;
    }

    let new_task = Task {
        id: new_id,
        head,
        body,
    };
    let slot = get_next_slot(data);
    data.store[slot] = new_task;
    data.id_to_slot.insert(new_id, slot);
    data.is_dirty = true;
    println!("Task {new_id} added!");
}

/// Adds a new task by opening the default editor.
fn add_new(data: &mut Storage) -> Result<(), io::Error> {
    let file = tempfile::NamedTempFile::new().unwrap();
    let temp_path = file.path().to_path_buf();

    // I think everyone has at least nano
    let editor = std::env::var("EDITOR").unwrap_or("nano".to_string());

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

    add_one(Some(head), Some(body), data);
    Ok(())
}

/// Lists all tasks.
fn list_all(data: &Storage) {
    let slots = data.id_to_slot.values().cloned().collect::<Vec<Slot>>();
    if slots.is_empty() {
        println!("No Tasks!");
    }

    slots
        .iter()
        .map(|slot| &data.store[*slot])
        .for_each(|task| {
            if task.body.is_empty() {
                println!("{}. {}", task.id, task.head);
            } else {
                println!("{}. HEAD: {}", task.id, task.head);
            }
        })
}

/// Deletes todos by their indices.
/// If a task is deleted, it re-indexes the mapping of task id
/// to slots in the tasks array.
fn delete_todos(indices: &[u64], data: &mut Storage) {
    indices
        .iter()
        .filter(|id| data.id_to_slot.contains_key(id))
        .cloned()
        .collect::<Vec<Id>>()
        .into_iter()
        .for_each(|id| {
            let slot = *data.id_to_slot.get(&id).unwrap();
            let _ = std::mem::take(&mut data.store[slot]);
            let _ = data.id_to_slot.remove(&id);
            println!("Task {id} Deleted!");
            data.is_dirty = true;
        });

    if !data.is_dirty {
        return;
    }

    // Reindex the map to fill in the gaps from deleted tasks.
    let old_map = data.id_to_slot.clone();
    data.id_to_slot.clear();

    for (i, (_old_id, slot)) in old_map.iter().enumerate() {
        let new_id = i as u64 + 1;
        data.id_to_slot.insert(new_id, *slot);
        if let Some(elem) = data.store.get_mut(*slot) {
            elem.id = new_id;
        } else {
            unreachable!("A bug in the slot allocation implementation!");
        }
    }
}

/// Gets a task by its index and opens it in the default editor.
/// If the task is modified, it updates the task.
/// If the task is empty, it deletes the task.
fn get_task(index: u64, data: &mut Storage) -> Result<(), io::Error> {
    if !data.id_to_slot.contains_key(&index) {
        return Err(io::Error::other(format!("Task with id {index} not found")));
    }

    let slot = data.id_to_slot.get(&index).unwrap();
    let current_task = data.store.get(*slot).unwrap();

    let mut temp_file = tempfile::NamedTempFile::new()?;
    writeln!(temp_file, "{}", current_task.head)?;
    write!(temp_file, "{}", current_task.body)?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_path_buf();

    let editor = match std::env::var("EDITOR") {
        Ok(e) => e,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Env variable EDITOR not specified",
            ));
        }
    };

    let status = process::Command::new(&editor).arg(&temp_path).status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "{editor} exited with non zero status"
        )));
    }

    let content = fs::read_to_string(&temp_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        delete_todos(&[index], data);
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

    if *current_task != updated_task {
        println!("Task {} updated!", &updated_task.id);
        data.store[*slot] = updated_task;
        data.is_dirty = true;
    } else {
        println!("Task {} not updated!", &updated_task.id);
    }

    Ok(())
}

/// The main function for the task manager.
fn main() -> Result<(), io::Error> {
    // Parse the cli arguments
    let args = Cli::parse();

    // Get filepath for the storage
    // Create one if it does not exist
    let storage_path = match get_storage() {
        Ok(v) => v,
        Err(err) => return Err(io::Error::other(err)),
    };

    // Load data from the storage file
    // If the data is corrupted, copy it to a backup file and start
    // this session from a clean slate.
    let mut data = load_from_storage(&storage_path);

    match args.command {
        Commands::List => {
            list_all(&data);
            return Ok(());
        }

        Commands::Get { id } => get_task(id, &mut data)?,

        Commands::New { head, body } => {
            if head.is_none() && body.is_none() {
                add_new(&mut data)?;
            } else {
                add_one(head, body, &mut data);
            }
        }

        Commands::Done { indices } => {
            delete_todos(&indices, &mut data);
        }
    };

    // save the current state to disk
    if data.is_dirty {
        save_to_storage(&storage_path, &data)?;
    }

    Ok(())
}
