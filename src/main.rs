use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, io::ErrorKind};

const STORAGE_FILE: &str = "todo.txt";

#[derive(Parser)]
#[command(name = "todo-list")]
#[command(author, version, about = "A simple CLI to-do list application", long_about = None)]
struct Args {
    /// Command to execute
    #[arg(value_enum)]
    command: Commands,

    /// Task description
    #[arg(long)]
    desc: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum TaskStatus {
    Pending,
    Done,
    TBD,
}

fn to_status_str(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "pending".to_string(),
        TaskStatus::Done => "done".to_string(),
        TaskStatus::TBD => "tbd".to_string(),
    }
}

fn from_status_str(status: &str) -> TaskStatus {
    match status.trim().trim_matches('"').to_lowercase().as_str() {
        "pending" | "p" => TaskStatus::Pending,
        "done" | "d" => TaskStatus::Done,
        "tbd" | "td" | "todo" => TaskStatus::TBD,
        _ => TaskStatus::Pending,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TodoItem {
    id: usize,
    description: String,
    status: TaskStatus,
    tag: Option<String>,
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: 0,
            description: String::new(),
            status: TaskStatus::Pending,
            tag: None,
        }
    }
}

fn descriptions_match(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn next_available_id(items: &[TodoItem]) -> usize {
    let mut id = items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    while items.iter().any(|item| item.id == id) {
        id = id.saturating_add(1);
    }
    id
}

fn save_tasks(items: &mut Vec<TodoItem>, filename: &str) -> Result<(), Box<dyn Error>> {
    let content: Vec<String> = items
        .iter()
        .map(|item| {
            format!(
                "{:02}. {} [{}]",
                item.id,
                item.description,
                to_status_str(&item.status)
            )
        })
        .collect();

    fs::write(filename, &content.join("\n"))?;
    Ok(())
}

fn parse_task_line(line: &str, line_number: usize) -> Result<Option<TodoItem>, Box<dyn Error>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let (id_text, rest) = trimmed.split_once(". ").ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("invalid task format on line {}", line_number),
        )
    })?;
    let id = id_text.trim().parse::<usize>()?;
    let (description, status_with_bracket) = rest.rsplit_once(" [").ok_or_else(|| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("invalid task format on line {}", line_number),
        )
    })?;
    let status = status_with_bracket
        .trim()
        .strip_suffix(']')
        .ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid task status on line {}", line_number),
            )
        })?
        .trim();

    Ok(Some(TodoItem {
        id,
        description: description.to_string(),
        status: from_status_str(status),
        tag: None,
    }))
}

fn load_tasks(filename: &str) -> Result<Vec<TodoItem>, Box<dyn Error>> {
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };

    let mut items = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if let Some(item) = parse_task_line(line, index + 1)? {
            items.push(item);
        }
    }
    Ok(items)
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
enum Commands {
    Add,
    Delete,
    List,
    Done,
    TODO,
}

fn required_description(args: &Args) -> Result<String, Box<dyn Error>> {
    let description = args.desc.clone().unwrap_or_default().trim().to_string();
    if description.is_empty() {
        return Err(std::io::Error::new(ErrorKind::InvalidInput, "--desc is required").into());
    }
    if description.contains('\n') || description.contains('\r') {
        return Err(
            std::io::Error::new(ErrorKind::InvalidInput, "--desc must be a single line").into(),
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

    let mut item = TodoItem::default();
    item.id = duplicate_id.unwrap_or_else(|| next_available_id(&items));
    item.description = description;
    items.push(item);
    save_tasks(&mut items, STORAGE_FILE)?;
    Ok(())
}

fn cmd_delete(args: &Args) -> Result<(), Box<dyn Error>> {
    let description = required_description(args)?;
    let mut items = load_tasks(STORAGE_FILE)?;
    items.retain(|item| !descriptions_match(&item.description, &description));
    save_tasks(&mut items, STORAGE_FILE)?;
    Ok(())
}

fn cmd_list(_args: &Args) -> Result<(), Box<dyn Error>> {
    let items = load_tasks(STORAGE_FILE)?;
    for item in items {
        println!(
            "{}. {} [{}]",
            item.id,
            item.description,
            to_status_str(&item.status)
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
    save_tasks(&mut items, STORAGE_FILE)?;
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
    save_tasks(&mut items, STORAGE_FILE)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Commands::Add => cmd_add(&args),
        Commands::Delete => cmd_delete(&args),
        Commands::List => cmd_list(&args),
        Commands::Done => cmd_done(&args),
        Commands::TODO => cmd_todo(&args),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_saved_task_lines_with_spaces() {
        let item = parse_task_line("07. Buy groceries [done]", 1)
            .unwrap()
            .unwrap();
        assert_eq!(item.id, 7);
        assert_eq!(item.description, "Buy groceries");
        assert_eq!(item.status, TaskStatus::Done);
    }

    #[test]
    fn assigns_a_unique_id_to_a_new_task() {
        let items = vec![
            TodoItem {
                id: 1,
                description: "one".to_string(),
                status: TaskStatus::Pending,
                tag: None,
            },
            TodoItem {
                id: 3,
                description: "three".to_string(),
                status: TaskStatus::Pending,
                tag: None,
            },
        ];
        assert_eq!(next_available_id(&items), 4);
    }

    #[test]
    fn description_matching_is_case_insensitive() {
        assert!(descriptions_match("Read the book", "read THE book"));
    }
}
