use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::Project;
use crate::ui::theme::{SidebarItem, Theme};

pub struct SidebarState {
    pub selected_item: SidebarItem,
    pub selected_project: Option<String>,
    pub projects: Vec<Project>,
    pub focused: bool,
    pub counts: SidebarCounts,
}

#[derive(Default)]
pub struct SidebarCounts {
    pub inbox: i64,
    pub today: i64,
    pub upcoming: i64,
    pub anytime: i64,
    pub completed: i64,
    pub review: i64,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            selected_item: SidebarItem::Inbox,
            selected_project: None,
            projects: Vec::new(),
            focused: false,
            counts: SidebarCounts::default(),
        }
    }
}

impl SidebarState {
    pub fn select_next(&mut self) {
        // If a project is selected, navigate within projects or to footer
        if let Some(ref proj_id) = self.selected_project {
            if let Some(pos) = self.projects.iter().position(|p| &p.id == proj_id) {
                if pos + 1 < self.projects.len() {
                    // Move to next project
                    self.selected_project = Some(self.projects[pos + 1].id.clone());
                } else {
                    // Move to Review (first footer item)
                    self.selected_project = None;
                    self.selected_item = SidebarItem::Review;
                }
            }
            return;
        }

        let items = SidebarItem::all();
        if let Some(pos) = items.iter().position(|&i| i == self.selected_item) {
            // At Completed, go to projects if any, otherwise skip to Review
            if self.selected_item == SidebarItem::Completed && !self.projects.is_empty() {
                self.selected_project = Some(self.projects[0].id.clone());
            } else {
                let next_pos = (pos + 1) % items.len();
                self.selected_item = items[next_pos];
            }
        }
    }

    pub fn select_previous(&mut self) {
        // If a project is selected, navigate within projects or to main items
        if let Some(ref proj_id) = self.selected_project {
            if let Some(pos) = self.projects.iter().position(|p| &p.id == proj_id) {
                if pos > 0 {
                    // Move to previous project
                    self.selected_project = Some(self.projects[pos - 1].id.clone());
                } else {
                    // Move to Completed (last main item before projects)
                    self.selected_project = None;
                    self.selected_item = SidebarItem::Completed;
                }
            }
            return;
        }

        let items = SidebarItem::all();
        if let Some(pos) = items.iter().position(|&i| i == self.selected_item) {
            // At Review, go to projects if any, otherwise go to Completed
            if self.selected_item == SidebarItem::Review && !self.projects.is_empty() {
                self.selected_project = Some(self.projects.last().unwrap().id.clone());
            } else {
                let prev_pos = if pos == 0 { items.len() - 1 } else { pos - 1 };
                self.selected_item = items[prev_pos];
            }
        }
    }

    pub fn select_first(&mut self) {
        self.selected_project = None;
        self.selected_item = SidebarItem::Inbox;
    }

    pub fn select_last(&mut self) {
        self.selected_project = None;
        self.selected_item = SidebarItem::Settings;
    }
}

pub fn render_sidebar(frame: &mut Frame, area: Rect, state: &SidebarState) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Theme::border_style(state.focused))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: header, main items, projects, footer items
    let chunks = Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Length(7), // Main nav (5 items + spacing)
        Constraint::Min(3),    // Projects
        Constraint::Length(6), // Footer nav (4 items + spacing)
    ])
    .split(inner);

    // Header
    render_header(frame, chunks[0]);

    // Main navigation items (Inbox, Today, Upcoming, Anytime, Completed)
    render_main_nav(frame, chunks[1], state);

    // Projects section
    render_projects(frame, chunks[2], state);

    // Footer items (Review, GitHub, Toggl, Settings)
    render_footer_nav(frame, chunks[3], state);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            " φ phitodo",
            Style::default()
                .fg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(" Personal tasks", Theme::dimmed_style())),
    ]);
    frame.render_widget(header, area);
}

fn render_main_nav(frame: &mut Frame, area: Rect, state: &SidebarState) {
    let items: Vec<ListItem> = [
        SidebarItem::Inbox,
        SidebarItem::Today,
        SidebarItem::Upcoming,
        SidebarItem::Anytime,
        SidebarItem::Completed,
    ]
    .iter()
    .map(|item| create_nav_item(item, state, get_count(item, &state.counts)))
    .collect();

    let list = List::new(items).style(Style::default().bg(Theme::BG_SECONDARY));
    frame.render_widget(list, area);
}

fn render_projects(frame: &mut Frame, area: Rect, state: &SidebarState) {
    if area.height < 2 {
        return;
    }

    // Projects header
    let header = Line::from(vec![
        Span::styled(" Projects", Theme::dimmed_style()),
    ]);
    frame.render_widget(Paragraph::new(header), Rect { height: 1, ..area });

    if state.projects.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No projects",
            Theme::muted_style(),
        ));
        frame.render_widget(
            empty,
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
        return;
    }

    let items: Vec<ListItem> = state
        .projects
        .iter()
        .map(|project| {
            let is_selected = state.selected_project.as_ref() == Some(&project.id);
            let style = if is_selected {
                Theme::selected_style()
            } else {
                Style::default().fg(Theme::FG)
            };

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(project.display_icon(), style),
                Span::raw(" "),
                Span::styled(&project.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(Theme::BG_SECONDARY));
    frame.render_widget(
        list,
        Rect {
            y: area.y + 1,
            height: area.height.saturating_sub(1),
            ..area
        },
    );
}

fn render_footer_nav(frame: &mut Frame, area: Rect, state: &SidebarState) {
    let items: Vec<ListItem> = [
        SidebarItem::Review,
        SidebarItem::GitHub,
        SidebarItem::Toggl,
        SidebarItem::Settings,
    ]
    .iter()
    .map(|item| create_nav_item(item, state, get_count(item, &state.counts)))
    .collect();

    let list = List::new(items).style(Style::default().bg(Theme::BG_SECONDARY));
    frame.render_widget(list, area);
}

fn create_nav_item(item: &SidebarItem, state: &SidebarState, count: Option<i64>) -> ListItem<'static> {
    let is_selected = state.selected_item == *item && state.selected_project.is_none();
    let style = if is_selected {
        Theme::selected_style()
    } else {
        Style::default().fg(Theme::FG)
    };

    let shortcut_style = if is_selected {
        Theme::selected_style()
    } else {
        Theme::muted_style()
    };

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(item.icon(), style),
        Span::raw(" "),
        Span::styled(item.label(), style),
    ];

    // Add count if available
    if let Some(c) = count {
        if c > 0 {
            spans.push(Span::styled(
                format!(" {}", c),
                Style::default().fg(Theme::FG_DIM),
            ));
        }
    }

    // Add shortcut hint
    spans.push(Span::styled(
        format!(" {}", item.shortcut()),
        shortcut_style,
    ));

    ListItem::new(Line::from(spans))
}

fn get_count(item: &SidebarItem, counts: &SidebarCounts) -> Option<i64> {
    match item {
        SidebarItem::Inbox => Some(counts.inbox),
        SidebarItem::Today => Some(counts.today),
        SidebarItem::Upcoming => Some(counts.upcoming),
        SidebarItem::Anytime => Some(counts.anytime),
        SidebarItem::Completed => Some(counts.completed),
        SidebarItem::Review => Some(counts.review),
        _ => None,
    }
}
