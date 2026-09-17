use herdr_repo_picker::direnv::{DirenvStatus, parse_status};

#[test]
fn no_rc_when_output_lacks_found_rc_path() {
    let out = "direnv exec path /usr/local/bin/direnv\n";
    assert_eq!(parse_status(out), DirenvStatus::NoRc);
}

#[test]
fn allowed_when_found_rc_allowed_zero() {
    let out = "Found RC path /tmp/repo/.envrc\nFound RC allowed 0\n";
    assert_eq!(parse_status(out), DirenvStatus::Allowed);
}

#[test]
fn not_allowed_when_found_rc_allowed_one() {
    let out = "Found RC path /tmp/repo/.envrc\nFound RC allowed 1\n";
    assert_eq!(parse_status(out), DirenvStatus::NotAllowed);
}

#[test]
fn denied_when_found_rc_allowed_two() {
    let out = "Found RC path /tmp/repo/.envrc\nFound RC allowed 2\n";
    assert_eq!(parse_status(out), DirenvStatus::Denied);
}
