use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pub name: String,
    pub tab_name: String,
    pub path: PathBuf,
}

impl Repo {
    pub fn from_relative(root: &Path, rel: &str) -> Repo {
        let tab_name = rel.rsplit('/').next().unwrap_or(rel).to_string();
        Repo {
            name: rel.to_string(),
            tab_name,
            path: root.join(rel),
        }
    }
}

pub trait Lister {
    fn list(&self) -> Result<Vec<Repo>>;
}

pub struct CommandLister {
    pub list_command: Vec<String>,
    pub root_command: Vec<String>,
}

impl Lister for CommandLister {
    fn list(&self) -> Result<Vec<Repo>> {
        let root = run_capture(&self.root_command).context("failed to resolve ghq root")?;
        let root = PathBuf::from(root.trim());
        let list = run_capture(&self.list_command).context("failed to list repositories")?;
        Ok(list
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| Repo::from_relative(&root, l))
            .collect())
    }
}

fn run_capture(cmd: &[String]) -> Result<String> {
    let (prog, args) = cmd.split_first().context("empty command")?;
    let out = Command::new(prog).args(args).output()?;
    if !out.status.success() {
        anyhow::bail!("{} exited with {}", prog, out.status);
    }
    Ok(String::from_utf8(out.stdout)?)
}
