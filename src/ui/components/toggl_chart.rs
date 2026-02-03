use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    symbols,
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::services::{format_hours, TogglData, TogglTimeEntry};
use crate::ui::theme::Theme;

pub struct TogglChartState {
    pub data: TogglData,
    pub days: i64,
    pub focused: bool,
}

impl Default for TogglChartState {
    fn default() -> Self {
        Self {
            data: TogglData::default(),
            days: 7,
            focused: false,
        }
    }
}

pub fn render_toggl_view(frame: &mut Frame, area: Rect, state: &TogglChartState) {
    let chunks = Layout::vertical([
        Constraint::Length(12), // Bar chart
        Constraint::Min(5),     // Entries list
        Constraint::Length(8),  // Project distribution
    ])
    .split(area);

    render_duration_chart(frame, chunks[0], state);
    render_entries_list(frame, chunks[1], state);
    render_project_distribution(frame, chunks[2], state);
}

fn render_duration_chart(frame: &mut Frame, area: Rect, state: &TogglChartState) {
    let block = Block::default()
        .title(" Duration by Day ")
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(state.focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Get last N days of data
    let today = chrono::Utc::now().date_naive();
    let mut bars: Vec<Bar> = Vec::new();

    for i in (0..state.days).rev() {
        let date = today - chrono::Duration::days(i);
        let duration = state.data.duration_for_date(date);
        let hours = duration as f64 / 3600.0;

        let label = date.format("%a").to_string();
        let value = (hours * 10.0) as u64; // Scale for display

        bars.push(
            Bar::default()
                .value(value)
                .label(Line::from(label))
                .text_value(format_hours(duration))
                .style(Style::default().fg(Theme::PRIMARY)),
        );
    }

    let bar_chart = BarChart::default()
        .bar_width(7)
        .bar_gap(2)
        .group_gap(0)
        .bar_style(Style::default().fg(Theme::PRIMARY))
        .value_style(Style::default().fg(Theme::FG))
        .label_style(Style::default().fg(Theme::FG_DIM))
        .data(BarGroup::default().bars(&bars))
        .max(100); // Max 10 hours

    frame.render_widget(bar_chart, inner);
}

fn render_entries_list(frame: &mut Frame, area: Rect, state: &TogglChartState) {
    let block = Block::default()
        .title(" Recent Entries ")
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(false));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let entries_by_date = state.data.entries_by_date();
    let mut items: Vec<ListItem> = Vec::new();

    for (date, entries) in entries_by_date.iter().take(3) {
        // Date header
        items.push(ListItem::new(Line::from(Span::styled(
            date.format("  %A, %B %d").to_string(),
            Theme::dimmed_style(),
        ))));

        // Entries for this date
        for entry in entries.iter().take(5) {
            items.push(create_entry_item(entry));
        }

        if entries.len() > 5 {
            items.push(ListItem::new(Line::from(Span::styled(
                format!("    ... and {} more", entries.len() - 5),
                Theme::muted_style(),
            ))));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  No time entries",
            Theme::muted_style(),
        ))));
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn create_entry_item(entry: &TogglTimeEntry) -> ListItem<'static> {
    let description = entry
        .description
        .clone()
        .unwrap_or_else(|| "(no description)".to_string());
    let project = entry
        .project_name
        .clone()
        .unwrap_or_else(|| "No Project".to_string());

    ListItem::new(Line::from(vec![
        Span::raw("    "),
        Span::styled(entry.format_duration_short(), Style::default().fg(Theme::PRIMARY)),
        Span::raw(" "),
        Span::styled(truncate(&description, 30), Style::default().fg(Theme::FG)),
        Span::raw(" "),
        Span::styled(format!("[{}]", truncate(&project, 15)), Style::default().fg(Theme::FG_DIM)),
    ]))
}

fn render_project_distribution(frame: &mut Frame, area: Rect, state: &TogglChartState) {
    let block = Block::default()
        .title(" Project Distribution ")
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(false));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let by_project = state.data.duration_by_project();
    let total: i64 = by_project.iter().map(|(_, d)| *d).sum();

    if total == 0 {
        let empty = Paragraph::new("No data").style(Theme::muted_style());
        frame.render_widget(empty, inner);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    let colors = [Theme::PRIMARY, Theme::SECONDARY, Theme::ACCENT, Theme::WARNING, Theme::INFO];

    for (i, (project, duration)) in by_project.iter().take(5).enumerate() {
        let percentage = (*duration as f64 / total as f64 * 100.0) as u16;
        let bar_width = (percentage as usize * inner.width as usize / 100).min(inner.width as usize - 25);
        let color = colors[i % colors.len()];

        lines.push(Line::from(vec![
            Span::styled(format!("{:>15} ", truncate(project, 15)), Theme::dimmed_style()),
            Span::styled(
                symbols::block::FULL.repeat(bar_width),
                Style::default().fg(color),
            ),
            Span::styled(
                format!(" {} ({:.0}%)", format_hours(*duration), percentage),
                Style::default().fg(Theme::FG_DIM),
            ),
        ]));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
