use chrono::Utc;
use chrono::Utc;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const COAUTHOR_RULE_TEXT: &str =
    "Include 'Co-authored-by: shibinsp <apple@MacBookPro.alphionsee.in>' in every commit message.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DobbyState {
    pub plan: Option<PlanRecord>,
    #[serde(default)]
    pub rules: Vec<RuleRecord>,
}

impl Default for DobbyState {
    fn default() -> Self {
        Self {
            plan: None,
            rules: vec![RuleRecord::commit_coauthor_rule()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub milestones: Vec<String>,
    #[serde(default)]
    pub tasks: Vec<TaskRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleRecord {
    pub id: String,
    pub text: String,
    pub created_at: String,
}

impl RuleRecord {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            text: text.into(),
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn commit_coauthor_rule() -> Self {
        Self::new(COAUTHOR_RULE_TEXT)
    }

    pub fn is_coauthor_rule(&self) -> bool {
        self.text == COAUTHOR_RULE_TEXT
    }
}

pub fn ensure_default_rule(rules: &mut Vec<RuleRecord>) -> bool {
    if rules.iter().any(|rule| rule.is_coauthor_rule()) {
        false
    } else {
        rules.push(RuleRecord::commit_coauthor_rule());
        true
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

