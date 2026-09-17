use herdr_repo_picker::config::Config;
use herdr_repo_picker::direnv::DirenvStatus;
use herdr_repo_picker::dispatch::{dispatch, DispatchOutcome};
use herdr_repo_picker::herdr::{HerdrClient, Pane, Workspace};
use herdr_repo_picker::repo::Repo;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct MockClient {
    workspaces: RefCell<Vec<Workspace>>,
    panes_by_ws: RefCell<std::collections::HashMap<String, Vec<Pane>>>,
    focused: RefCell<Vec<String>>,
    created: RefCell<Vec<(PathBuf, String)>>,
    ran: RefCell<Vec<(String, String)>>,
}

impl HerdrClient for MockClient {
    fn workspace_list(&self) -> anyhow::Result<Vec<Workspace>> {
        Ok(self.workspaces.borrow().clone())
    }
    fn workspace_focus(&self, id: &str) -> anyhow::Result<()> {
        self.focused.borrow_mut().push(id.into());
        Ok(())
    }
    fn workspace_create(&self, cwd: &Path, label: &str) -> anyhow::Result<String> {
        let id = format!("ws-{}", label);
        self.created.borrow_mut().push((cwd.to_path_buf(), label.into()));
        self.workspaces.borrow_mut().push(Workspace { workspace_id: id.clone(), label: label.into() });
        self.panes_by_ws.borrow_mut().insert(id.clone(), vec![Pane { pane_id: format!("pane-{}", label) }]);
        Ok(id)
    }
    fn pane_list(&self, workspace_id: &str) -> anyhow::Result<Vec<Pane>> {
        Ok(self.panes_by_ws.borrow().get(workspace_id).cloned().unwrap_or_default())
    }
    fn pane_run(&self, pane_id: &str, cmd: &str) -> anyhow::Result<()> {
        self.ran.borrow_mut().push((pane_id.into(), cmd.into()));
        Ok(())
    }
}

fn sample_repo() -> Repo {
    Repo {
        name: "github.com/mayaton/foo".into(),
        tab_name: "foo".into(),
        path: PathBuf::from("/tmp/ghq/github.com/mayaton/foo"),
    }
}

#[test]
fn focuses_existing_workspace() {
    let client = MockClient::default();
    client.workspaces.borrow_mut().push(Workspace { workspace_id: "ws-existing".into(), label: "foo".into() });
    let outcome = dispatch(&sample_repo(), &Config::defaults(), &client, DirenvStatus::NoRc).unwrap();
    assert_eq!(outcome, DispatchOutcome::FocusedExisting("ws-existing".into()));
    assert_eq!(*client.focused.borrow(), vec!["ws-existing".to_string()]);
    assert!(client.created.borrow().is_empty());
    assert!(client.ran.borrow().is_empty());
}

#[test]
fn creates_workspace_and_runs_launch_command() {
    let client = MockClient::default();
    let outcome = dispatch(&sample_repo(), &Config::defaults(), &client, DirenvStatus::NoRc).unwrap();
    assert_eq!(outcome, DispatchOutcome::CreatedAndLaunched("ws-foo".into()));
    assert_eq!(*client.created.borrow(), vec![(PathBuf::from("/tmp/ghq/github.com/mayaton/foo"), "foo".to_string())]);
    assert_eq!(*client.ran.borrow(), vec![("pane-foo".to_string(), "claude".to_string())]);
}

#[test]
fn blocks_when_direnv_not_allowed() {
    let client = MockClient::default();
    let outcome = dispatch(&sample_repo(), &Config::defaults(), &client, DirenvStatus::NotAllowed).unwrap();
    assert_eq!(outcome, DispatchOutcome::BlockedByDirenv(PathBuf::from("/tmp/ghq/github.com/mayaton/foo/.envrc")));
    assert!(client.created.borrow().is_empty());
    assert!(client.ran.borrow().is_empty());
}

#[test]
fn direnv_guard_disabled_bypasses_block() {
    let client = MockClient::default();
    let mut c = Config::defaults();
    c.direnv_guard = false;
    let outcome = dispatch(&sample_repo(), &c, &client, DirenvStatus::NotAllowed).unwrap();
    assert_eq!(outcome, DispatchOutcome::CreatedAndLaunched("ws-foo".into()));
}

#[test]
fn empty_launch_command_does_not_run_anything() {
    let client = MockClient::default();
    let mut c = Config::defaults();
    c.launch_command = vec![];
    let outcome = dispatch(&sample_repo(), &c, &client, DirenvStatus::NoRc).unwrap();
    assert_eq!(outcome, DispatchOutcome::CreatedAndLaunched("ws-foo".into()));
    assert!(client.ran.borrow().is_empty());
}

#[test]
fn quotes_launch_command_arguments() {
    let client = MockClient::default();
    let mut c = Config::defaults();
    c.launch_command = vec!["claude".into(), "--model".into(), "a b".into()];
    let _ = dispatch(&sample_repo(), &c, &client, DirenvStatus::NoRc).unwrap();
    assert_eq!(*client.ran.borrow(), vec![("pane-foo".to_string(), "claude --model 'a b'".to_string())]);
}
