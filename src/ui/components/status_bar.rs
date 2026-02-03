use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::theme::Theme;

pub struct StatusBarContext {
    pub has_selection: bool,
    pub is_completed: bool,
    pub focus: &'static str, // "sidebar", "list", "detail"
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, ctx: &StatusBarContext) {
    let shortcuts = match ctx.focus {
        "sidebar" => vec![
            ("j/k", "navigate"),
            ("Enter", "select"),
            ("l/→", "go to list"),
            ("n", "new task"),
            ("?", "help"),
            ("q", "quit"),
        ],
        "list" => {
            let mut s = vec![
                ("j/k", "navigate"),
                ("h/←", "sidebar"),
                ("l/→", "details"),
            ];
            if ctx.has_selection {
                s.push(("Space", if ctx.is_completed { "uncomplete" } else { "complete" }));
                s.push(("e", "edit"));
                s.push(("d", "delete"));
                s.push(("A-1-4", "priority"));
            }
            s.push(("n", "new"));
            s.push(("?", "help"));
            s
        }
        "detail" => vec![
            ("h/←", "go to list"),
            ("e", "edit"),
            ("Space", if ctx.is_completed { "uncomplete" } else { "complete" }),
            ("?", "help"),
        ],
        _ => vec![("?", "help"), ("q", "quit")],
    };

    let spans: Vec<Span> = shortcuts
        .iter()
        .enumerate()
        .flat_map(|(i, (key, action))| {
            let mut s = vec![
                Span::styled(
                    *key,
                    Style::default()
                        .fg(Theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", action), Style::default().fg(Theme::FG_DIM)),
            ];
            if i < shortcuts.len() - 1 {
                s.push(Span::styled("  │  ", Style::default().fg(Theme::BORDER)));
            }
            s
        })
        .collect();

    let help_line = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    frame.render_widget(help_line, area);
}
