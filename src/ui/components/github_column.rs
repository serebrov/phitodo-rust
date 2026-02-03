use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::services::GitHubIssue;
use crate::ui::theme::Theme;

pub struct GitHubColumnState {
    pub items: Vec<GitHubIssue>,
    pub list_state: ListState,
    pub title: String,
    pub focused: bool,
}

impl GitHubColumnState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            list_state: ListState::default(),
            title: title.into(),
            focused: false,
        }
    }

    pub fn set_items(&mut self, items: Vec<GitHubIssue>) {
        self.items = items;
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.items.len() {
                self.list_state.select(if self.items.is_empty() {
                    None
                } else {
                    Some(self.items.len() - 1)
                });
            }
        } else if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn selected_item(&self) -> Option<&GitHubIssue> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn render_github_column(frame: &mut Frame, area: Rect, state: &mut GitHubColumnState) {
    let block = Block::default()
        .title(format!(" {} ({}) ", state.title, state.items.len()))
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(state.focused));

    let items: Vec<ListItem> = state
        .items
        .iter()
        .map(|issue| create_github_item(issue))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Theme::selected_style())
        .highlight_symbol("› ");

    frame.render_stateful_widget(list, area, &mut state.list_state);
}

fn create_github_item(issue: &GitHubIssue) -> ListItem<'static> {
    let repo_name = issue.repo_name();
    let short_repo = repo_name.split('/').last().unwrap_or(&repo_name).to_string();

    let icon = if issue.is_pr() { "" } else { "" };
    let icon_color = if issue.is_pr() {
        Theme::SUCCESS
    } else {
        Theme::INFO
    };

    ListItem::new(Line::from(vec![
        Span::styled(icon.to_string(), Style::default().fg(icon_color)),
        Span::raw(" "),
        Span::styled(
            format!("#{}", issue.number),
            Style::default().fg(Theme::FG_DIM),
        ),
        Span::raw(" "),
        Span::styled(truncate(&issue.title, 40), Style::default().fg(Theme::FG)),
        Span::raw(" "),
        Span::styled(short_repo, Style::default().fg(Theme::FG_MUTED)),
    ]))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
