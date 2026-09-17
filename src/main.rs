use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use herdr_repo_picker::config::Config;
use herdr_repo_picker::direnv;
use herdr_repo_picker::dispatch::{DispatchOutcome, dispatch};
use herdr_repo_picker::herdr::CliHerdrClient;
use herdr_repo_picker::repo::{CommandLister, Lister};
use herdr_repo_picker::ui::app::{App, AppAction, AppEvent, AppMode};
use herdr_repo_picker::ui::render::draw;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(500);

/// Maps a mouse click's screen coordinates to a row index within
/// `app.visible_page()`, accounting for the current scroll offset.
/// Returns `None` when the click falls outside `area`.
fn mouse_row_to_index(area: Rect, scroll_offset: usize, col: u16, row: u16) -> Option<usize> {
    if col < area.x || col >= area.x + area.width {
        return None;
    }
    if row < area.y || row >= area.y + area.height {
        return None;
    }
    Some(scroll_offset + (row - area.y) as usize)
}

struct MouseClickTracker {
    last: Option<(usize, Instant)>,
}

impl MouseClickTracker {
    fn new() -> Self {
        Self { last: None }
    }

    /// Registers a left-click on `row` at `now`, returning `DoubleClick` when
    /// it lands on the same row within `DOUBLE_CLICK_THRESHOLD` of the
    /// previous click, otherwise `Click`.
    fn register(&mut self, row: usize, now: Instant) -> AppEvent {
        let is_double = self
            .last
            .is_some_and(|(r, t)| r == row && now.duration_since(t) <= DOUBLE_CLICK_THRESHOLD);
        self.last = if is_double { None } else { Some((row, now)) };
        if is_double {
            AppEvent::DoubleClick(row)
        } else {
            AppEvent::Click(row)
        }
    }
}

fn translate_mouse(app: &App, tracker: &mut MouseClickTracker, m: MouseEvent) -> Option<AppEvent> {
    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let area = app.list_area()?;
            let row = mouse_row_to_index(area, app.scroll_offset(), m.column, m.row)?;
            Some(tracker.register(row, Instant::now()))
        }
        MouseEventKind::ScrollUp => Some(AppEvent::WheelUp),
        MouseEventKind::ScrollDown => Some(AppEvent::WheelDown),
        _ => None,
    }
}

fn config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("HERDR_PLUGIN_CONFIG_DIR") {
        return Some(PathBuf::from(dir).join("config.toml"));
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/herdr-repo-picker/config.toml"))
}

fn main() -> Result<()> {
    let config = config_path()
        .map(|p| Config::load(&p))
        .unwrap_or_else(Config::defaults);

    let lister = CommandLister {
        list_command: config.list_command.clone(),
        root_command: config.root_command.clone(),
    };
    let repos = lister.list()?;
    if repos.is_empty() {
        eprintln!("herdr-repo-picker: no repositories found");
        return Ok(());
    }

    let mut app = App::new(repos);
    let picked = run_picker(&mut app)?;

    let Some(repo) = picked else {
        return Ok(());
    };

    let direnv_status = if config.direnv_guard {
        direnv::check(&repo.path)
    } else {
        direnv::DirenvStatus::NoRc
    };
    let client = CliHerdrClient::from_env();
    let outcome = dispatch(&repo, &config, &client, direnv_status)?;

    if let DispatchOutcome::BlockedByDirenv(envrc) = &outcome {
        app.mode = AppMode::DirenvBlocked {
            envrc: envrc.clone(),
        };
        run_blocking_screen(&mut app)?;
    }
    Ok(())
}

fn run_picker(app: &mut App) -> Result<Option<herdr_repo_picker::repo::Repo>> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut click_tracker = MouseClickTracker::new();
    loop {
        terminal.draw(|f| draw(f, app))?;
        let ev = match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => translate(k),
            Event::Mouse(m) => translate_mouse(app, &mut click_tracker, m),
            _ => None,
        };
        if let Some(ev) = ev
            && let AppAction::Exit(picked) = app.handle(ev)
        {
            return Ok(picked);
        }
    }
}

