use crate::repo::Repo;
use crate::ui::matcher::Matcher;
use ratatui::layout::Rect;
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
    Click(usize),
    DoubleClick(usize),
    WheelUp,
    WheelDown,
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
    scroll_offset: usize,
    list_height: usize,
    list_area: Option<Rect>,
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
            scroll_offset: 0,
            list_height: 0,
            list_area: None,
            mode: AppMode::Picking,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the number of rows available to display the list, so selection
    /// changes can keep the selected row inside the visible window.
    pub fn set_list_height(&mut self, height: usize) {
        self.list_height = height;
        self.ensure_visible();
    }

    pub fn list_area(&self) -> Option<Rect> {
        self.list_area
    }

    /// Records where the list is drawn on screen, so mouse coordinates can
    /// be translated into a row index.
    pub fn set_list_area(&mut self, area: Rect) {
        self.list_area = Some(area);
    }

    pub fn visible(&mut self) -> Vec<&Repo> {
        self.visible_indices
            .iter()
            .map(|&i| &self.items[i])
            .collect()
    }

    /// Returns only the slice of visible repos that fits within the current
    /// `list_height`, starting at `scroll_offset`.
    pub fn visible_page(&mut self) -> Vec<&Repo> {
        let start = self.scroll_offset;
        let end = if self.list_height == 0 {
            self.visible_indices.len()
        } else {
            (start + self.list_height).min(self.visible_indices.len())
        };
        self.visible_indices[start..end]
            .iter()
            .map(|&i| &self.items[i])
            .collect()
    }

    pub fn handle(&mut self, ev: AppEvent) -> AppAction {
        match ev {
            AppEvent::Cancel => return AppAction::Exit(None),
            AppEvent::Enter => {
                let selected_repo = self
                    .visible_indices
                    .get(self.selected)
                    .map(|&i| self.items[i].clone());
                return AppAction::Exit(selected_repo);
            }
            AppEvent::ToggleHelp => {
                self.mode = if matches!(self.mode, AppMode::Help) {
                    AppMode::Picking
                } else {
                    AppMode::Help
                };
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
            AppEvent::Down | AppEvent::WheelDown => {
                let last = self.visible_indices.len().saturating_sub(1);
                if self.selected < last {
                    self.selected += 1;
                    self.ensure_visible();
                }
            }
            AppEvent::Up | AppEvent::WheelUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.ensure_visible();
                }
            }
            AppEvent::Click(row) => {
                if row < self.visible_indices.len() {
                    self.selected = row;
                    self.ensure_visible();
                }
            }
            AppEvent::DoubleClick(row) => {
                if row < self.visible_indices.len() {
                    self.selected = row;
                }
                let selected_repo = self
                    .visible_indices
                    .get(self.selected)
                    .map(|&i| self.items[i].clone());
                return AppAction::Exit(selected_repo);
            }
        }
        AppAction::None
    }

    fn refilter(&mut self) {
        self.matcher.set_query(&self.query);
        self.visible_indices = self.matcher.matches();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    fn ensure_visible(&mut self) {
        if self.list_height == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.list_height {
            self.scroll_offset = self.selected + 1 - self.list_height;
        }
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
        assert!(matches!(
            app.handle(AppEvent::Cancel),
            AppAction::Exit(None)
        ));
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
        let vis: Vec<_> = app
            .visible()
            .into_iter()
            .map(|r| r.tab_name.clone())
            .collect();
        assert_eq!(vis, vec!["foo".to_string()]);
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn wheel_events_move_selection_like_up_down() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        app.handle(AppEvent::WheelDown);
        assert_eq!(app.selected(), 1);
        app.handle(AppEvent::WheelDown);
        assert_eq!(app.selected(), 1);
        app.handle(AppEvent::WheelUp);
        assert_eq!(app.selected(), 0);
        app.handle(AppEvent::WheelUp);
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn click_sets_selection_without_exiting() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar"), repo("a/baz")]);
        let action = app.handle(AppEvent::Click(2));
        assert!(matches!(action, AppAction::None));
        assert_eq!(app.selected(), 2);
    }

    #[test]
    fn click_out_of_range_is_ignored() {
        let mut app = App::new(vec![repo("a/foo")]);
        app.handle(AppEvent::Click(5));
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn double_click_selects_and_exits_with_repo() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        match app.handle(AppEvent::DoubleClick(1)) {
            AppAction::Exit(Some(r)) => assert_eq!(r.tab_name, "bar"),
            _ => panic!("expected Exit(Some(bar))"),
        }
    }

    #[test]
    fn selection_scrolls_list_when_moving_past_visible_window() {
        let mut app = App::new(vec![repo("a/r0"), repo("a/r1"), repo("a/r2"), repo("a/r3")]);
        app.set_list_height(2);
        assert_eq!(app.scroll_offset(), 0);
        app.handle(AppEvent::Down);
        assert_eq!(app.scroll_offset(), 0);
        app.handle(AppEvent::Down);
        assert_eq!(app.selected(), 2);
        assert_eq!(app.scroll_offset(), 1);
        app.handle(AppEvent::Up);
        assert_eq!(app.scroll_offset(), 1);
        app.handle(AppEvent::Up);
        assert_eq!(app.scroll_offset(), 0);
    }

    #[test]
    fn refilter_resets_scroll_offset() {
        let mut app = App::new(vec![repo("a/r0"), repo("a/r1"), repo("a/r2")]);
        app.set_list_height(2);
        app.handle(AppEvent::Down);
        app.handle(AppEvent::Down);
        assert_eq!(app.scroll_offset(), 1);
        app.handle(AppEvent::Char('r'));
        assert_eq!(app.scroll_offset(), 0);
    }

    #[test]
    fn visible_page_returns_window_sized_slice() {
        let mut app = App::new(vec![repo("a/r0"), repo("a/r1"), repo("a/r2"), repo("a/r3")]);
        app.set_list_height(2);
        app.handle(AppEvent::Down);
        app.handle(AppEvent::Down);
        let page: Vec<_> = app
            .visible_page()
            .into_iter()
            .map(|r| r.tab_name.clone())
            .collect();
        assert_eq!(page, vec!["r1".to_string(), "r2".to_string()]);
    }

    #[test]
    fn list_area_defaults_to_none_and_can_be_set() {
        let mut app = App::new(vec![repo("a/foo")]);
        assert_eq!(app.list_area(), None);
        let area = ratatui::layout::Rect::new(1, 2, 30, 10);
        app.set_list_area(area);
        assert_eq!(app.list_area(), Some(area));
    }
}
