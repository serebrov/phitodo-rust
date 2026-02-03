use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table},
    Frame,
};

use crate::ui::theme::Theme;

pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Center the help panel
    let width = area.width.min(70);
    let height = area.height.min(30);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let help_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, help_area);

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(true))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(help_area);
    frame.render_widget(block, help_area);

    let chunks = Layout::vertical([
        Constraint::Length(1),  // Header
        Constraint::Min(10),    // Shortcuts table
        Constraint::Length(2),  // Footer
    ])
    .split(inner);

    // Header
    let header = Paragraph::new(Line::from(Span::styled(
        "Press ? to close this help",
        Theme::muted_style(),
    )));
    frame.render_widget(header, chunks[0]);

    // Shortcuts table
    let shortcuts = vec![
        ("Navigation", vec![
            ("Alt+1-9", "Switch views (Inbox, Today, etc.)"),
            ("j/k or ↓/↑", "Move selection down/up"),
            ("g/G", "Go to first/last item"),
            ("Tab", "Cycle focus (sidebar → list → detail)"),
            ("Enter", "Open selected item"),
        ]),
        ("Task Actions", vec![
            ("Space", "Toggle task completion"),
            ("n", "New task"),
            ("N", "New project"),
            ("e", "Edit selected"),
            ("d", "Delete (with confirmation)"),
            ("1-4", "Set priority (None/Low/Medium/High)"),
            ("i/a/s", "Move to Inbox/Active/Scheduled"),
        ]),
        ("Other", vec![
            ("/", "Search/filter"),
            ("r", "Refresh data"),
            ("?", "Show/hide help"),
            ("q", "Quit"),
        ]),
    ];

    let mut rows: Vec<Row> = Vec::new();
    for (section, bindings) in shortcuts {
        // Section header
        rows.push(Row::new(vec![
            "",
            "",
        ]).style(Style::default()));
        rows.push(Row::new(vec![
            section,
            "",
        ]).style(Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)));

        for (key, action) in bindings {
            rows.push(Row::new(vec![
                key,
                action,
            ]));
        }
    }

    let table = Table::new(
        rows,
        [Constraint::Length(20), Constraint::Min(30)],
    )
    .style(Style::default().fg(Theme::FG))
    .column_spacing(2);

    frame.render_widget(table, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("φ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
        Span::raw(" phitodo-tui"),
    ]))
    .style(Theme::muted_style());
    frame.render_widget(footer, chunks[2]);
}
