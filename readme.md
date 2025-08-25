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

# Adding a new task

```bash
# This opens your `$EDITOR` to compose a new task.  
# First line becomes the “head”  
# Remaining lines become the “body”  
# Empty file → aborts without creating a task  
todo new

# This does it in one shot
todo add "Review pull requests" "The code looks good to me"
```

# Get task by Id
```bash
# Edit an existing task in your `$EDITOR`.  
# Modify content and save → updates task  
# Leave file blank → deletes task  
todo get 1
```

# List all tasks
```bash
todo list
```

# Delete one of more tasks
```bash
todo done 1 2
```

# View help
```bash
todo help
```


## Build and Install
### Automated Script (Linux)

Make the script executable and run it to compile in release mode and install to `/usr/local/bin/todo`:

```bash
chmod +x build.sh
./build.sh
```

By default, `build.sh` uses:
- `cargo build --release`  
- `sudo cp target/release/todo /usr/local/bin/todo`

### Development

The development mode can be set via environment variables. This creates an alternative database file `$HOME/.dev_tasks.bin`
for the tasks processes in development mode, preventing data corruption of the global tasks `$HOME/.tasks.bin`.
```bash
ENVIRONMENT="DEVELOPMENT" todo new
```

### License
This project is distributed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for full terms.
