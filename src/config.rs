use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub list_command: Vec<String>,
    pub root_command: Vec<String>,
    pub launch_command: Vec<String>,
    pub direnv_guard: bool,
    pub preview_lines: u16,
}

impl Config {
    pub fn defaults() -> Self {
        Self {
            list_command: vec!["ghq".into(), "list".into()],
            root_command: vec!["ghq".into(), "root".into()],
            launch_command: vec!["claude".into()],
            direnv_guard: true,
            preview_lines: 10,
        }
    }

    pub fn from_toml_str(s: &str) -> Self {
        match toml::from_str::<RawConfig>(s) {
            Ok(raw) => raw.into_config(),
            Err(_) => Self::defaults(),
        }
    }

    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(s) => Self::from_toml_str(&s),
            Err(_) => Self::defaults(),
        }
    }
}

#[derive(Deserialize, Default)]
struct RawConfig {
    list_command: Option<Vec<String>>,
    root_command: Option<Vec<String>>,
    launch_command: Option<Vec<String>>,
    direnv_guard: Option<bool>,
    preview_lines: Option<u16>,
}

impl RawConfig {
    fn into_config(self) -> Config {
        let d = Config::defaults();
        Config {
            list_command: self.list_command.unwrap_or(d.list_command),
            root_command: self.root_command.unwrap_or(d.root_command),
            launch_command: self.launch_command.unwrap_or(d.launch_command),
            direnv_guard: self.direnv_guard.unwrap_or(d.direnv_guard),
            preview_lines: self.preview_lines.unwrap_or(d.preview_lines),
        }
    }
}
