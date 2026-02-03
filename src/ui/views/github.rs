use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::services::{GitHubData, GitHubIssue};
use crate::ui::components::{render_github_column, GitHubColumnState};
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitHubColumn {
    ReviewPRs,
    MyPRs,
    AssignedIssues,
}

pub struct GitHubView {
    pub review_prs: GitHubColumnState,
    pub my_prs: GitHubColumnState,
    pub assigned_issues: GitHubColumnState,
    pub active_column: GitHubColumn,
    pub loading: bool,
    pub error: Option<String>,
}

impl GitHubView {
    pub fn new() -> Self {
        Self {
            review_prs: GitHubColumnState::new("Review Requested"),
            my_prs: GitHubColumnState::new("My PRs"),
            assigned_issues: GitHubColumnState::new("Assigned Issues"),
            active_column: GitHubColumn::ReviewPRs,
            loading: false,
            error: None,
        }
    }

    pub fn set_data(&mut self, data: GitHubData) {
        self.review_prs.set_items(data.review_prs);
        self.my_prs.set_items(data.my_prs);
        self.assigned_issues.set_items(data.assigned_issues);
        self.loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn next_column(&mut self) {
        self.active_column = match self.active_column {
            GitHubColumn::ReviewPRs => GitHubColumn::MyPRs,
            GitHubColumn::MyPRs => GitHubColumn::AssignedIssues,
            GitHubColumn::AssignedIssues => GitHubColumn::ReviewPRs,
        };
        self.update_focus();
    }

    pub fn prev_column(&mut self) {
        self.active_column = match self.active_column {
            GitHubColumn::ReviewPRs => GitHubColumn::AssignedIssues,
            GitHubColumn::MyPRs => GitHubColumn::ReviewPRs,
            GitHubColumn::AssignedIssues => GitHubColumn::MyPRs,
        };
        self.update_focus();
    }

    fn update_focus(&mut self) {
        self.review_prs.focused = self.active_column == GitHubColumn::ReviewPRs;
        self.my_prs.focused = self.active_column == GitHubColumn::MyPRs;
        self.assigned_issues.focused = self.active_column == GitHubColumn::AssignedIssues;
    }

    pub fn select_next(&mut self) {
        match self.active_column {
            GitHubColumn::ReviewPRs => self.review_prs.select_next(),
            GitHubColumn::MyPRs => self.my_prs.select_next(),
            GitHubColumn::AssignedIssues => self.assigned_issues.select_next(),
        }
    }

    pub fn select_previous(&mut self) {
        match self.active_column {
            GitHubColumn::ReviewPRs => self.review_prs.select_previous(),
            GitHubColumn::MyPRs => self.my_prs.select_previous(),
            GitHubColumn::AssignedIssues => self.assigned_issues.select_previous(),
        }
    }

    pub fn selected_item(&self) -> Option<&GitHubIssue> {
        match self.active_column {
            GitHubColumn::ReviewPRs => self.review_prs.selected_item(),
            GitHubColumn::MyPRs => self.my_prs.selected_item(),
            GitHubColumn::AssignedIssues => self.assigned_issues.selected_item(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.update_focus();

        if self.loading {
            let loading = Paragraph::new("Loading GitHub data...")
                .style(Theme::dimmed_style());
            frame.render_widget(loading, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_msg = Paragraph::new(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Theme::ERROR)),
                Span::raw(error.clone()),
            ]));
            frame.render_widget(error_msg, area);
            return;
        }

        let chunks = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

        render_github_column(frame, chunks[0], &mut self.review_prs);
        render_github_column(frame, chunks[1], &mut self.my_prs);
        render_github_column(frame, chunks[2], &mut self.assigned_issues);
    }
}

impl Default for GitHubView {
    fn default() -> Self {
        Self::new()
    }
}
