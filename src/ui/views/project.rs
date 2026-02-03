use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::models::{Project, Task};
use crate::services::filter_by_project;
use crate::ui::components::{render_task_detail, render_task_list, TaskListState};

pub struct ProjectView {
    pub task_list: TaskListState,
    pub detail_focused: bool,
    pub project: Option<Project>,
}

impl ProjectView {
    pub fn new() -> Self {
        Self {
            task_list: TaskListState::new("Project"),
            detail_focused: false,
            project: None,
        }
    }

    pub fn set_project(&mut self, project: Option<Project>) {
        let title = project
            .as_ref()
            .map(|p| format!("{} {}", p.display_icon(), p.name))
            .unwrap_or_else(|| "No Project".to_string());
        self.task_list.title = title;
        self.project = project;
    }

    pub fn update_tasks(&mut self, all_tasks: &[Task]) {
        let filtered: Vec<Task> = if let Some(ref project) = self.project {
            filter_by_project(all_tasks, &project.id)
                .into_iter()
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        self.task_list.set_tasks(filtered);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::horizontal([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

        render_task_list(frame, chunks[0], &mut self.task_list);
        render_task_detail(
            frame,
            chunks[1],
            self.task_list.selected_task(),
            self.detail_focused,
        );
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.task_list.selected_task()
    }
}

impl Default for ProjectView {
    fn default() -> Self {
        Self::new()
    }
}
