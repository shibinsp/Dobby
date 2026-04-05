use anyhow::{bail, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use uuid::Uuid;

use crate::display::{color_status, short_id, PREFIX};
use crate::state::PlanRecord;
use crate::storage::Storage;

#[derive(Parser)]
#[command(
    name = "dobby plan",
    about = "Plan workflow commands",
    arg_required_else_help = true
)]
pub struct PlanCli {
    #[command(subcommand)]
    command: PlanCommand,
}

#[derive(Subcommand)]
enum PlanCommand {
    /// Initialize a new implementation plan.
    Init {
        /// Name of the plan.
        #[arg(short = 'n', long = "name")]
        name: String,

        /// Optional description for additional context.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,

        /// Milestones to seed the plan with (pass multiple times).
        #[arg(short = 'm', long = "milestone")]
        milestones: Vec<String>,
    },

    /// Show the current plan summary.
    Show,

    /// Reset the stored plan and tasks.
    Reset,
}

pub fn run(args: &[String]) -> Result<()> {
    let cli_args = plan_cli_args(args);
    let cli = PlanCli::parse_from(cli_args);
    let storage = Storage::new()?;

    match cli.command {
        PlanCommand::Init {
            name,
            description,
            milestones,
        } => init_plan(&storage, name, description, milestones),
        PlanCommand::Show => show_plan(&storage),
        PlanCommand::Reset => reset_plan(&storage),
    }
}

fn plan_cli_args(args: &[String]) -> Vec<String> {
    let mut cli_args = Vec::with_capacity(args.len());
    cli_args.push("dobby-plan".to_string());
    cli_args.extend(args.iter().skip(2).cloned());
    cli_args
}

fn init_plan(
    storage: &Storage,
    name: String,
    description: Option<String>,
    milestones: Vec<String>,
) -> Result<()> {
    let state = storage.load()?;
    if state.plan.is_some() {
        bail!("A plan already exists. Use `dobby plan show` or `dobby plan reset` first.");
    }

    let timestamp = Utc::now().to_rfc3339();
    let plan = PlanRecord {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        milestones,
        tasks: Vec::new(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    };

    storage.write_plan(plan.clone())?;
    println!("{} Created plan '{}'", PREFIX, plan.name.green().bold());
    Ok(())
}

fn show_plan(storage: &Storage) -> Result<()> {
    let state = storage.load()?;
    let Some(plan) = state.plan else {
        println!(
            "{} No plan found. Run `dobby plan init` to get started.",
            PREFIX
        );
        return Ok(());
    };

    println!("{} Plan: {}", PREFIX, plan.name.bold());
    if let Some(description) = &plan.description {
        println!("  Description: {}", description);
    }

    if !plan.milestones.is_empty() {
        println!("  Milestones:");
        for (index, milestone) in plan.milestones.iter().enumerate() {
            println!("    {}. {}", index + 1, milestone);
        }
    }

    if plan.tasks.is_empty() {
        println!("  Tasks: none yet. Use `dobby task add` to create work items.");
    } else {
        println!("  Tasks:");
        for (index, task) in plan.tasks.iter().enumerate() {
            let status = color_status(task.status);
            let short = short_id(&task.id);
            println!(
                "    {:>2}. [{}] {} ({})",
                index + 1,
                status,
                task.title,
                short
            );
            if let Some(notes) = &task.notes {
                println!("         notes: {}", notes);
            }
        }
    }

    println!("  Created at: {}", plan.created_at);
    println!("  Updated at: {}", plan.updated_at);
    Ok(())
}

fn reset_plan(storage: &Storage) -> Result<()> {
    if storage.reset()? {
        println!("{} Cleared the current plan.", PREFIX);
    } else {
        println!("{} Nothing to reset — no plan stored.", PREFIX);
    }
    Ok(())
}
