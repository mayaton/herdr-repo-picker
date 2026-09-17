use herdr_repo_picker::ui::matcher::Matcher;

#[test]
fn empty_query_returns_all_items_in_original_order() {
    let items = vec!["alpha".into(), "beta".into(), "gamma".into()];
    let mut m = Matcher::new(items);
    m.set_query("");
    assert_eq!(m.matches(), vec![0, 1, 2]);
}

#[test]
fn substring_query_matches_only_containing_items() {
    let items = vec![
        "github.com/foo/bar".into(),
        "github.com/baz/qux".into(),
        "gitlab.com/foo/bar".into(),
    ];
    let mut m = Matcher::new(items);
    m.set_query("gitlab");
    let ids = m.matches();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], 2);
}

#[test]
fn fuzzy_query_ranks_closer_matches_first() {
    let items = vec![
        "one/foo-bar".into(),
        "two/foobar".into(),
        "three/unrelated".into(),
    ];
    let mut m = Matcher::new(items);
    m.set_query("foobar");
    let ids = m.matches();
    assert!(ids.contains(&1));
    assert!(!ids.contains(&2));
    assert_eq!(ids[0], 1);
}
