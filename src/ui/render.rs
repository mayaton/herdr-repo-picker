use crate::ui::app::{App, AppMode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();
    match app.mode.clone() {
        AppMode::DirenvBlocked { envrc } => {
            let text = format!(
                ".envrc is not allowed by direnv.\n\
                 Review and allow it, then retry:\n\
                   cat {p}\n\
                   direnv allow {p}\n\n\
                 [Enter to close]",
                p = envrc.display()
            );
            let block = Block::default()
                .borders(Borders::ALL)
                .title("direnv blocked");
            frame.render_widget(Paragraph::new(text).block(block), area);
            return;
        }
        AppMode::Help => {
            let text = "Keys:\n\
                        Enter        pick\n\
                        Esc/Ctrl-c   cancel\n\
                        Ctrl-n/Down  next\n\
                        Ctrl-p/Up    prev\n\
                        Backspace    delete char\n\
                        Ctrl-u       clear query\n\
                        ?            toggle this help";
            let block = Block::default().borders(Borders::ALL).title("Help");
            frame.render_widget(Paragraph::new(text).block(block), area);
            return;
        }
        AppMode::Picking => {}
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let query_line = Line::from(vec![
        Span::styled("> ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(app.query().to_string()),
    ]);
    frame.render_widget(
        Paragraph::new(query_line)
            .block(Block::default().borders(Borders::ALL).title("Repo picker")),
        chunks[0],
    );

    let list_block = Block::default().borders(Borders::ALL);
    let inner_area = list_block.inner(chunks[1]);
    app.set_list_area(inner_area);
    app.set_list_height(inner_area.height as usize);

    let selected = app.selected();
    let scroll_offset = app.scroll_offset();
    let items: Vec<ListItem> = app
        .visible_page()
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut style = Style::default();
            if scroll_offset + i == selected {
                style = style.add_modifier(Modifier::REVERSED);
            }
            ListItem::new(r.name.clone()).style(style)
        })
        .collect();
    frame.render_widget(List::new(items).block(list_block), chunks[1]);

    let footer = Paragraph::new("Enter pick  Esc cancel  ? help");
    frame.render_widget(footer, chunks[2]);
}
