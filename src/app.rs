use std::sync::mpsc;
use chrono::Utc;
use rusqlite::Connection;

use crate::config::Config;
use crate::db::{init_database, Repository};
use crate::error::Result;
use crate::models::{Project, Tag, Task, TaskPriority, TaskStatus};
use crate::services::{GitHubData, GitHubIssue, GitHubService, TogglData, TogglService};
use crate::ui::components::{
    ConfirmModal, InputState, NotificationModal, SidebarCounts, SidebarState, TaskFormState,
};
use crate::ui::theme::SidebarItem;
use crate::ui::views::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Input,
    TaskForm,
    Confirm,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Sidebar,
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Completed,
    Project,
    Tag,
    Review,
    GitHub,
    Toggl,
    Settings,
}

pub enum AsyncMessage {
    GitHubDataReady(std::result::Result<GitHubData, String>),
    TogglDataReady(std::result::Result<TogglData, String>),
}

pub struct App {
    pub config: Config,
    pub mode: AppMode,
    pub focus: FocusArea,
    pub current_view: CurrentView,
    pub show_help: bool,

    // Data
    pub tasks: Vec<Task>,
    pub projects: Vec<Project>,
    pub tags: Vec<Tag>,

    // UI State
    pub sidebar: SidebarState,
    pub inbox_view: InboxView,
    pub today_view: TodayView,
    pub upcoming_view: UpcomingView,
    pub anytime_view: AnytimeView,
    pub completed_view: CompletedView,
    pub project_view: ProjectView,
    pub tag_view: TagView,
    pub review_view: ReviewView,
    pub github_view: GitHubView,
    pub toggl_view: TogglView,
    pub settings_view: SettingsView,

    // Input / Modals
    pub input: InputState,
    pub task_form: Option<TaskFormState>,
    pub confirm_modal: Option<ConfirmModal>,
    pub notification: Option<NotificationModal>,
    pub pending_delete_id: Option<String>,

    // Async
    pub async_rx: mpsc::Receiver<AsyncMessage>,
    pub async_tx: mpsc::Sender<AsyncMessage>,

    // Database path for creating new connections
    db_path: std::path::PathBuf,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let db_path = Config::database_path()?;

        // Initialize database
        {
            let conn = Connection::open(&db_path)?;
            init_database(&conn)?;
        }

        let (tx, rx) = mpsc::channel();

        let mut app = Self {
            config: config.clone(),
            mode: AppMode::Normal,
            focus: FocusArea::List,
            current_view: CurrentView::Inbox,
            show_help: false,

            tasks: Vec::new(),
            projects: Vec::new(),
            tags: Vec::new(),

            sidebar: SidebarState::default(),
            inbox_view: InboxView::new(),
            today_view: TodayView::new(),
            upcoming_view: UpcomingView::new(),
            anytime_view: AnytimeView::new(),
            completed_view: CompletedView::new(),
            project_view: ProjectView::new(),
            tag_view: TagView::new(),
            review_view: ReviewView::new(),
            github_view: GitHubView::new(),
            toggl_view: TogglView::new(),
            settings_view: SettingsView::new(config),

            input: InputState::new(""),
            task_form: None,
            confirm_modal: None,
            notification: None,
            pending_delete_id: None,

            async_rx: rx,
            async_tx: tx,

            db_path,
        };

