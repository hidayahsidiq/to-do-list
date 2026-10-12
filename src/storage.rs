use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind};

pub const STORAGE_FILE: &str = "todo.txt";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[default]
    Pending,
    Done,
    TBD,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Done => "done",
            TaskStatus::TBD => "tbd",
        }
    }

    pub fn from_str(status: &str) -> TaskStatus {
        match status.trim().trim_matches('"').to_lowercase().as_str() {
            "pending" | "p" => TaskStatus::Pending,
            "done" | "d" => TaskStatus::Done,
            "tbd" | "td" | "todo" => TaskStatus::TBD,
            _ => TaskStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoItem {
    pub id: usize,
    pub description: String,
    pub status: TaskStatus,
    pub tag: Option<String>,
}

pub fn descriptions_match(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

pub fn next_available_id(items: &[TodoItem]) -> usize {
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

pub fn save_tasks(items: &[TodoItem], filename: &str) -> Result<(), std::io::Error> {
    let content: Vec<String> = items
        .iter()
        .map(|item| {
            format!(
                "{:02}. {} [{}]",
                item.id,
                item.description,
                item.status.as_str()
            )
        })
        .collect();

    fs::write(filename, content.join("\n"))
}

pub fn parse_task_line(line: &str, line_number: usize) -> Result<Option<TodoItem>, std::io::Error> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let invalid = |message: String| std::io::Error::new(ErrorKind::InvalidData, message);

    let (id_text, rest) = trimmed.split_once(". ").ok_or_else(|| {
        invalid(format!("invalid task format on line {}", line_number))
    })?;
    let id = id_text
        .trim()
        .parse::<usize>()
        .map_err(|_| invalid(format!("invalid task id on line {}", line_number)))?;
    let (description, status_with_bracket) = rest.rsplit_once(" [").ok_or_else(|| {
        invalid(format!("invalid task format on line {}", line_number))
    })?;
    let status = status_with_bracket
        .trim()
        .strip_suffix(']')
        .ok_or_else(|| invalid(format!("invalid task status on line {}", line_number)))?
        .trim();

    Ok(Some(TodoItem {
        id,
        description: description.to_string(),
        status: TaskStatus::from_str(status),
        tag: None,
    }))
}

pub fn load_tasks(filename: &str) -> Result<Vec<TodoItem>, std::io::Error> {
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let mut items = Vec::new();
    for (index, line) in content.lines().enumerate() {
        if let Some(item) = parse_task_line(line, index + 1)? {
            items.push(item);
        }
    }
    Ok(items)
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
