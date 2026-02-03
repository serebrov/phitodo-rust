use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::Result;
use crate::models::{Project, Tag, Task, TaskKind, TaskPriority, TaskSize, TaskStatus};

pub struct Repository {
    conn: Connection,
}

impl Repository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    // ==================== Tasks ====================

    pub fn get_all_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, notes, created_at, updated_at, due_date, start_date,
                    completed_at, project_id, priority, status, order_index, deleted,
                    kind, size, assignee, context_url, metadata
             FROM tasks WHERE deleted = 0 ORDER BY order_index ASC, created_at DESC",
        )?;

        let task_iter = stmt.query_map([], |row| {
            Ok(self.row_to_task(row))
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            let mut task = task??;
            task.tags = self.get_task_tags(&task.id)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, notes, created_at, updated_at, due_date, start_date,
                    completed_at, project_id, priority, status, order_index, deleted,
                    kind, size, assignee, context_url, metadata
             FROM tasks WHERE id = ?1 AND deleted = 0",
        )?;

        let task = stmt
            .query_row([id], |row| Ok(self.row_to_task(row)))
            .optional()?;

        match task {
            Some(Ok(mut task)) => {
                task.tags = self.get_task_tags(&task.id)?;
                Ok(Some(task))
            }
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn insert_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks (id, title, notes, created_at, updated_at, due_date, start_date,
                               completed_at, project_id, priority, status, order_index, deleted,
                               kind, size, assignee, context_url, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                task.id,
                task.title,
                task.notes,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                task.due_date.map(|d| d.to_string()),
                task.start_date.map(|d| d.to_string()),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.project_id,
                task.priority.as_str(),
                task.status.as_str(),
                task.order_index,
                task.deleted,
                task.kind.map(|k| k.as_str()),
                task.size.map(|s| s.as_str()),
                task.assignee,
                task.context_url,
                if task.metadata.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&task.metadata).unwrap_or_default())
                },
            ],
        )?;

        // Insert task tags
        for tag_id in &task.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![task.id, tag_id],
            )?;
        }

        Ok(())
    }

    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET title = ?2, notes = ?3, updated_at = ?4, due_date = ?5,
                             start_date = ?6, completed_at = ?7, project_id = ?8,
                             priority = ?9, status = ?10, order_index = ?11, deleted = ?12,
                             kind = ?13, size = ?14, assignee = ?15, context_url = ?16,
                             metadata = ?17
             WHERE id = ?1",
            params![
                task.id,
                task.title,
                task.notes,
                task.updated_at.to_rfc3339(),
                task.due_date.map(|d| d.to_string()),
                task.start_date.map(|d| d.to_string()),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.project_id,
                task.priority.as_str(),
                task.status.as_str(),
                task.order_index,
                task.deleted,
                task.kind.map(|k| k.as_str()),
                task.size.map(|s| s.as_str()),
                task.assignee,
                task.context_url,
                if task.metadata.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&task.metadata).unwrap_or_default())
                },
            ],
        )?;

        // Update tags: remove old, add new
        self.conn.execute(
            "DELETE FROM task_tags WHERE task_id = ?1",
            params![task.id],
        )?;
        for tag_id in &task.tags {
            self.conn.execute(
                "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
                params![task.id, tag_id],
            )?;
        }

        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET deleted = 1, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn get_task_tags(&self, task_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT tag_id FROM task_tags WHERE task_id = ?1",
        )?;
        let tags: Vec<String> = stmt
            .query_map([task_id], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tags)
    }

    fn row_to_task(&self, row: &rusqlite::Row) -> Result<Task> {
        let created_at: String = row.get(3)?;
        let updated_at: String = row.get(4)?;
        let due_date: Option<String> = row.get(5)?;
        let start_date: Option<String> = row.get(6)?;
        let completed_at: Option<String> = row.get(7)?;
        let priority: String = row.get(9)?;
        let status: String = row.get(10)?;
        let kind: Option<String> = row.get(13)?;
        let size: Option<String> = row.get(14)?;
        let metadata: Option<String> = row.get(17)?;

        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            notes: row.get(2)?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            due_date: due_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            start_date: start_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            completed_at: completed_at.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map(|d| d.with_timezone(&Utc))
                    .ok()
            }),
            project_id: row.get(8)?,
            priority: TaskPriority::from_str(&priority),
            status: TaskStatus::from_str(&status),
            order_index: row.get(11)?,
            deleted: row.get(12)?,
            kind: kind.and_then(|k| TaskKind::from_str(&k)),
            size: size.and_then(|s| TaskSize::from_str(&s)),
            assignee: row.get(15)?,
            context_url: row.get(16)?,
            tags: Vec::new(), // Filled in later
            metadata: metadata
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default(),
        })
    }

    // ==================== Projects ====================

    pub fn get_all_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, icon, order_index, is_inbox, created_at, updated_at, deleted
             FROM projects WHERE deleted = 0 ORDER BY order_index ASC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                let created_at: String = row.get(7)?;
                let updated_at: String = row.get(8)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    icon: row.get(4)?,
                    order_index: row.get(5)?,
                    is_inbox: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    deleted: row.get(9)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, color, icon, order_index, is_inbox, created_at, updated_at, deleted
             FROM projects WHERE id = ?1 AND deleted = 0",
        )?;

        let project = stmt
            .query_row([id], |row| {
                let created_at: String = row.get(7)?;
                let updated_at: String = row.get(8)?;

                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    color: row.get(3)?,
                    icon: row.get(4)?,
                    order_index: row.get(5)?,
                    is_inbox: row.get(6)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    deleted: row.get(9)?,
                })
            })
            .optional()?;

        Ok(project)
    }

    pub fn insert_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, description, color, icon, order_index, is_inbox, created_at, updated_at, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                project.id,
                project.name,
                project.description,
                project.color,
                project.icon,
                project.order_index,
                project.is_inbox,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
                project.deleted,
            ],
        )?;
        Ok(())
    }

    pub fn update_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET name = ?2, description = ?3, color = ?4, icon = ?5,
                                order_index = ?6, is_inbox = ?7, updated_at = ?8, deleted = ?9
             WHERE id = ?1",
            params![
                project.id,
                project.name,
                project.description,
                project.color,
                project.icon,
                project.order_index,
                project.is_inbox,
                project.updated_at.to_rfc3339(),
                project.deleted,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET deleted = 1, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ==================== Tags ====================

    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, created_at, updated_at, deleted
             FROM tags WHERE deleted = 0 ORDER BY name ASC",
        )?;

        let tags = stmt
            .query_map([], |row| {
                let created_at: String = row.get(3)?;
                let updated_at: String = row.get(4)?;

                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    deleted: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tags)
    }

    pub fn get_tag(&self, id: &str) -> Result<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, created_at, updated_at, deleted
             FROM tags WHERE id = ?1 AND deleted = 0",
        )?;

        let tag = stmt
            .query_row([id], |row| {
                let created_at: String = row.get(3)?;
                let updated_at: String = row.get(4)?;

                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    deleted: row.get(5)?,
                })
            })
            .optional()?;

        Ok(tag)
    }

    pub fn insert_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tags (id, name, color, created_at, updated_at, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                tag.id,
                tag.name,
                tag.color,
                tag.created_at.to_rfc3339(),
                tag.updated_at.to_rfc3339(),
                tag.deleted,
            ],
        )?;
        Ok(())
    }

    pub fn update_tag(&self, tag: &Tag) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET name = ?2, color = ?3, updated_at = ?4, deleted = ?5 WHERE id = ?1",
            params![
                tag.id,
                tag.name,
                tag.color,
                tag.updated_at.to_rfc3339(),
                tag.deleted,
            ],
        )?;
        Ok(())
    }

    pub fn delete_tag(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET deleted = 1, updated_at = ?2 WHERE id = ?1",
            params![id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // ==================== Stats ====================

    pub fn count_tasks_by_status(&self, status: TaskStatus) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = ?1 AND deleted = 0",
            params![status.as_str()],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_tasks_due_today(&self) -> Result<i64> {
        let today = Utc::now().date_naive().to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE due_date = ?1 AND status != 'completed' AND deleted = 0",
            params![today],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_overdue_tasks(&self) -> Result<i64> {
        let today = Utc::now().date_naive().to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE due_date < ?1 AND status != 'completed' AND deleted = 0",
            params![today],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_tasks_for_project(&self, project_id: &str) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND status != 'completed' AND deleted = 0",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_next_order_index(&self, table: &str) -> Result<i64> {
        let query = format!(
            "SELECT COALESCE(MAX(order_index), 0) + 1 FROM {} WHERE deleted = 0",
            table
        );
        let index: i64 = self.conn.query_row(&query, [], |row| row.get(0))?;
        Ok(index)
    }
}
