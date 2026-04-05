use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::state::{PlanRecord, TaskRecord, TaskStatus};
use crate::storage::Storage;

const FOOTER_HINTS: &str =
    "tab agents   ctrl+p commands   q quit   r refresh   dobby --forge to open Forge";
const MAX_WORKSPACE_ENTRIES: usize = 64;
const MAX_WORKSPACE_DEPTH: usize = 2;

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = {
        let mut app = App::new()?;
        app.run(&mut terminal)
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

struct App {
    storage: Storage,
    plan: Option<PlanRecord>,
    workspace_root: PathBuf,
    workspace_entries: Vec<String>,
    workspace_state: ListState,
}

impl App {
    fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = Self {
            storage,
            plan: None,
            workspace_root,
            workspace_entries: Vec::new(),
            workspace_state: ListState::default(),
        };
        app.refresh()?;
        Ok(app)
    }

    fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => self.refresh()?,
                    KeyCode::Down | KeyCode::Char('j') => self.next_entry(),
                    KeyCode::Up | KeyCode::Char('k') => self.previous_entry(),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        let state = self.storage.load()?;
        self.plan = state.plan;
        self.workspace_entries = collect_workspace_entries(&self.workspace_root);
        self.sync_selection();
        Ok(())
    }

    fn sync_selection(&mut self) {
        if self.workspace_entries.is_empty() {
            self.workspace_state.select(None);
            return;
        }

        let current = self.workspace_state.selected().unwrap_or(0);
        self.workspace_state
            .select(Some(current.min(self.workspace_entries.len() - 1)));
    }

    fn next_entry(&mut self) {
        let len = self.workspace_entries.len();
        if len == 0 {
            return;
        }
        let next = match self.workspace_state.selected() {
            Some(idx) => (idx + 1) % len,
            None => 0,
        };
        self.workspace_state.select(Some(next));
    }

    fn previous_entry(&mut self) {
        let len = self.workspace_entries.len();
        if len == 0 {
            return;
        }
        let prev = match self.workspace_state.selected() {
            Some(0) | None => len - 1,
            Some(idx) => idx - 1,
        };
        self.workspace_state.select(Some(prev));
    }

    fn draw(&mut self, f: &mut Frame<'_>) {
        let frame_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(12), Constraint::Length(1)].as_ref())
            .split(f.size());

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(30),
            ])
            .split(frame_chunks[0]);

        self.draw_workspace(f, body_chunks[0]);
        self.draw_summary(f, body_chunks[1]);
        self.draw_context(f, body_chunks[2]);
        self.draw_footer(f, frame_chunks[1]);
    }

    fn draw_workspace(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Workspace")
            .border_style(Style::default().fg(Color::DarkGray));

        let items: Vec<ListItem> = if self.workspace_entries.is_empty() {
            vec![ListItem::new("No files detected in the current directory.")]
        } else {
            self.workspace_entries
                .iter()
                .map(|entry| ListItem::new(entry.clone()))
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› ");
        f.render_stateful_widget(list, area, &mut self.workspace_state);
    }

    fn draw_summary(&self, f: &mut Frame<'_>, area: Rect) {
        let summary = self.plan_stats();
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![
            Span::styled(
                "Thinking",
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .fg(Color::Magenta),
            ),
            Span::raw(": "),
            Span::raw(summary.status_message.clone()),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "Done!",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            ),
            Span::raw(format!(" {}", summary.next_action)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "Tests:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", summary.tests_line)),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                "Documentation:",
                Style::default()
                    .fg(Color::Rgb(255, 170, 0))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(" {}", summary.doc_line)),
        ]));

        if let Some(plan) = &self.plan {
            if !plan.milestones.is_empty() {
                lines.push(Line::from(Span::raw("")));
                lines.push(Line::from(Span::raw(format!(
                    "Milestones: {}",
                    plan.milestones.join(", ")
                ))));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Activity Stream")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn draw_context(&self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Context")
            .border_style(Style::default().fg(Color::DarkGray));

        let mut lines: Vec<Line> = Vec::new();
        if let Some(plan) = &self.plan {
            lines.push(Line::from(vec![Span::styled(
                plan.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )]));
            if let Some(desc) = &plan.description {
                lines.push(Line::from(desc.clone()));
            }
            lines.push(Line::from(""));
            let counts = self.plan_counts(plan);
            lines.push(Line::from(format!(
                "Tasks: {} total • {} pending • {} in progress • {} done",
                counts.total, counts.pending, counts.in_progress, counts.completed
            )));
            lines.push(Line::from(format!("Updated: {}", plan.updated_at)));
            lines.push(Line::from(format!("Created: {}", plan.created_at)));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Backlog",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )]));
            let mut task_lines = self.plan_task_lines(plan);
            lines.append(&mut task_lines);
        } else {
            lines.push(Line::from(
                "Welcome to Dobby. Initialize a plan to see project context.",
            ));
            lines.push(Line::from("Run `dobby plan init -n \"Feature\"` to begin."));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    fn draw_footer(&self, f: &mut Frame<'_>, area: Rect) {
        let footer = Paragraph::new(FOOTER_HINTS)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default());
        f.render_widget(footer, area);
    }

    fn plan_counts(&self, plan: &PlanRecord) -> PlanCounts {
        let mut counts = PlanCounts::default();
        counts.total = plan.tasks.len();
        for task in &plan.tasks {
            match task.status {
                TaskStatus::Pending => counts.pending += 1,
                TaskStatus::InProgress => counts.in_progress += 1,
                TaskStatus::Completed => counts.completed += 1,
            }
        }
        counts
    }

    fn plan_task_lines(&self, plan: &PlanRecord) -> Vec<Line<'static>> {
        if plan.tasks.is_empty() {
            return vec![Line::from(
                "No tasks yet. Use `dobby task add` to seed work.",
            )];
        }

        let mut lines = Vec::new();
        for (index, task) in plan.tasks.iter().take(6).enumerate() {
            lines.push(self.plan_task_line(index, task));
        }
        if plan.tasks.len() > 6 {
            lines.push(Line::from(format!(
                "…and {} more tasks. Use `dobby task list` for the full view.",
                plan.tasks.len() - 6
            )));
        }
        lines
    }

    fn plan_task_line(&self, index: usize, task: &TaskRecord) -> Line<'static> {
        let status_style = match task.status {
            TaskStatus::Pending => Style::default().fg(Color::Yellow),
            TaskStatus::InProgress => Style::default().fg(Color::Cyan),
            TaskStatus::Completed => Style::default().fg(Color::Green),
        };

        Line::from(vec![
            Span::styled(
                format!("{:>2}. ", index + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("[{}] ", self.status_label(task.status)),
                status_style,
            ),
            Span::raw(task.title.clone()),
            task.notes
                .as_ref()
                .map(|notes| Span::raw(format!(" — {}", notes)))
                .unwrap_or_else(|| Span::raw("")),
        ])
    }

    fn status_label(&self, status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
        }
    }

    fn plan_stats(&self) -> PlanSummary {
        match &self.plan {
            Some(plan) => {
                let counts = self.plan_counts(plan);
                PlanSummary {
                    status_message: if counts.total == 0 {
                        "Plan ready. Start by adding tasks.".to_string()
                    } else if counts.pending == 0 && counts.in_progress == 0 {
                        "All tasks completed. Consider shipping!".to_string()
                    } else {
                        format!(
                            "{} tasks pending, {} in progress.",
                            counts.pending, counts.in_progress
                        )
                    },
                    next_action: if counts.total == 0 {
                        "Use `dobby task add` to seed the backlog.".to_string()
                    } else {
                        "Review highlighted backlog items and press `r` after updates.".to_string()
                    },
                    tests_line: if counts.completed > 0 {
                        format!("All passing ({} tracked)", counts.completed)
                    } else {
                        "Not run yet".to_string()
                    },
                    doc_line: if plan
                        .description
                        .as_ref()
                        .map(|d| !d.is_empty())
                        .unwrap_or(false)
                    {
                        "README and plan details documented.".to_string()
                    } else {
                        "Add descriptions to capture decisions.".to_string()
                    },
                }
            }
            None => PlanSummary {
                status_message: "No implementation plan yet.".into(),
                next_action: "Create one with `dobby plan init` to unlock the dashboard.".into(),
                tests_line: "Blocked until a plan exists.".into(),
                doc_line: "Docs will be summarized once a plan is saved.".into(),
            },
        }
    }
}

#[derive(Default)]
struct PlanCounts {
    total: usize,
    pending: usize,
    in_progress: usize,
    completed: usize,
}

struct PlanSummary {
    status_message: String,
    next_action: String,
    tests_line: String,
    doc_line: String,
}

fn collect_workspace_entries(root: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    collect_dir(root, root, 0, &mut entries);
    if entries.is_empty() {
        entries.push("./".to_string());
    }
    entries
}

fn collect_dir(root: &Path, dir: &Path, depth: usize, entries: &mut Vec<String>) {
    if depth > MAX_WORKSPACE_DEPTH || entries.len() >= MAX_WORKSPACE_ENTRIES {
        return;
    }

    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<_> = read_dir.filter_map(|entry| entry.ok()).collect();
    children.sort_by_key(|entry| entry.file_name());

    for entry in children {
        if entries.len() >= MAX_WORKSPACE_ENTRIES {
            break;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let display = if rel.as_os_str().is_empty() {
            "./".to_string()
        } else {
            format!("./{}", rel.display())
        };

        if path.is_dir() {
            entries.push(format!("{}/", display.trim_end_matches('/')));
            collect_dir(root, &path, depth + 1, entries);
        } else {
            entries.push(display);
        }
    }
}
