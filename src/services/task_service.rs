use chrono::Utc;
use crate::db::Repository;
use crate::error::Result;
use crate::models::{Project, Tag, Task, TaskStatus};

pub struct TaskService<'a> {
    repo: &'a Repository,
}

impl<'a> TaskService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    /// Create a new task
    pub fn create_task(&self, title: String) -> Result<Task> {
        let mut task = Task::new(title);
        task.order_index = self.repo.get_next_order_index("tasks")?;
        self.repo.insert_task(&task)?;
        Ok(task)
    }

    /// Update an existing task
    pub fn update_task(&self, task: &mut Task) -> Result<()> {
        task.updated_at = Utc::now();
        self.repo.update_task(task)?;
        Ok(())
    }

    /// Set task status
    pub fn set_status(&self, task: &mut Task, status: TaskStatus) -> Result<()> {
        task.status = status;
        task.updated_at = Utc::now();
        if status == TaskStatus::Completed {
            task.completed_at = Some(Utc::now());
        } else {
            task.completed_at = None;
        }
        self.repo.update_task(task)?;
        Ok(())
    }

    /// Toggle task completion
    pub fn toggle_completed(&self, task: &mut Task) -> Result<()> {
        if task.status == TaskStatus::Completed {
            self.set_status(task, TaskStatus::Inbox)
        } else {
            self.set_status(task, TaskStatus::Completed)
        }
    }

    /// Soft delete a task
    pub fn delete_task(&self, id: &str) -> Result<()> {
        self.repo.delete_task(id)
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Result<Vec<Task>> {
        self.repo.get_all_tasks()
    }

    /// Get a single task by ID
    pub fn get_task(&self, id: &str) -> Result<Option<Task>> {
        self.repo.get_task(id)
    }

    // ==================== Projects ====================

    /// Create a new project
    pub fn create_project(&self, name: String) -> Result<Project> {
        let mut project = Project::new(name);
        project.order_index = self.repo.get_next_order_index("projects")?;
        self.repo.insert_project(&project)?;
        Ok(project)
    }

    /// Update an existing project
    pub fn update_project(&self, project: &mut Project) -> Result<()> {
        project.updated_at = Utc::now();
        self.repo.update_project(project)?;
        Ok(())
    }

    /// Soft delete a project
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.repo.delete_project(id)
    }

    /// Get all projects
    pub fn get_all_projects(&self) -> Result<Vec<Project>> {
        self.repo.get_all_projects()
    }

    /// Get a single project by ID
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        self.repo.get_project(id)
    }

    // ==================== Tags ====================

    /// Create a new tag
    pub fn create_tag(&self, name: String) -> Result<Tag> {
        let tag = Tag::new(name);
        self.repo.insert_tag(&tag)?;
        Ok(tag)
    }

    /// Update an existing tag
    pub fn update_tag(&self, tag: &mut Tag) -> Result<()> {
        tag.updated_at = Utc::now();
        self.repo.update_tag(tag)?;
        Ok(())
    }

    /// Soft delete a tag
    pub fn delete_tag(&self, id: &str) -> Result<()> {
        self.repo.delete_tag(id)
    }

    /// Get all tags
    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        self.repo.get_all_tags()
    }

    /// Get a single tag by ID
    pub fn get_tag(&self, id: &str) -> Result<Option<Tag>> {
        self.repo.get_tag(id)
    }
}
