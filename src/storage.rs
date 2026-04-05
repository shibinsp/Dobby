use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::state::{ensure_default_rule, DobbyState, PlanRecord, RuleRecord};

pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let mut dir = dirs::home_dir().context("Unable to locate home directory")?;
        dir.push(".dobby-cli");
        fs::create_dir_all(&dir).context("Failed to create Dobby state directory")?;
        Ok(Self {
            path: dir.join("state.json"),
        })
    }

    pub fn load(&self) -> Result<DobbyState> {
        let mut state = if !self.path.exists() {
            DobbyState::default()
        } else {
            let bytes = fs::read(&self.path).context("Failed to read Dobby state file")?;
            serde_json::from_slice(&bytes).context("Failed to parse Dobby state file")?
        };

        let mut changed = false;
        if ensure_default_rule(&mut state.rules) {
            changed = true;
        }

        if changed {
            self.save(&state)?;
        }

        Ok(state)
    }

    pub fn save(&self, state: &DobbyState) -> Result<()> {
        let contents =
            serde_json::to_vec_pretty(state).context("Failed to serialize Dobby state")?;
        fs::write(&self.path, contents).context("Failed to write Dobby state file")?;
        Ok(())
    }

    pub fn write_plan(&self, plan: PlanRecord) -> Result<()> {
        let mut state = self.load()?;
        state.plan = Some(plan);
        self.save(&state)
    }

    pub fn reset(&self) -> Result<bool> {
        if self.path.exists() {
            fs::remove_file(&self.path).context("Failed to remove state file")?;
            return Ok(true);
        }
        Ok(false)
    }
}
