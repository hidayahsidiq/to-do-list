# To-Do List CLI

A simple command-line to-do list application written in Rust. It lets you add, list, complete, reopen, and delete tasks from a terminal.

## Requirements

Before running the project, install the following on Windows:

- **Windows 10 or Windows 11**
- **PowerShell** or another terminal
- **Rust and Cargo** (the Rust package manager)

### Install Rust on Windows

1. Open PowerShell.
2. Download and run the official Rust installer:

   ```powershell
   irm https://sh.rustup.rs -Method Get | iex
   ```

3. When prompted, choose the default installation.
4. Close and reopen PowerShell so the new PATH settings are loaded.
5. Confirm that Rust is installed:

   ```powershell
   rustc --version
   cargo --version
   ```

   You should see version information for both commands.

> This project uses Rust edition 2024, so use a current stable Rust installation. If `cargo` is not recognized after installation, restart your terminal or add Cargo's `bin` directory to your PATH.

## Get the project

If the project is already on your computer, continue to the next section.

If you received the project as a ZIP file:

1. Extract the ZIP file.
2. Open PowerShell.
3. Change to the extracted project directory. For this project, the path is:

   ```powershell
   cd C:\Users\sidiq\rust\CLI\to-do-list
   ```

   If you extracted it somewhere else, replace the path with the actual folder path.

You can confirm that you are in the correct directory by running:

```powershell
Get-ChildItem
```

The directory should contain `Cargo.toml` and the `src` folder.

## Build the project

From the project directory, run:

```powershell
cargo build --release
```

Cargo downloads the required dependencies (`clap` and `serde`) and creates the executable at:

```text
C:\Users\sidiq\rust\CLI\to-do-list\target\release\to-do-list.exe
```

The first build may take longer because dependencies are being downloaded and compiled.

## Run the project

### Run with Cargo

```powershell
cargo run -- --help
```

The first `--` belongs to Cargo and separates Cargo options from options for this application.

List the current tasks:

```powershell
cargo run -- list
```

### Run the built executable directly

```powershell
.\target\release\to-do-list.exe list
```

For a debug build, use:

```powershell
cargo build
.\target\debug\to-do-list.exe list
```

## Commands

Every command that changes or finds a task uses the `--desc` option. Put descriptions containing spaces inside double quotes.

### Add a task

```powershell
cargo run -- add --desc "Buy groceries"
```

### List all tasks

```powershell
cargo run -- list
```

Example output:

```text
1. Buy groceries [pending]
2. Read a book [done]
```

### Mark a task as done

```powershell
cargo run -- done --desc "Buy groceries"
```

### Reopen a completed task

```powershell
cargo run -- todo --desc "Buy groceries"
```

This changes the task status to `tbd`.

### Delete a task

```powershell
cargo run -- delete --desc "Buy groceries"
```

Matching is case-insensitive, so `BUY GROCERIES` and `Buy groceries` refer to the same description.

## Task storage

Tasks are saved automatically in a file named `todo.txt`. The file is created in the directory from which you run the command.

For example, when commands are run from this project directory, the file is:

```text
C:\Users\sidiq\rust\CLI\to-do-list\todo.txt
```

You can view it with:

```powershell
Get-Content .\todo.txt
```

Do not change the task-file format manually unless you know what you are doing. A typical line looks like this:

```text
01. Buy groceries [pending]
```

If `todo.txt` does not exist, the application starts with an empty task list.

## Useful checks

Format the Rust source:

```powershell
cargo fmt
```

Check that the code compiles:

```powershell
cargo check
```

Run the automated tests:

```powershell
cargo test --all-targets
```

## Troubleshooting

### `cargo` or `rustc` is not recognized

1. Close and reopen PowerShell.
2. Run:

   ```powershell
   cargo --version
   rustc --version
   ```

3. If the commands still fail, verify that Rust was installed and that the Cargo `bin` directory is in your PATH.

### Dependencies cannot be downloaded

Check your internet connection, then run the build again:

```powershell
cargo build --release
```

### `--desc is required`

The command needs a task description. Use quotes when the description contains spaces:

```powershell
cargo run -- add --desc "My task"
```

### The executable cannot be found

Build the project first:

```powershell
cargo build --release
```

Then run:

```powershell
.\target\release\to-do-list.exe list
```
