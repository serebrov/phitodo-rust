use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub order_index: i64,
    pub is_inbox: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            color: None,
            icon: None,
            order_index: 0,
            is_inbox: false,
            created_at: now,
            updated_at: now,
            deleted: false,
        }
    }

    /// Returns the display icon or a default folder icon
    pub fn display_icon(&self) -> &str {
        self.icon.as_deref().unwrap_or("📁")
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new(String::new())
    }
}
