use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::ExitCode;

fn main() {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    };
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let filepath = "tasks.txt";
    tasks_exists(filepath);

    if let Some(program) = args.next() {
        let subcommand = args.next().unwrap_or("list".to_string());

        match subcommand.as_str() {
            "add" | "a" => {
                if let Some(argument) = args.next() {
                    add_new(argument, filepath);
                } else {
                    usage(&program);
                }
            }
            "list" | "l" => list_all(filepath),

            "done" | "d" => {
                if let Some(arg) = args.next() {
                    match arg.parse() {
                        Ok(index) => delete_todo(index, filepath),
                        Err(err) => {
                            eprintln!("Failed to parse index: {err}");
                            return Err(());
                        }
                    }
                } else {
                    usage(&program);
                }
            }
            "update" | "u" => {
                if let Some(arg) = args.next() {
                    match arg.parse() {
                        Ok(index) => {
                            if let Some(arg) = args.next() {
                                update_task(index, arg, filepath);
                            } else {
                                usage(&program);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to parse index: {err}");
                            return Err(());
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

    eprintln!("Failed to parse program arguments");
    Err(())
}

fn usage(program: &str) {
    eprintln!("USAGE: {program} <subcommand> [task]");
    eprintln!("\th[elp]:\tShow usage");
    eprintln!("\ta[dd] <task>:\tAdd a new task");
    eprintln!("\tl[ist]:\tList all tasks");
    eprintln!("\td[one] <index>:\tDelete a task");
    eprintln!("\tu[pdate] <index> <new_task>:\tUpdate an existing task");
}

fn add_new(task: String, filepath: &str) {
    let mut file = BufWriter::new(
        OpenOptions::new()
            .append(true)
            .open(filepath)
            .expect("Failed to open {filepath}"),
    );
    writeln!(&mut file, "{task}").expect("ERROR: Failed to write new task");
    println!("New Task added");
}

fn list_all(filepath: &str) {
    let buf = read_file(filepath);
    if buf.is_empty() {
        println!("No Tasks!");
        return;
    }
    let mut index = 1;
    for line in buf.lines() {
        println!("{index}: {line:?}");
        index += 1;
    }
}

fn delete_todo(index: u32, filepath: &str) {
    let content = read_file(filepath);
    let mut i = 1;
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)
            .expect("ERROR: Failed to truncate {filepath}"),
    );

    for line in content.lines() {
        if i == index {
            println!("Task {i} Deleted");
            i += 1;
            continue;
        }
        writeln!(&mut writer, "{line}").expect("ERROR: Failed to write line: {line}");
        i += 1;
    }
}

fn update_task(index: u32, new_task: String, filepath: &str) {
    let mut i = 1;
    let buf = read_file(filepath);
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)
            .expect("ERROR: Failed to truncate {filepath}"),
    );

    for mut line in buf.lines() {
        if i == index {
            line = &new_task;
            println!("Task Updated");
            println!("{i}: {line}");
        }
        writeln!(&mut writer, "{line}").expect("ERROR: Failed to write line: {line}");
        i += 1;
    }
}

fn read_file(filepath: &str) -> String {
    let mut content = String::new();
    BufReader::new(File::open(filepath).unwrap())
        .read_to_string(&mut content)
        .unwrap();
    content
}
fn tasks_exists(filepath: &str) {
    match File::open(filepath) {
        Ok(_) => (),
        Err(_) => {
            File::create(filepath).expect("ERROR: Failed to cerate tasks file");
        }
    }
}
