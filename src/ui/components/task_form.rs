use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::models::{Project, Task, TaskKind, TaskPriority, TaskSize, TaskStatus};
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFormField {
    Title,
    Notes,
    DueDate,
    Project,
    Priority,
    Status,
    Kind,
    Size,
}

impl TaskFormField {
    pub fn all() -> &'static [TaskFormField] {
        &[
            TaskFormField::Title,
            TaskFormField::Notes,
            TaskFormField::DueDate,
            TaskFormField::Project,
            TaskFormField::Priority,
            TaskFormField::Status,
            TaskFormField::Kind,
            TaskFormField::Size,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TaskFormField::Title => "Title",
            TaskFormField::Notes => "Notes",
            TaskFormField::DueDate => "Due Date",
            TaskFormField::Project => "Project",
            TaskFormField::Priority => "Priority",
            TaskFormField::Status => "Status",
            TaskFormField::Kind => "Kind",
            TaskFormField::Size => "Size",
        }
    }
}

pub struct TaskFormState {
    pub task: Task,
    pub is_new: bool,
    pub current_field: TaskFormField,
    pub title_input: String,
    pub notes_input: String,
    pub due_date_input: String,
    pub available_projects: Vec<Project>,
    pub selected_project_index: Option<usize>,
}

impl TaskFormState {
    pub fn new_task(projects: Vec<Project>) -> Self {
        Self {
            task: Task::new(String::new()),
            is_new: true,
            current_field: TaskFormField::Title,
            title_input: String::new(),
            notes_input: String::new(),
            due_date_input: String::new(),
            available_projects: projects,
            selected_project_index: None,
        }
    }

    pub fn edit_task(task: Task, projects: Vec<Project>) -> Self {
        let title_input = task.title.clone();
        let notes_input = task.notes.clone().unwrap_or_default();
        let due_date_input = task.due_date.map(|d| d.to_string()).unwrap_or_default();

        // Find current project index
        let selected_project_index = task.project_id.as_ref().and_then(|pid| {
            projects.iter().position(|p| &p.id == pid)
        });

        Self {
            task,
            is_new: false,
            current_field: TaskFormField::Title,
            title_input,
            notes_input,
            due_date_input,
            available_projects: projects,
            selected_project_index,
        }
    }

    pub fn cycle_project(&mut self) {
        if self.available_projects.is_empty() {
            return;
        }
        self.selected_project_index = match self.selected_project_index {
            None => Some(0),
            Some(i) if i + 1 < self.available_projects.len() => Some(i + 1),
            Some(_) => None,
        };
        // Update task's project_id
        self.task.project_id = self.selected_project_index
            .map(|i| self.available_projects[i].id.clone());
    }

    pub fn selected_project_name(&self) -> &str {
        match self.selected_project_index {
            Some(i) => &self.available_projects[i].name,
            None => "none",
        }
    }

    pub fn next_field(&mut self) {
        let fields = TaskFormField::all();
        if let Some(pos) = fields.iter().position(|&f| f == self.current_field) {
            self.current_field = fields[(pos + 1) % fields.len()];
        }
    }

    pub fn prev_field(&mut self) {
        let fields = TaskFormField::all();
        if let Some(pos) = fields.iter().position(|&f| f == self.current_field) {
            self.current_field = fields[(pos + fields.len() - 1) % fields.len()];
        }
    }

    pub fn cycle_priority(&mut self) {
        self.task.priority = match self.task.priority {
            TaskPriority::None => TaskPriority::Low,
            TaskPriority::Low => TaskPriority::Medium,
            TaskPriority::Medium => TaskPriority::High,
            TaskPriority::High => TaskPriority::None,
        };
    }

    pub fn cycle_status(&mut self) {
        self.task.status = match self.task.status {
            TaskStatus::Inbox => TaskStatus::Active,
            TaskStatus::Active => TaskStatus::Scheduled,
            TaskStatus::Scheduled => TaskStatus::Completed,
            TaskStatus::Completed => TaskStatus::Cancelled,
            TaskStatus::Cancelled => TaskStatus::Inbox,
        };
    }

    pub fn cycle_kind(&mut self) {
        self.task.kind = match self.task.kind {
            None => Some(TaskKind::Task),
            Some(TaskKind::Task) => Some(TaskKind::Bug),
            Some(TaskKind::Bug) => Some(TaskKind::Feature),
            Some(TaskKind::Feature) => Some(TaskKind::Chore),
            Some(TaskKind::Chore) => Some(TaskKind::GhIssue),
            Some(TaskKind::GhIssue) => Some(TaskKind::GhPr),
            Some(TaskKind::GhPr) => Some(TaskKind::GhReview),
            Some(TaskKind::GhReview) => None,
        };
    }

