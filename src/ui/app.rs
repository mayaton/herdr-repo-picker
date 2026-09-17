use crate::repo::Repo;
use crate::ui::matcher::Matcher;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Picking,
    Help,
    DirenvBlocked { envrc: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Char(char),
    Backspace,
    ClearQuery,
    Up,
    Down,
    Enter,
    Cancel,
    ToggleHelp,
}

pub enum AppAction {
    None,
    Exit(Option<Repo>),
}

pub struct App {
    items: Vec<Repo>,
    matcher: Matcher,
    query: String,
    selected: usize,
    visible_indices: Vec<usize>,
    pub mode: AppMode,
}

impl App {
    pub fn new(items: Vec<Repo>) -> Self {
        let names: Vec<String> = items.iter().map(|r| r.name.clone()).collect();
        let mut matcher = Matcher::new(names);
        matcher.set_query("");
        let visible_indices = matcher.matches();
        Self {
            items,
            matcher,
            query: String::new(),
            selected: 0,
            visible_indices,
            mode: AppMode::Picking,
        }
    }

    pub fn query(&self) -> &str { &self.query }
    pub fn selected(&self) -> usize { self.selected }

    pub fn visible(&mut self) -> Vec<&Repo> {
        self.visible_indices.iter().map(|&i| &self.items[i]).collect()
    }

    pub fn handle(&mut self, ev: AppEvent) -> AppAction {
        match ev {
            AppEvent::Cancel => return AppAction::Exit(None),
            AppEvent::Enter => {
                let selected_repo = self.visible_indices.get(self.selected).map(|&i| self.items[i].clone());
                return AppAction::Exit(selected_repo);
            }
            AppEvent::ToggleHelp => {
                self.mode = if matches!(self.mode, AppMode::Help) { AppMode::Picking } else { AppMode::Help };
            }
            AppEvent::Char(c) => {
                self.query.push(c);
                self.refilter();
            }
            AppEvent::Backspace => {
                self.query.pop();
                self.refilter();
            }
            AppEvent::ClearQuery => {
                self.query.clear();
                self.refilter();
            }
            AppEvent::Down => {
                let last = self.visible_indices.len().saturating_sub(1);
                if self.selected < last { self.selected += 1; }
            }
            AppEvent::Up => {
                if self.selected > 0 { self.selected -= 1; }
            }
        }
        AppAction::None
    }

    fn refilter(&mut self) {
        self.matcher.set_query(&self.query);
        self.visible_indices = self.matcher.matches();
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo(name: &str) -> Repo {
        Repo {
            name: name.into(),
            tab_name: name.rsplit('/').next().unwrap().into(),
            path: PathBuf::from(format!("/tmp/{}", name)),
        }
    }

    #[test]
    fn char_event_appends_to_query() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        app.handle(AppEvent::Char('f'));
        app.handle(AppEvent::Char('o'));
        assert_eq!(app.query(), "fo");
    }

    #[test]
    fn backspace_removes_last_query_char() {
        let mut app = App::new(vec![repo("a/foo")]);
        app.handle(AppEvent::Char('a'));
        app.handle(AppEvent::Char('b'));
        app.handle(AppEvent::Backspace);
        assert_eq!(app.query(), "a");
    }

    #[test]
    fn clear_query_resets_query() {
        let mut app = App::new(vec![repo("a/foo")]);
        app.handle(AppEvent::Char('x'));
        app.handle(AppEvent::ClearQuery);
        assert_eq!(app.query(), "");
    }

    #[test]
    fn cancel_returns_exit_none() {
        let mut app = App::new(vec![repo("a/foo")]);
        assert!(matches!(app.handle(AppEvent::Cancel), AppAction::Exit(None)));
    }

    #[test]
    fn enter_returns_exit_with_selected_repo() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        app.handle(AppEvent::Down);
        match app.handle(AppEvent::Enter) {
            AppAction::Exit(Some(r)) => assert_eq!(r.tab_name, "bar"),
            _ => panic!("expected Exit(Some(bar))"),
        }
    }

    #[test]
    fn up_and_down_move_selection_within_bounds() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        assert_eq!(app.selected(), 0);
        app.handle(AppEvent::Down);
        assert_eq!(app.selected(), 1);
        app.handle(AppEvent::Down);
        assert_eq!(app.selected(), 1);
        app.handle(AppEvent::Up);
        assert_eq!(app.selected(), 0);
        app.handle(AppEvent::Up);
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn changing_query_resets_selection_and_filters_visible() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        app.handle(AppEvent::Down);
        app.handle(AppEvent::Char('f'));
        let vis: Vec<_> = app.visible().into_iter().map(|r| r.tab_name.clone()).collect();
        assert_eq!(vis, vec!["foo".to_string()]);
        assert_eq!(app.selected(), 0);
    }
}
