use crate::config::Config;
use crate::direnv::DirenvStatus;
use crate::herdr::HerdrClient;
use crate::repo::Repo;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    FocusedExisting(String),
    CreatedAndLaunched(String),
    BlockedByDirenv(PathBuf),
}

pub fn dispatch<C: HerdrClient>(
    repo: &Repo,
    config: &Config,
    client: &C,
    direnv_status: DirenvStatus,
) -> Result<DispatchOutcome> {
    if let Some(ws) = client.workspace_list()?.into_iter().find(|w| w.label == repo.tab_name) {
        client.workspace_focus(&ws.workspace_id)?;
        return Ok(DispatchOutcome::FocusedExisting(ws.workspace_id));
    }

    if config.direnv_guard && matches!(direnv_status, DirenvStatus::NotAllowed | DirenvStatus::Denied) {
        return Ok(DispatchOutcome::BlockedByDirenv(repo.path.join(".envrc")));
    }

    let ws_id = client.workspace_create(&repo.path, &repo.tab_name)?;
    if !config.launch_command.is_empty() {
        let pane = client.pane_list(&ws_id)?.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("no initial pane found for workspace {}", ws_id))?;
        let cmd = shell_words::join(&config.launch_command);
        client.pane_run(&pane.pane_id, &cmd)?;
    }
    Ok(DispatchOutcome::CreatedAndLaunched(ws_id))
}
