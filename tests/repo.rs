use herdr_repo_picker::repo::Repo;
use std::path::PathBuf;

#[test]
fn from_relative_builds_absolute_path_and_tab_name() {
    let root = PathBuf::from("/tmp/ghq");
    let r = Repo::from_relative(&root, "github.com/mayaton/foo");
    assert_eq!(r.name, "github.com/mayaton/foo");
    assert_eq!(r.tab_name, "foo");
    assert_eq!(r.path, PathBuf::from("/tmp/ghq/github.com/mayaton/foo"));
}

#[test]
fn from_relative_handles_single_component() {
    let root = PathBuf::from("/tmp/ghq");
    let r = Repo::from_relative(&root, "myrepo");
    assert_eq!(r.tab_name, "myrepo");
}
