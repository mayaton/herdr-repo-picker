use herdr_repo_picker::config::Config;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn defaults_are_cl_equivalent() {
    let c = Config::defaults();
    assert_eq!(c.list_command, vec!["ghq".to_string(), "list".to_string()]);
    assert_eq!(c.root_command, vec!["ghq".to_string(), "root".to_string()]);
    assert_eq!(c.launch_command, vec!["claude".to_string()]);
    assert!(c.direnv_guard);
    assert_eq!(c.preview_lines, 10);
}

#[test]
fn partial_toml_falls_back_to_defaults_per_key() {
    let s = r#"launch_command = ["codex"]"#;
    let c = Config::from_toml_str(s);
    assert_eq!(c.launch_command, vec!["codex".to_string()]);
    assert_eq!(c.list_command, vec!["ghq".to_string(), "list".to_string()]);
    assert!(c.direnv_guard);
}

#[test]
fn malformed_toml_returns_full_defaults() {
    let s = "this is [not toml";
    let c = Config::from_toml_str(s);
    assert_eq!(c, Config::defaults());
}

#[test]
fn wrong_typed_key_returns_full_defaults() {
    let s = r#"launch_command = "claude""#;
    let c = Config::from_toml_str(s);
    assert_eq!(c, Config::defaults());
}

#[test]
fn missing_file_returns_defaults() {
    let path = std::path::Path::new("/nonexistent/no-such-file.toml");
    let c = Config::load(path);
    assert_eq!(c, Config::defaults());
}

#[test]
fn load_reads_file_when_present() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "preview_lines = 5").unwrap();
    let c = Config::load(f.path());
    assert_eq!(c.preview_lines, 5);
}
