use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

impl Tag {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            color: None,
            created_at: now,
            updated_at: now,
            deleted: false,
        }
    }

    /// Returns a display symbol for the tag
    pub fn display_symbol(&self) -> &str {
        "#"
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new(String::new())
    }
}
