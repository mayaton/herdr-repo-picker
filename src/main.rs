use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
use std::io;
use std::path::PathBuf;

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
    loop {
        terminal.draw(|f| draw(f, app))?;
        let ev = match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => translate(k),
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
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
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
