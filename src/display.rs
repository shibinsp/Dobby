use crate::state::TaskStatus;
use owo_colors::OwoColorize;

pub const PREFIX: &str = "[dobby]";

pub fn color_status(status: TaskStatus) -> String {
    match status {
        TaskStatus::Pending => status.as_str().yellow().to_string(),
        TaskStatus::InProgress => status.as_str().cyan().to_string(),
        TaskStatus::Completed => status.as_str().green().to_string(),
    }
}

pub fn short_id(id: &str) -> String {
    let length = std::cmp::min(6, id.len());
    id[..length].to_string()
}
