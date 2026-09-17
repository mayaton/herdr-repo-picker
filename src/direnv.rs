use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirenvStatus {
    NoRc,
    Allowed,
    NotAllowed,
    Denied,
}

pub fn parse_status(output: &str) -> DirenvStatus {
    if !output.lines().any(|l| l.starts_with("Found RC path ")) {
        return DirenvStatus::NoRc;
    }
    let allowed = output
        .lines()
        .find_map(|l| l.strip_prefix("Found RC allowed "))
        .and_then(|s| s.trim().parse::<u8>().ok());
    match allowed {
        Some(0) => DirenvStatus::Allowed,
        Some(2) => DirenvStatus::Denied,
        _ => DirenvStatus::NotAllowed,
    }
}

pub fn check(cwd: &Path) -> DirenvStatus {
    if !cwd.join(".envrc").exists() {
        return DirenvStatus::NoRc;
    }
    let Ok(out) = Command::new("direnv").arg("status").current_dir(cwd).output() else {
        return DirenvStatus::NoRc;
    };
    parse_status(&String::from_utf8_lossy(&out.stdout))
}
