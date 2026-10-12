mod gui;
mod storage;

use clap::{Parser, ValueEnum};
use std::error::Error;

use storage::{
    STORAGE_FILE, TaskStatus, TodoItem, descriptions_match, load_tasks, next_available_id,
    save_tasks,
};

#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author, version, about = "A simple to-do list application (CLI + GUI)", long_about = None)]
struct Args {
    /// Command to execute
    #[arg(value_enum)]
    command: Option<Commands>,

    /// Task description
    #[arg(long)]
    desc: Option<String>,

    /// Launch the graphical user interface (same as the `gui` command)
    #[arg(long)]
    gui: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum Commands {
    Add,
    Delete,
    List,
    Done,
    TODO,
    Gui,
}

fn required_description(args: &Args) -> Result<String, Box<dyn Error>> {
    let description = args.desc.clone().unwrap_or_default().trim().to_string();
    if description.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "--desc is required").into());
    }
    if description.contains('\n') || description.contains('\r') {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "--desc must be a single line")
                .into(),
        );
    }
    Ok(description)
}

fn cmd_add(args: &Args) -> Result<(), Box<dyn Error>> {
    let description = required_description(args)?;
    let mut items = load_tasks(STORAGE_FILE)?;

    let duplicate_id = items
        .iter()
        .filter(|item| descriptions_match(&item.description, &description))
        .map(|item| item.id)
        .last();

    let id = duplicate_id.unwrap_or_else(|| next_available_id(&items));
    items.push(TodoItem {
        id,
        description,
        status: TaskStatus::Pending,
        tag: None,
    });
    save_tasks(&items, STORAGE_FILE)?;
    Ok(())
}

fn cmd_delete(args: &Args) -> Result<(), Box<dyn Error>> {
    let description = required_description(args)?;
    let mut items = load_tasks(STORAGE_FILE)?;
    items.retain(|item| !descriptions_match(&item.description, &description));
    save_tasks(&items, STORAGE_FILE)?;
    Ok(())
}

fn cmd_list(_args: &Args) -> Result<(), Box<dyn Error>> {
    let items = load_tasks(STORAGE_FILE)?;
    for item in items {
        println!(
            "{}. {} [{}]",
            item.id,
            item.description,
            item.status.as_str()
        );
    }
    Ok(())
}

fn cmd_done(args: &Args) -> Result<(), Box<dyn Error>> {
    let description = required_description(args)?;
    let mut items = load_tasks(STORAGE_FILE)?;
    for item in &mut items {
        if descriptions_match(&item.description, &description) {
            item.status = TaskStatus::Done;
        }
    }
    save_tasks(&items, STORAGE_FILE)?;
    Ok(())
}

fn cmd_todo(args: &Args) -> Result<(), Box<dyn Error>> {
    let description = required_description(args)?;
    let mut items = load_tasks(STORAGE_FILE)?;
    for item in &mut items {
        if descriptions_match(&item.description, &description) {
            item.status = TaskStatus::TBD;
        }
    }
    save_tasks(&items, STORAGE_FILE)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Add) => cmd_add(&args),
        Some(Commands::Delete) => cmd_delete(&args),
        Some(Commands::List) => cmd_list(&args),
        Some(Commands::Done) => cmd_done(&args),
        Some(Commands::TODO) => cmd_todo(&args),
        Some(Commands::Gui) | None => {
            if matches!(args.command, None) && !args.gui {
                eprintln!("No command given. Try `--help`, or use `gui` / `--gui` to open the window.");
                std::process::exit(2);
            }
            gui::run();
            Ok(())
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