        app.load_data()?;
        Ok(app)
    }

    fn get_repo(&self) -> Result<Repository> {
        let conn = Connection::open(&self.db_path)?;
        Ok(Repository::new(conn))
    }

    pub fn load_data(&mut self) -> Result<()> {
        let repo = self.get_repo()?;
        self.tasks = repo.get_all_tasks()?;
        self.projects = repo.get_all_projects()?;
        self.tags = repo.get_all_tags()?;

        self.update_sidebar_counts();
        self.update_views();

        Ok(())
    }

    fn update_sidebar_counts(&mut self) {
        use crate::services::*;

        self.sidebar.counts = SidebarCounts {
            inbox: filter_inbox(&self.tasks).len() as i64,
            today: filter_today(&self.tasks).len() as i64,
            upcoming: filter_upcoming(&self.tasks).len() as i64,
            anytime: filter_anytime(&self.tasks).len() as i64,
            completed: filter_completed(&self.tasks).len() as i64,
            review: filter_review(&self.tasks).len() as i64,
        };
        self.sidebar.projects = self.projects.clone();
    }

    fn update_views(&mut self) {
        self.inbox_view.update_tasks(&self.tasks);
        self.today_view.update_tasks(&self.tasks);
        self.upcoming_view.update_tasks(&self.tasks);
        self.anytime_view.update_tasks(&self.tasks);
        self.completed_view.update_tasks(&self.tasks);
        self.project_view.update_tasks(&self.tasks);
        self.tag_view.update_tasks(&self.tasks);
        self.review_view.update_tasks(&self.tasks);
    }

    pub fn switch_to_view(&mut self, item: SidebarItem) {
        self.sidebar.selected_item = item;
        self.sidebar.selected_project = None;

        self.current_view = match item {
            SidebarItem::Inbox => CurrentView::Inbox,
            SidebarItem::Today => CurrentView::Today,
            SidebarItem::Upcoming => CurrentView::Upcoming,
            SidebarItem::Anytime => CurrentView::Anytime,
            SidebarItem::Completed => CurrentView::Completed,
            SidebarItem::Review => CurrentView::Review,
            SidebarItem::GitHub => CurrentView::GitHub,
            SidebarItem::Toggl => CurrentView::Toggl,
            SidebarItem::Settings => CurrentView::Settings,
        };

        if self.current_view == CurrentView::Settings {
            self.mode = AppMode::Settings;
        } else {
            self.mode = AppMode::Normal;
        }

        // Trigger data loading for GitHub/Toggl
        if self.current_view == CurrentView::GitHub {
            self.fetch_github_data();
        } else if self.current_view == CurrentView::Toggl {
            self.fetch_toggl_data();
        }
    }

    pub fn switch_to_project(&mut self, project_id: &str) {
        if let Some(project) = self.projects.iter().find(|p| p.id == project_id).cloned() {
            self.sidebar.selected_project = Some(project_id.to_string());
            self.current_view = CurrentView::Project;
            self.project_view.set_project(Some(project));
            self.project_view.update_tasks(&self.tasks);
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Sidebar => FocusArea::List,
            FocusArea::List => FocusArea::Detail,
            FocusArea::Detail => FocusArea::Sidebar,
        };
    }

    pub fn cycle_focus_reverse(&mut self) {
        self.focus = match self.focus {
            FocusArea::Sidebar => FocusArea::Detail,
            FocusArea::List => FocusArea::Sidebar,
            FocusArea::Detail => FocusArea::List,
        };
    }

    pub fn select_next(&mut self) {
        if self.focus == FocusArea::Sidebar {
            self.sidebar.select_next();
            return;
        }
        match self.current_view {
            CurrentView::Inbox => self.inbox_view.task_list.select_next(),
            CurrentView::Today => self.today_view.task_list.select_next(),
            CurrentView::Upcoming => self.upcoming_view.task_list.select_next(),
            CurrentView::Anytime => self.anytime_view.task_list.select_next(),
            CurrentView::Completed => self.completed_view.task_list.select_next(),
            CurrentView::Project => self.project_view.task_list.select_next(),
            CurrentView::Tag => self.tag_view.task_list.select_next(),
            CurrentView::Review => self.review_view.task_list.select_next(),
            CurrentView::GitHub => self.github_view.select_next(),
            _ => {}
        }
    }

    pub fn select_previous(&mut self) {
        if self.focus == FocusArea::Sidebar {
            self.sidebar.select_previous();
            return;
        }
        match self.current_view {
            CurrentView::Inbox => self.inbox_view.task_list.select_previous(),
            CurrentView::Today => self.today_view.task_list.select_previous(),
            CurrentView::Upcoming => self.upcoming_view.task_list.select_previous(),
            CurrentView::Anytime => self.anytime_view.task_list.select_previous(),
            CurrentView::Completed => self.completed_view.task_list.select_previous(),
            CurrentView::Project => self.project_view.task_list.select_previous(),
            CurrentView::Tag => self.tag_view.task_list.select_previous(),
            CurrentView::Review => self.review_view.task_list.select_previous(),
            CurrentView::GitHub => self.github_view.select_previous(),
            _ => {}
        }
    }

    pub fn select_first(&mut self) {
        if self.focus == FocusArea::Sidebar {
            self.sidebar.select_first();
            return;
        }
        match self.current_view {
            CurrentView::Inbox => self.inbox_view.task_list.select_first(),
            CurrentView::Today => self.today_view.task_list.select_first(),
            CurrentView::Upcoming => self.upcoming_view.task_list.select_first(),
            CurrentView::Anytime => self.anytime_view.task_list.select_first(),
            CurrentView::Completed => self.completed_view.task_list.select_first(),
            CurrentView::Project => self.project_view.task_list.select_first(),
            CurrentView::Tag => self.tag_view.task_list.select_first(),
            CurrentView::Review => self.review_view.task_list.select_first(),
            _ => {}
        }
    }

    pub fn select_last(&mut self) {
        if self.focus == FocusArea::Sidebar {
            self.sidebar.select_last();
            return;
        }
        match self.current_view {
            CurrentView::Inbox => self.inbox_view.task_list.select_last(),
            CurrentView::Today => self.today_view.task_list.select_last(),
            CurrentView::Upcoming => self.upcoming_view.task_list.select_last(),
            CurrentView::Anytime => self.anytime_view.task_list.select_last(),
            CurrentView::Completed => self.completed_view.task_list.select_last(),
            CurrentView::Project => self.project_view.task_list.select_last(),
            CurrentView::Tag => self.tag_view.task_list.select_last(),
            CurrentView::Review => self.review_view.task_list.select_last(),
            _ => {}
        }
    }

    pub fn activate_selected(&mut self) {
        if self.focus == FocusArea::Sidebar {
            // Check if a project is selected
            if let Some(ref project_id) = self.sidebar.selected_project.clone() {
                self.switch_to_project(&project_id);
                self.focus = FocusArea::List;
            } else {
                // Switch to the selected view
                self.switch_to_view(self.sidebar.selected_item);
                self.focus = FocusArea::List;
            }
        }
    }

    // Task operations
    fn selected_task(&self) -> Option<&Task> {
        match self.current_view {
            CurrentView::Inbox => self.inbox_view.selected_task(),
            CurrentView::Today => self.today_view.selected_task(),
            CurrentView::Upcoming => self.upcoming_view.selected_task(),
            CurrentView::Anytime => self.anytime_view.selected_task(),
            CurrentView::Completed => self.completed_view.selected_task(),
            CurrentView::Project => self.project_view.selected_task(),
            CurrentView::Tag => self.tag_view.selected_task(),
            CurrentView::Review => self.review_view.selected_task(),
            _ => None,
        }
    }

    pub fn toggle_task_completed(&mut self) {
        if let Some(task) = self.selected_task().cloned() {
            if let Ok(repo) = self.get_repo() {
                if let Some(mut t) = self.tasks.iter().find(|t| t.id == task.id).cloned() {
                    if t.status == TaskStatus::Completed {
                        t.status = TaskStatus::Inbox;
                        t.completed_at = None;
                    } else {
                        t.status = TaskStatus::Completed;
                        t.completed_at = Some(Utc::now());
                    }
                    t.updated_at = Utc::now();
                    let _ = repo.update_task(&t);
                    let _ = self.load_data();
                }
            }
        }
    }

    pub fn set_task_priority(&mut self, priority: TaskPriority) {
        if let Some(task) = self.selected_task().cloned() {
            if let Ok(repo) = self.get_repo() {
                if let Some(mut t) = self.tasks.iter().find(|t| t.id == task.id).cloned() {
                    t.priority = priority;
                    t.updated_at = Utc::now();
                    let _ = repo.update_task(&t);
                    let _ = self.load_data();
                }
            }
        }
    }

    pub fn set_task_status(&mut self, status: TaskStatus) {
        if let Some(task) = self.selected_task().cloned() {
            if let Ok(repo) = self.get_repo() {
                if let Some(mut t) = self.tasks.iter().find(|t| t.id == task.id).cloned() {
                    t.status = status;
                    t.updated_at = Utc::now();
                    if status == TaskStatus::Completed {
                        t.completed_at = Some(Utc::now());
                    } else {
                        t.completed_at = None;
                    }
                    let _ = repo.update_task(&t);
                    let _ = self.load_data();
                }
            }
        }
    }

    pub fn start_new_task(&mut self) {
        self.task_form = Some(TaskFormState::new_task(self.projects.clone()));
        self.mode = AppMode::TaskForm;
    }

    pub fn start_edit_task(&mut self) {
        if let Some(task) = self.selected_task().cloned() {
            self.task_form = Some(TaskFormState::edit_task(task, self.projects.clone()));
            self.mode = AppMode::TaskForm;
        }
    }

    pub fn save_task_form(&mut self) {
        let form_data = if let Some(ref mut form) = self.task_form {
            form.apply_inputs();

            if form.title_input.is_empty() {
                self.show_error("Task title cannot be empty".to_string());
                return;
            }

            Some((
                form.is_new,
                form.title_input.clone(),
                form.task.notes.clone(),
                form.task.due_date,
                form.task.priority,
                form.task.status,
                form.task.kind,
                form.task.size,
                form.task.id.clone(),
            ))
        } else {
            None
        };

        if let Some((is_new, title, notes, due_date, priority, status, kind, size, id)) = form_data {
            if let Ok(repo) = self.get_repo() {
                if is_new {
                    let mut task = Task::new(title);
                    task.notes = notes;
                    task.due_date = due_date;
                    task.priority = priority;
                    task.status = status;
                    task.kind = kind;
                    task.size = size;

                    if let Ok(idx) = repo.get_next_order_index("tasks") {
                        task.order_index = idx;
                    }

                    if let Err(e) = repo.insert_task(&task) {
                        self.show_error(format!("Failed to create task: {}", e));
                    }
                } else {
                    if let Some(mut task) = self.tasks.iter().find(|t| t.id == id).cloned() {
                        task.title = title;
                        task.notes = notes;
                        task.due_date = due_date;
                        task.priority = priority;
                        task.status = status;
                        task.kind = kind;
                        task.size = size;
                        task.updated_at = Utc::now();
                        if let Err(e) = repo.update_task(&task) {
                            self.show_error(format!("Failed to update task: {}", e));
                        }
                    }
                }
                let _ = self.load_data();
            }
        }

        self.task_form = None;
        self.mode = AppMode::Normal;
    }

    pub fn start_new_project(&mut self) {
        self.input = InputState::new("Project name:").with_placeholder("Enter project name");
        self.mode = AppMode::Input;
    }

    pub fn start_delete(&mut self) {
        let task_info = self.selected_task().map(|t| (t.title.clone(), t.id.clone()));
        if let Some((title, id)) = task_info {
            self.confirm_modal = Some(ConfirmModal::delete(&title));
            self.pending_delete_id = Some(id);
            self.mode = AppMode::Confirm;
        }
    }

    pub fn execute_confirm(&mut self) {
        if let Some(id) = self.pending_delete_id.take() {
            if let Ok(repo) = self.get_repo() {
                let _ = repo.delete_task(&id);
                let _ = self.load_data();
            }
        }
        self.confirm_modal = None;
        self.mode = AppMode::Normal;
    }

    pub fn start_search(&mut self) {
        self.input = InputState::new("/").with_placeholder("Search tasks...");
        self.mode = AppMode::Input;
    }

    pub fn cancel_input(&mut self) {
        self.input.clear();
        self.mode = AppMode::Normal;
    }

    pub fn submit_input(&mut self) {
        let value = self.input.value.clone();
        let prompt = self.input.prompt.clone();
        self.input.clear();
        self.mode = AppMode::Normal;

        // Handle based on prompt
        if prompt == "Project name:" && !value.is_empty() {
            if let Ok(repo) = self.get_repo() {
                let mut project = Project::new(value);
                if let Ok(idx) = repo.get_next_order_index("projects") {
                    project.order_index = idx;
                }
                let _ = repo.insert_project(&project);
                let _ = self.load_data();
            }
        }
    }

    pub fn refresh_data(&mut self) {
        let _ = self.load_data();

        if self.current_view == CurrentView::GitHub {
            self.fetch_github_data();
        } else if self.current_view == CurrentView::Toggl {
            self.fetch_toggl_data();
        }
    }

    pub fn show_error(&mut self, message: String) {
        self.notification = Some(NotificationModal::error(message));
    }

    pub fn show_info(&mut self, message: String) {
        self.notification = Some(NotificationModal::info(message));
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    // Async operations
    pub fn fetch_github_data(&mut self) {
        let Some(ref token) = self.config.github_token else {
            self.github_view.set_error("GitHub token not configured. Set it in Settings.".to_string());
            return;
        };

        if token.is_empty() {
            self.github_view.set_error("GitHub token not configured. Set it in Settings.".to_string());
            return;
        }

        self.github_view.set_loading(true);
        let token = token.clone();
        let tx = self.async_tx.clone();

        tokio::spawn(async move {
            let service = GitHubService::new(token);
            let result = service.fetch_all().await;
            let _ = tx.send(AsyncMessage::GitHubDataReady(
                result.map_err(|e| e.to_string()),
            ));
        });
    }

    pub fn fetch_toggl_data(&mut self) {
        let Some(ref token) = self.config.toggl_token else {
            self.toggl_view.set_error("Toggl token not configured. Set it in Settings.".to_string());
            return;
        };

        if token.is_empty() {
            self.toggl_view.set_error("Toggl token not configured. Set it in Settings.".to_string());
            return;
        }

        self.toggl_view.set_loading(true);
        let token = token.clone();
        let tx = self.async_tx.clone();

        tokio::spawn(async move {
            let service = TogglService::new(token);
            let result = service.fetch_all(7).await;
            let _ = tx.send(AsyncMessage::TogglDataReady(
                result.map_err(|e| e.to_string()),
            ));
        });
    }

    /// Sync GitHub items to local tasks
    fn sync_github_to_tasks(&mut self, data: &GitHubData) {
        let repo = match self.get_repo() {
            Ok(r) => r,
            Err(_) => return,
        };

        // Collect all GitHub items with their type
        let mut github_items: Vec<(&GitHubIssue, &str)> = Vec::new();
        for issue in &data.assigned_issues {
            github_items.push((issue, "issue"));
        }
        for pr in &data.my_prs {
            github_items.push((pr, "my_pr"));
        }
        for pr in &data.review_prs {
            github_items.push((pr, "review"));
        }

        // Build a map of repo names to project IDs, creating projects as needed
        let mut repo_to_project: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        // First, map existing projects by checking metadata for github_repo
        for project in &self.projects {
            // Check if project name matches a repo pattern (owner/repo or just repo)
            // We'll use the project name as the key
            repo_to_project.insert(project.name.clone(), project.id.clone());
        }

        // Track which GitHub URLs we've seen (to mark closed items)
        let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (item, github_type) in &github_items {
            seen_urls.insert(item.html_url.clone());
            let repo_name = item.repo_name();

            // Get or create project for this repo
            let project_id = if let Some(id) = repo_to_project.get(&repo_name) {
                Some(id.clone())
            } else {
                // Create a new project for this repo
                let mut project = Project::new(repo_name.clone());
                project.icon = Some("".to_string()); // GitHub icon
                if let Ok(idx) = repo.get_next_order_index("projects") {
                    project.order_index = idx;
                }
                let project_id = project.id.clone();
                if repo.insert_project(&project).is_ok() {
                    repo_to_project.insert(repo_name.clone(), project_id.clone());
                    Some(project_id)
                } else {
                    None
                }
            };

            // Check if task already exists with this URL
            let existing_task = self.tasks.iter().find(|t| {
                t.context_url.as_ref() == Some(&item.html_url)
            });

            if let Some(task) = existing_task {
                // Task exists - check if we need to update it
                let mut needs_update = false;
                let mut updated_task = task.clone();

                if item.state == "closed" && task.status != TaskStatus::Completed {
                    updated_task.status = TaskStatus::Completed;
                    updated_task.completed_at = Some(Utc::now());
                    needs_update = true;
                }

                // Update project assignment if not set
                if task.project_id.is_none() && project_id.is_some() {
                    updated_task.project_id = project_id.clone();
                    needs_update = true;
                }

                if needs_update {
                    updated_task.updated_at = Utc::now();
                    let _ = repo.update_task(&updated_task);
                }
            } else {
                // Create new task
                let mut task = Task::new(item.title.clone());
                task.context_url = Some(item.html_url.clone());
                task.status = TaskStatus::Inbox;
                task.project_id = project_id;
                task.notes = item.body.clone();
                task.metadata.insert("github_id".to_string(), item.id.to_string());
                task.metadata.insert("github_type".to_string(), github_type.to_string());
                task.metadata.insert("github_repo".to_string(), repo_name);

                // Set task kind based on GitHub type
                task.kind = match *github_type {
                    "issue" => Some(crate::models::TaskKind::GhIssue),
                    "my_pr" => Some(crate::models::TaskKind::GhPr),
                    "review" => Some(crate::models::TaskKind::GhReview),
                    _ => None,
                };

                if let Ok(idx) = repo.get_next_order_index("tasks") {
                    task.order_index = idx;
                }
                let _ = repo.insert_task(&task);
            }
        }

        // Check for tasks that were synced from GitHub but the item is now closed
        for task in &self.tasks {
            if let Some(ref url) = task.context_url {
                if url.contains("github.com") && !seen_urls.contains(url) {
                    // This GitHub item is no longer in our open lists - it's closed
                    if task.status != TaskStatus::Completed {
                        let mut updated_task = task.clone();
                        updated_task.status = TaskStatus::Completed;
                        updated_task.completed_at = Some(Utc::now());
                        updated_task.updated_at = Utc::now();
                        let _ = repo.update_task(&updated_task);
                    }
                }
            }
        }

        // Reload tasks to reflect changes
        let _ = self.load_data();
    }

    pub fn poll_async_messages(&mut self) {
        while let Ok(msg) = self.async_rx.try_recv() {
            match msg {
                AsyncMessage::GitHubDataReady(result) => {
                    match result {
                        Ok(data) => {
                            self.sync_github_to_tasks(&data);
                            self.github_view.set_data(data);
                        }
                        Err(e) => self.github_view.set_error(e),
                    }
                }
                AsyncMessage::TogglDataReady(result) => {
                    match result {
                        Ok(data) => self.toggl_view.set_data(data),
                        Err(e) => self.toggl_view.set_error(e),
                    }
                }
            }
        }
    }
}
