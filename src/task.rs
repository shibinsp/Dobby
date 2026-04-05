use anyhow::{anyhow, bail, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use uuid::Uuid;

use crate::display::{color_status, short_id, PREFIX};
use crate::state::{DobbyState, PlanRecord, TaskRecord, TaskStatus};
use crate::storage::Storage;

#[derive(Parser)]
#[command(
    name = "dobby task",
    about = "Task workflow commands",
    arg_required_else_help = true
)]
pub struct TaskCli {
    #[command(subcommand)]
    command: TaskCommand,
}

#[derive(Subcommand)]
enum TaskCommand {
    /// Add a new task to the active plan.
    Add {
        /// Task title.
        title: String,

        /// Optional notes that describe the task in more detail.
        #[arg(long)]
        notes: Option<String>,
    },

    /// List all tasks (optionally filtered by status).
    List {
        /// Filter to a specific status (pending, in_progress, completed).
        #[arg(long)]
        status: Option<TaskStatus>,
    },

    /// Update a task's status by index, ID, or unique ID prefix.
    Status {
        /// Index (1-based) or ID/prefix identifying the task.
        target: String,

        /// New status to apply.
        status: TaskStatus,
    },
}

pub fn run(args: &[String]) -> Result<()> {
    let cli_args = task_cli_args(args);
    let cli = TaskCli::parse_from(cli_args);
    let storage = Storage::new()?;

    match cli.command {
        TaskCommand::Add { title, notes } => add_task(&storage, title, notes),
        TaskCommand::List { status } => list_tasks(&storage, status),
        TaskCommand::Status { target, status } => update_status(&storage, target, status),
    }
}

fn task_cli_args(args: &[String]) -> Vec<String> {
    let mut cli_args = Vec::with_capacity(args.len());
    cli_args.push("dobby-task".to_string());
    cli_args.extend(args.iter().skip(2).cloned());
    cli_args
}

fn add_task(storage: &Storage, title: String, notes: Option<String>) -> Result<()> {
    let mut state = storage.load()?;
    let plan = active_plan_mut(&mut state)?;

    let timestamp = Utc::now().to_rfc3339();
    let task = TaskRecord {
        id: Uuid::new_v4().to_string(),
        title,
        status: TaskStatus::Pending,
        notes,
        created_at: timestamp.clone(),
        updated_at: timestamp.clone(),
    };

    plan.tasks.push(task.clone());
    plan.updated_at = timestamp;
    storage.save(&state)?;

    println!(
        "{} Added task '{}' [{}]",
        PREFIX,
        task.title.as_str().cyan().bold(),
        short_id(&task.id)
    );
    Ok(())
}

fn list_tasks(storage: &Storage, status: Option<TaskStatus>) -> Result<()> {
    let state = storage.load()?;
    let Some(plan) = state.plan else {
        println!("{} No plan found. Run `dobby plan init` first.", PREFIX);
        return Ok(());
    };

    let tasks: Vec<_> = plan
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| status.map(|s| task.status == s).unwrap_or(true))
        .collect();

    if tasks.is_empty() {
        println!("{} No tasks match the requested filters.", PREFIX);
        return Ok(());
    }

    println!("{} Tasks:", PREFIX);
    for (index, task) in tasks {
        let status_str = color_status(task.status);
        println!(
            "  {:>2}. [{}] {} ({})",
            index + 1,
            status_str,
            task.title,
            short_id(&task.id)
        );
        if let Some(notes) = &task.notes {
            println!("       notes: {}", notes);
        }
    }

    Ok(())
}

fn update_status(storage: &Storage, target: String, status: TaskStatus) -> Result<()> {
    let mut state = storage.load()?;
    let plan = active_plan_mut(&mut state)?;
    let timestamp = Utc::now().to_rfc3339();

    let (title, final_status) = {
        let task = resolve_task_mut(plan, &target)?;
        if task.status == status {
            println!("{} Task already marked as {}.", PREFIX, status.as_str());
            return Ok(());
        }

        task.status = status;
        task.updated_at = timestamp.clone();
        (task.title.clone(), task.status)
    };

    plan.updated_at = timestamp;
    storage.save(&state)?;
    println!(
        "{} Updated task '{}' -> {}",
        PREFIX,
        title.as_str().green().bold(),
        color_status(final_status)
    );
    Ok(())
}

fn active_plan_mut<'a>(state: &'a mut DobbyState) -> Result<&'a mut PlanRecord> {
    state
        .plan
        .as_mut()
        .ok_or_else(|| anyhow!("No plan found. Run `dobby plan init` first."))
}

fn resolve_task_mut<'a>(plan: &'a mut PlanRecord, target: &str) -> Result<&'a mut TaskRecord> {
    if let Ok(index) = target.parse::<usize>() {
        if index == 0 {
            bail!("Indexes start at 1. Received 0.");
        }
        return plan
            .tasks
            .get_mut(index - 1)
            .ok_or_else(|| anyhow!("No task found at position {}", index));
    }

    let mut matches: Vec<_> = plan
        .tasks
        .iter_mut()
        .filter(|task| task.id.starts_with(target))
        .collect();

    match matches.len() {
        0 => bail!("No task matches identifier '{}'.", target),
        1 => Ok(matches.remove(0)),
        _ => bail!(
            "Identifier '{}' is ambiguous. Provide more characters.",
            target
        ),
    }
}
