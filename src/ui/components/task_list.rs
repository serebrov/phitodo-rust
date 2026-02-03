use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::models::{Task, TaskPriority};
use crate::ui::theme::Theme;

pub struct TaskListState {
    pub tasks: Vec<Task>,
    pub list_state: ListState,
    pub focused: bool,
    pub title: String,
}

impl TaskListState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            tasks: Vec::new(),
            list_state: ListState::default(),
            focused: false,
            title: title.into(),
        }
    }

    pub fn set_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        // Reset selection if out of bounds
        if let Some(selected) = self.list_state.selected() {
            if selected >= self.tasks.len() {
                self.list_state.select(if self.tasks.is_empty() {
                    None
                } else {
                    Some(self.tasks.len() - 1)
                });
            }
        } else if !self.tasks.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.list_state
            .selected()
            .and_then(|i| self.tasks.get(i))
    }

    pub fn select_next(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
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
        if self.tasks.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_first(&mut self) {
        if !self.tasks.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.tasks.is_empty() {
            self.list_state.select(Some(self.tasks.len() - 1));
        }
    }
}

pub fn render_task_list(frame: &mut Frame, area: Rect, state: &mut TaskListState) {
    let block = Block::default()
        .title(format!(" {} ({}) ", state.title, state.tasks.len()))
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(state.focused));

    let items: Vec<ListItem> = state
        .tasks
        .iter()
        .map(|task| create_task_item(task))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Theme::selected_style())
        .highlight_symbol("› ");

    frame.render_stateful_widget(list, area, &mut state.list_state);
}

fn create_task_item(task: &Task) -> ListItem<'static> {
    let checkbox = if task.is_completed() {
        "[x]"
    } else {
        "[ ]"
    };

    let checkbox_style = if task.is_completed() {
        Style::default().fg(Theme::SUCCESS)
    } else {
        Style::default().fg(Theme::FG_DIM)
    };

    let title_style = Theme::status_style(task.is_completed(), task.is_overdue());

    let mut spans = vec![
        Span::styled(checkbox, checkbox_style),
        Span::raw(" "),
    ];

    // Add priority indicator
    if task.priority != TaskPriority::None {
        spans.push(Span::styled(
            task.priority.symbol(),
            Style::default().fg(Theme::priority_color(&task.priority)),
        ));
        spans.push(Span::raw(" "));
    }

    // Add kind indicator
    if let Some(ref kind) = task.kind {
        spans.push(Span::styled(
            kind.symbol(),
            Style::default().fg(Theme::kind_color(kind)),
        ));
        spans.push(Span::raw(" "));
    }

    // Add size indicator
    if let Some(ref size) = task.size {
        spans.push(Span::styled(
            format!("[{}]", size.display()),
            Theme::dimmed_style(),
        ));
        spans.push(Span::raw(" "));
    }

    // Add title
    spans.push(Span::styled(task.title.clone(), title_style));

    // Add due date if present
    if let Some(due) = task.due_date {
        let due_style = if task.is_overdue() {
            Style::default()
                .fg(Theme::ERROR)
                .add_modifier(Modifier::BOLD)
        } else if task.is_due_today() {
            Style::default().fg(Theme::WARNING)
        } else {
            Theme::dimmed_style()
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("({})", due), due_style));
    }

    ListItem::new(Line::from(spans))
}
