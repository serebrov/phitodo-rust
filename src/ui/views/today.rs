use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::models::Task;
use crate::services::filter_today;
use crate::ui::components::{render_task_detail, render_task_list, TaskListState};

pub struct TodayView {
    pub task_list: TaskListState,
    pub detail_focused: bool,
}

impl TodayView {
    pub fn new() -> Self {
        Self {
            task_list: TaskListState::new("Today"),
            detail_focused: false,
        }
    }

    pub fn update_tasks(&mut self, all_tasks: &[Task]) {
        let filtered: Vec<Task> = filter_today(all_tasks)
            .into_iter()
            .cloned()
            .collect();
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

impl Default for TodayView {
    fn default() -> Self {
        Self::new()
    }
}
