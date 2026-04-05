use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;

use crate::display::{short_id, PREFIX};
use crate::state::{RuleRecord, COAUTHOR_RULE_TEXT};
use crate::storage::Storage;

#[derive(Parser)]
#[command(
    name = "dobby rule",
    about = "Manage global commit rules",
    arg_required_else_help = true
)]
pub struct RuleCli {
    #[command(subcommand)]
    command: RuleCommand,
}

#[derive(Subcommand)]
enum RuleCommand {
    /// List all registered commit rules.
    List,

    /// Add a new rule that contributors must follow.
    Add {
        /// Rule text to append.
        text: String,
    },

    /// Remove a rule by 1-based index or ID/prefix.
    Remove {
        /// Index, ID, or unique prefix identifying the rule.
        target: String,
    },
}

pub fn run(args: &[String]) -> Result<()> {
    let cli_args = rule_cli_args(args);
    let cli = RuleCli::parse_from(cli_args);
    let storage = Storage::new()?;

    match cli.command {
        RuleCommand::List => list_rules(&storage),
        RuleCommand::Add { text } => add_rule(&storage, text),
        RuleCommand::Remove { target } => remove_rule(&storage, target),
    }
}

fn rule_cli_args(args: &[String]) -> Vec<String> {
    let mut cli_args = Vec::with_capacity(args.len());
    cli_args.push("dobby-rule".to_string());
    cli_args.extend(args.iter().skip(2).cloned());
    cli_args
}

fn list_rules(storage: &Storage) -> Result<()> {
    let state = storage.load()?;
    if state.rules.is_empty() {
        println!("{} No rules defined yet.", PREFIX);
        return Ok(());
    }

    println!("{} Commit Rules:", PREFIX);
    for (index, rule) in state.rules.iter().enumerate() {
        let label = if rule.text == COAUTHOR_RULE_TEXT {
            format!("{} (enforced)", rule.text)
        } else {
            rule.text.clone()
        };
        println!(
            "  {:>2}. {} [{}]",
            index + 1,
            label,
            short_id(&rule.id)
        );
        println!("       added at {}", rule.created_at);
    }

    Ok(())
}

fn add_rule(storage: &Storage, text: String) -> Result<()> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!("Rule text cannot be empty.");
    }

    let mut state = storage.load()?;
    if state.rules.iter().any(|rule| rule.text == trimmed) {
        bail!("Rule already exists: {}", trimmed);
    }

    let record = RuleRecord::new(trimmed.to_string());
    state.rules.push(record.clone());
    storage.save(&state)?;

    println!(
        "{} Added rule '{}' [{}]",
        PREFIX,
        record.text.cyan().bold(),
        short_id(&record.id)
    );
    Ok(())
}

fn remove_rule(storage: &Storage, target: String) -> Result<()> {
    let mut state = storage.load()?;
    if state.rules.is_empty() {
        bail!("No rules stored.");
    }

    let index = resolve_rule_index(&state.rules, &target)?;
    if state.rules[index].text == COAUTHOR_RULE_TEXT {
        bail!("The co-author rule is enforced and cannot be removed.");
    }

    let removed = state.rules.remove(index);
    storage.save(&state)?;
    println!(
        "{} Removed rule '{}' [{}]",
        PREFIX,
        removed.text.yellow(),
        short_id(&removed.id)
    );
    Ok(())
}

fn resolve_rule_index(rules: &[RuleRecord], target: &str) -> Result<usize> {
    if let Ok(index) = target.parse::<usize>() {
        if index == 0 {
            bail!("Indexes start at 1.");
        }
        return rules
            .get(index - 1)
            .map(|_| index - 1)
            .ok_or_else(|| anyhow!("No rule at position {}", index));
    }

    let mut matches: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.id.starts_with(target))
        .map(|(index, _)| index)
        .collect();

    match matches.len() {
        0 => bail!("No rule matches identifier '{}'.", target),
        1 => Ok(matches.remove(0)),
        _ => bail!("Identifier '{}' is ambiguous. Provide more characters.", target),
    }
}
