use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Workspace {
    pub workspace_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Pane {
    pub pane_id: String,
}

pub trait HerdrClient {
    fn workspace_list(&self) -> Result<Vec<Workspace>>;
    fn workspace_focus(&self, id: &str) -> Result<()>;
    fn workspace_create(&self, cwd: &Path, label: &str) -> Result<String>;
    fn pane_list(&self, workspace_id: &str) -> Result<Vec<Pane>>;
    fn pane_run(&self, pane_id: &str, cmd: &str) -> Result<()>;
}

pub struct CliHerdrClient {
    bin: String,
}

impl CliHerdrClient {
    pub fn from_env() -> Self {
        Self {
            bin: std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".into()),
        }
    }

    fn run_json(&self, args: &[&str]) -> Result<serde_json::Value> {
        let out = Command::new(&self.bin)
            .args(args)
            .output()
            .with_context(|| format!("failed to spawn {}", self.bin))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("{} {:?} failed: {}", self.bin, args, stderr);
        }
        Ok(serde_json::from_slice(&out.stdout)?)
    }
}

impl HerdrClient for CliHerdrClient {
    fn workspace_list(&self) -> Result<Vec<Workspace>> {
        let v = self.run_json(&["workspace", "list"])?;
        let workspaces = v["result"]["workspaces"].clone();
        Ok(serde_json::from_value(workspaces).unwrap_or_default())
    }

    fn workspace_focus(&self, id: &str) -> Result<()> {
        self.run_json(&["workspace", "focus", id]).map(|_| ())
    }

    fn workspace_create(&self, cwd: &Path, label: &str) -> Result<String> {
        let cwd_s = cwd.to_string_lossy().into_owned();
        let v = self.run_json(&[
            "workspace",
            "create",
            "--cwd",
            &cwd_s,
            "--label",
            label,
            "--focus",
        ])?;
        v["result"]["workspace"]["workspace_id"]
            .as_str()
            .map(|s| s.to_string())
            .context("workspace_id not found in herdr response")
    }

    fn pane_list(&self, workspace_id: &str) -> Result<Vec<Pane>> {
        let v = self.run_json(&["pane", "list", "--workspace", workspace_id])?;
        let panes = v["result"]["panes"].clone();
        Ok(serde_json::from_value(panes).unwrap_or_default())
    }

    fn pane_run(&self, pane_id: &str, cmd: &str) -> Result<()> {
        self.run_json(&["pane", "run", pane_id, cmd]).map(|_| ())
    }
}