    pub fn cycle_size(&mut self) {
        self.task.size = match self.task.size {
            None => Some(TaskSize::Xs),
            Some(TaskSize::Xs) => Some(TaskSize::S),
            Some(TaskSize::S) => Some(TaskSize::M),
            Some(TaskSize::M) => Some(TaskSize::L),
            Some(TaskSize::L) => None,
        };
    }

    pub fn apply_inputs(&mut self) {
        self.task.title = self.title_input.clone();
        self.task.notes = if self.notes_input.is_empty() {
            None
        } else {
            Some(self.notes_input.clone())
        };
        self.task.due_date = chrono::NaiveDate::parse_from_str(&self.due_date_input, "%Y-%m-%d").ok();
    }
}

pub fn render_task_form(frame: &mut Frame, area: Rect, state: &TaskFormState) {
    // Center the form
    let width = area.width.min(60);
    let height = area.height.min(22);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let form_area = Rect::new(x, y, width, height);

    // Clear background
    frame.render_widget(Clear, form_area);

    let title = if state.is_new {
        " New Task "
    } else {
        " Edit Task "
    };

    let block = Block::default()
        .title(title)
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(true))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(form_area);
    frame.render_widget(block, form_area);

    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Length(3), // Notes
        Constraint::Length(3), // Due Date
        Constraint::Length(2), // Project
        Constraint::Length(2), // Priority
        Constraint::Length(2), // Status
        Constraint::Length(2), // Kind
        Constraint::Length(2), // Size
        Constraint::Min(1),    // Help text
    ])
    .split(inner);

    // Title field
    render_text_field(
        frame,
        chunks[0],
        "Title",
        &state.title_input,
        state.current_field == TaskFormField::Title,
    );

    // Notes field
    render_text_field(
        frame,
        chunks[1],
        "Notes",
        &state.notes_input,
        state.current_field == TaskFormField::Notes,
    );

    // Due Date field
    render_text_field(
        frame,
        chunks[2],
        "Due Date (YYYY-MM-DD)",
        &state.due_date_input,
        state.current_field == TaskFormField::DueDate,
    );

    // Project field
    render_select_field(
        frame,
        chunks[3],
        "Project",
        state.selected_project_name(),
        state.current_field == TaskFormField::Project,
    );

    // Priority field
    render_select_field(
        frame,
        chunks[4],
        "Priority",
        state.task.priority.as_str(),
        state.current_field == TaskFormField::Priority,
    );

    // Status field
    render_select_field(
        frame,
        chunks[5],
        "Status",
        state.task.status.as_str(),
        state.current_field == TaskFormField::Status,
    );

    // Kind field
    render_select_field(
        frame,
        chunks[6],
        "Kind",
        state.task.kind.map(|k| k.as_str()).unwrap_or("none"),
        state.current_field == TaskFormField::Kind,
    );

    // Size field
    render_select_field(
        frame,
        chunks[7],
        "Size",
        state.task.size.map(|s| s.display()).unwrap_or("none"),
        state.current_field == TaskFormField::Size,
    );

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("/"),
        Span::styled("Shift+Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate | "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Save | "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .style(Theme::muted_style());
    frame.render_widget(help, chunks[8]);
}

fn render_text_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let style = if focused {
        Style::default().fg(Theme::PRIMARY)
    } else {
        Theme::dimmed_style()
    };

    let content = if value.is_empty() && focused {
        "_"
    } else {
        value
    };

    let field = Paragraph::new(Line::from(vec![
        Span::styled(format!("{}: ", label), style),
        Span::styled(
            content,
            if focused {
                Style::default().fg(Theme::FG).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Theme::FG)
            },
        ),
    ]));
    frame.render_widget(field, area);
}

fn render_select_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let style = if focused {
        Style::default().fg(Theme::PRIMARY)
    } else {
        Theme::dimmed_style()
    };

    let field = Paragraph::new(Line::from(vec![
        Span::styled(format!("{}: ", label), style),
        Span::styled(
            format!("< {} >", value),
            if focused {
                Style::default().fg(Theme::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::FG)
            },
        ),
    ]));
    frame.render_widget(field, area);
}
