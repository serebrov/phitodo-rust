use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::Task;
use crate::ui::theme::Theme;

pub fn render_task_detail(frame: &mut Frame, area: Rect, task: Option<&Task>, focused: bool) {
    let block = Block::default()
        .title(" Task Details ")
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(task) = task else {
        let empty = Paragraph::new("No task selected")
            .style(Theme::muted_style());
        frame.render_widget(empty, inner);
        return;
    };

    let chunks = Layout::vertical([
        Constraint::Length(2), // Title
        Constraint::Length(3), // Status line
        Constraint::Min(3),    // Notes
        Constraint::Length(4), // Metadata
    ])
    .split(inner);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            &task.title,
            Style::default()
                .fg(Theme::FG)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .wrap(Wrap { trim: false });
    frame.render_widget(title, chunks[0]);

    // Status line
    let status_line = create_status_line(task);
    frame.render_widget(Paragraph::new(status_line), chunks[1]);

    // Notes
    if let Some(ref notes) = task.notes {
        let notes_para = Paragraph::new(notes.as_str())
            .style(Theme::dimmed_style())
            .wrap(Wrap { trim: false });
        frame.render_widget(notes_para, chunks[2]);
    } else {
        let empty_notes = Paragraph::new("No notes")
            .style(Theme::muted_style());
        frame.render_widget(empty_notes, chunks[2]);
    }

    // Metadata
    let metadata_lines = create_metadata_lines(task);
    let metadata = Paragraph::new(metadata_lines);
    frame.render_widget(metadata, chunks[3]);
}

fn create_status_line(task: &Task) -> Vec<Line<'static>> {
    let mut spans = vec![];

    // Status
    spans.push(Span::styled(
        format!("Status: {} ", task.status.as_str()),
        Theme::dimmed_style(),
    ));

    // Priority
    if task.priority != crate::models::TaskPriority::None {
        spans.push(Span::styled(
            format!("Priority: {} ", task.priority.as_str()),
            Style::default().fg(Theme::priority_color(&task.priority)),
        ));
    }

    // Kind
    if let Some(ref kind) = task.kind {
        spans.push(Span::styled(
            format!("Kind: {} ", kind.as_str()),
            Style::default().fg(Theme::kind_color(kind)),
        ));
    }

    // Size
    if let Some(ref size) = task.size {
        spans.push(Span::styled(
            format!("Size: {} ", size.display()),
            Theme::dimmed_style(),
        ));
    }

    vec![Line::from(spans)]
}

fn create_metadata_lines(task: &Task) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Due date
    if let Some(due) = task.due_date {
        let style = if task.is_overdue() {
            Style::default().fg(Theme::ERROR)
        } else {
            Theme::dimmed_style()
        };
        lines.push(Line::from(Span::styled(
            format!("Due: {}", due),
            style,
        )));
    }

    // Created/Updated
    lines.push(Line::from(Span::styled(
        format!(
            "Created: {} | Updated: {}",
            task.created_at.format("%Y-%m-%d %H:%M"),
            task.updated_at.format("%Y-%m-%d %H:%M")
        ),
        Theme::muted_style(),
    )));

    // Context URL
    if let Some(ref url) = task.context_url {
        lines.push(Line::from(Span::styled(
            format!("URL: {}", url),
            Style::default().fg(Theme::INFO),
        )));
    }

    lines
}
