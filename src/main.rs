use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Result, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};

fn main() {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    };
}

fn entry() -> Result<()> {
    let mut args = env::args().peekable();
    let home = home::home_dir().unwrap_or(PathBuf::from("."));
    let filepath = home.join(".tasks.txt").to_string_lossy().to_string();

    tasks_exists(&filepath)?;

    if let Some(program) = args.next() {
        let subcommand = args.next().unwrap_or("list".to_string());

        match subcommand.as_str() {
            "add" | "a" | "new" | "n" => {
                let mut tasks: Vec<String> = Vec::new();
                while let Some(argument) = &args.next() {
                    tasks.push(argument.to_string());
                }
                add_new(tasks, &filepath)?;
            }
            "list" | "l" => list_all(&filepath)?,

            "done" | "d" => {
                if let Some(arg) = args.peek() {
                    if arg == "all" {
                        delete_all(&filepath)?;
                        return Ok(());
                    }
                }

                let mut indexes: Vec<u32> = Vec::new();
                while let Some(arg) = &args.next() {
                    match arg.parse() {
                        Ok(index) => indexes.push(index),
                        Err(err) => {
                            eprintln!("{err}: \"{arg}\"");
                        }
                    }
                }
                delete_todos(indexes, &filepath)?;
            }
            "update" | "u" => {
                if let Some(arg) = args.next() {
                    match arg.parse() {
                        Ok(index) => {
                            if let Some(arg) = args.next() {
                                update_task(index, arg, &filepath)?;
                            } else {
                                usage(&program);
                            }
                        }
                        Err(err) => {
                            eprintln!("{err}: \"{arg}\"");
                        }
                    }
                } else {
                    usage(&program);
                }
            }
            "help" | "-h" | "h" => usage(&program),
            _ => usage(&program),
        }

        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Failed to parse program arguments",
    ))
}

fn usage(program: &str) {
    eprintln!("USAGE: {program} <subcommand>");
    eprintln!("\th[elp]:                        Show usage");
    eprintln!("\ta[dd] | n[new] <task>:         Add a new task");
    eprintln!("\tl[ist]:                        List all tasks");
    eprintln!("\td[one] <..indexes> | all:      Delete a task");
    eprintln!("\tu[pdate] <index> <new_task>:   Update an existing task");
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

fn delete_all(filepath: &str) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(filepath)?;

    println!("All Tasks Deleted!");
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