fn run_blocking_screen(app: &mut App) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    loop {
        terminal.draw(|f| draw(f, app))?;
        if let Event::Key(k) = event::read()? {
            let esc_or_enter = matches!(k.code, KeyCode::Enter | KeyCode::Esc);
            let ctrl_c =
                matches!(k.code, KeyCode::Char('c')) && k.modifiers.contains(KeyModifiers::CONTROL);
            if esc_or_enter || ctrl_c {
                return Ok(());
            }
        }
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

fn translate(k: KeyEvent) -> Option<AppEvent> {
    match (k.code, k.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL)
        | (KeyCode::Char('g'), KeyModifiers::CONTROL)
        | (KeyCode::Esc, _) => Some(AppEvent::Cancel),
        (KeyCode::Enter, _) => Some(AppEvent::Enter),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(AppEvent::ClearQuery),
        (KeyCode::Char('n'), KeyModifiers::CONTROL) | (KeyCode::Down, _) => Some(AppEvent::Down),
        (KeyCode::Char('p'), KeyModifiers::CONTROL) | (KeyCode::Up, _) => Some(AppEvent::Up),
        (KeyCode::Char('?'), _) => Some(AppEvent::ToggleHelp),
        (KeyCode::Backspace, _) => Some(AppEvent::Backspace),
        (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => Some(AppEvent::Char(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use herdr_repo_picker::repo::Repo;
    use ratatui::layout::Rect;
    use std::time::{Duration, Instant};

    fn repo(name: &str) -> Repo {
        Repo::from_relative(&PathBuf::from("/tmp"), name)
    }

    #[test]
    fn mouse_row_to_index_maps_row_within_area() {
        let area = Rect::new(1, 1, 20, 5);
        assert_eq!(mouse_row_to_index(area, 0, 5, 3), Some(2));
    }

    #[test]
    fn mouse_row_to_index_returns_none_outside_area() {
        let area = Rect::new(1, 1, 20, 5);
        assert_eq!(mouse_row_to_index(area, 0, 5, 10), None);
        assert_eq!(mouse_row_to_index(area, 0, 0, 3), None);
    }

    #[test]
    fn mouse_row_to_index_accounts_for_scroll_offset() {
        let area = Rect::new(1, 1, 20, 5);
        assert_eq!(mouse_row_to_index(area, 3, 5, 1), Some(3));
    }

    #[test]
    fn tracker_emits_click_on_first_registration() {
        let mut tracker = MouseClickTracker::new();
        assert_eq!(tracker.register(2, Instant::now()), AppEvent::Click(2));
    }

    #[test]
    fn tracker_emits_double_click_within_threshold_on_same_row() {
        let mut tracker = MouseClickTracker::new();
        let t0 = Instant::now();
        tracker.register(2, t0);
        let t1 = t0 + Duration::from_millis(200);
        assert_eq!(tracker.register(2, t1), AppEvent::DoubleClick(2));
    }

    #[test]
    fn tracker_emits_click_when_threshold_exceeded() {
        let mut tracker = MouseClickTracker::new();
        let t0 = Instant::now();
        tracker.register(2, t0);
        let t1 = t0 + Duration::from_millis(600);
        assert_eq!(tracker.register(2, t1), AppEvent::Click(2));
    }

    #[test]
    fn tracker_emits_click_when_row_differs() {
        let mut tracker = MouseClickTracker::new();
        let t0 = Instant::now();
        tracker.register(1, t0);
        let t1 = t0 + Duration::from_millis(100);
        assert_eq!(tracker.register(2, t1), AppEvent::Click(2));
    }

    #[test]
    fn translate_mouse_left_click_maps_to_app_click() {
        let mut app = App::new(vec![repo("a/foo"), repo("a/bar")]);
        app.set_list_area(Rect::new(0, 0, 10, 5));
        let mut tracker = MouseClickTracker::new();
        let ev = translate_mouse(
            &app,
            &mut tracker,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 2,
                row: 1,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(ev, Some(AppEvent::Click(1)));
    }

    #[test]
    fn translate_mouse_scroll_maps_to_wheel_events() {
        let app = App::new(vec![repo("a/foo")]);
        let mut tracker = MouseClickTracker::new();
        let up = translate_mouse(
            &app,
            &mut tracker,
            MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(up, Some(AppEvent::WheelUp));
        let down = translate_mouse(
            &app,
            &mut tracker,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(down, Some(AppEvent::WheelDown));
    }

    #[test]
    fn translate_mouse_click_outside_list_area_is_ignored() {
        let mut app = App::new(vec![repo("a/foo")]);
        app.set_list_area(Rect::new(0, 0, 10, 5));
        let mut tracker = MouseClickTracker::new();
        let ev = translate_mouse(
            &app,
            &mut tracker,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 0,
                row: 20,
                modifiers: KeyModifiers::NONE,
            },
        );
        assert_eq!(ev, None);
    }
}
