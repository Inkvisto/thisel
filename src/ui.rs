use crate::prelude::App;
use ansi_to_tui::IntoText;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, Wrap},
};
use tui_textarea::TextArea;

/// # Panics
///
/// Will panic app.messages is not properly Vec<String>
pub fn ui(f: &mut Frame, textarea: &TextArea, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(f.size());

    f.render_widget(textarea.widget(), chunks[0]);

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let suggestions: Text = app.suggestions.join(" ").into_bytes().into_text().unwrap();
    let paragraph: Paragraph = Paragraph::new(suggestions).wrap(Wrap { trim: true });

    f.render_widget(paragraph, chunks[1]);

    let txt: Text = app.messages.join("").into_bytes().into_text().unwrap();

    app.vertical_scroll_state = app.vertical_scroll_state.content_length(app.messages.len());

    let paragraph: Paragraph = Paragraph::new(txt)
        .block(create_block("Output"))
        .style(Style::default().fg(app.fg_color))
        .wrap(Wrap { trim: true })
        .scroll((u16::try_from(app.vertical_scroll).unwrap(), 0));

    f.render_widget(paragraph, chunks[2]);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        chunks[2],
        &mut app.vertical_scroll_state,
    );
}
