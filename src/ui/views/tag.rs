use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::models::{Tag, Task};
use crate::services::filter_by_tag;
use crate::ui::components::{render_task_detail, render_task_list, TaskListState};

pub struct TagView {
    pub task_list: TaskListState,
    pub detail_focused: bool,
    pub tag: Option<Tag>,
}

impl TagView {
    pub fn new() -> Self {
        Self {
            task_list: TaskListState::new("Tag"),
            detail_focused: false,
            tag: None,
        }
    }

    pub fn set_tag(&mut self, tag: Option<Tag>) {
        let title = tag
            .as_ref()
            .map(|t| format!("# {}", t.name))
            .unwrap_or_else(|| "No Tag".to_string());
        self.task_list.title = title;
        self.tag = tag;
    }

    pub fn update_tasks(&mut self, all_tasks: &[Task]) {
        let filtered: Vec<Task> = if let Some(ref tag) = self.tag {
            filter_by_tag(all_tasks, &tag.id)
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

impl Default for TagView {
    fn default() -> Self {
        Self::new()
    }
}
