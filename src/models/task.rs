use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    #[default]
    Inbox,
    Active,
    Scheduled,
    Completed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Inbox => "inbox",
            TaskStatus::Active => "active",
            TaskStatus::Scheduled => "scheduled",
            TaskStatus::Completed => "completed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "inbox" => TaskStatus::Inbox,
            "active" => TaskStatus::Active,
            "scheduled" => TaskStatus::Scheduled,
            "completed" => TaskStatus::Completed,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Inbox,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    #[default]
    None,
    Low,
    Medium,
    High,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::None => "none",
            TaskPriority::Low => "low",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "none" => TaskPriority::None,
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            _ => TaskPriority::None,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskPriority::None => " ",
            TaskPriority::Low => "!",
            TaskPriority::Medium => "!!",
            TaskPriority::High => "!!!",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskKind {
    Task,
    Bug,
    Feature,
    Chore,
    #[serde(rename = "gh:issue")]
    GhIssue,
    #[serde(rename = "gh:pr")]
    GhPr,
    #[serde(rename = "gh:review")]
    GhReview,
}

impl TaskKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskKind::Task => "task",
            TaskKind::Bug => "bug",
            TaskKind::Feature => "feature",
            TaskKind::Chore => "chore",
            TaskKind::GhIssue => "gh:issue",
            TaskKind::GhPr => "gh:pr",
            TaskKind::GhReview => "gh:review",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "task" => Some(TaskKind::Task),
            "bug" => Some(TaskKind::Bug),
            "feature" => Some(TaskKind::Feature),
            "chore" => Some(TaskKind::Chore),
            "gh:issue" => Some(TaskKind::GhIssue),
            "gh:pr" => Some(TaskKind::GhPr),
            "gh:review" => Some(TaskKind::GhReview),
            _ => None,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            TaskKind::Task => "[T]",
            TaskKind::Bug => "[B]",
            TaskKind::Feature => "[F]",
            TaskKind::Chore => "[C]",
            TaskKind::GhIssue => "[ISS]",
            TaskKind::GhPr => "[PR]",
            TaskKind::GhReview => "[REV]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskSize {
    Xs,
    S,
    M,
    L,
}

impl TaskSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskSize::Xs => "xs",
            TaskSize::S => "s",
            TaskSize::M => "m",
            TaskSize::L => "l",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "xs" => Some(TaskSize::Xs),
            "s" => Some(TaskSize::S),
            "m" => Some(TaskSize::M),
            "l" => Some(TaskSize::L),
            _ => None,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            TaskSize::Xs => "XS",
            TaskSize::S => "S",
            TaskSize::M => "M",
            TaskSize::L => "L",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<NaiveDate>,
    pub start_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub project_id: Option<String>,
    pub priority: TaskPriority,
    pub tags: Vec<String>,
    pub status: TaskStatus,
    pub order_index: i64,
    pub deleted: bool,
    pub kind: Option<TaskKind>,
    pub size: Option<TaskSize>,
    pub assignee: Option<String>,
    pub context_url: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            notes: None,
            created_at: now,
            updated_at: now,
            due_date: None,
            start_date: None,
            completed_at: None,
            project_id: None,
            priority: TaskPriority::None,
            tags: Vec::new(),
            status: TaskStatus::Inbox,
            order_index: 0,
            deleted: false,
            kind: None,
            size: None,
            assignee: None,
            context_url: None,
            metadata: HashMap::new(),
        }
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Utc::now().date_naive();
            due < today && !self.is_completed()
        } else {
            false
        }
    }

    pub fn is_due_today(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Utc::now().date_naive();
            due == today
        } else {
            false
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new(String::new())
    }
}
