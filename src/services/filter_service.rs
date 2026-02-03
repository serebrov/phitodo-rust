use chrono::{NaiveDate, Utc};
use crate::models::{Task, TaskStatus};

/// Filter tasks for the Inbox view (status = inbox)
pub fn filter_inbox(tasks: &[Task]) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Inbox && !t.deleted)
        .collect()
}

/// Filter tasks for Today view (due today or overdue, not completed)
pub fn filter_today(tasks: &[Task]) -> Vec<&Task> {
    let today = Utc::now().date_naive();
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.due_date.is_some_and(|due| due <= today)
        })
        .collect()
}

/// Filter tasks for Upcoming view (future due dates, not completed)
pub fn filter_upcoming(tasks: &[Task]) -> Vec<&Task> {
    let today = Utc::now().date_naive();
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.due_date.is_some_and(|due| due > today)
        })
        .collect()
}

/// Filter tasks for Anytime view (no due date, not completed)
pub fn filter_anytime(tasks: &[Task]) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.due_date.is_none()
        })
        .collect()
}

/// Filter tasks for Completed view
pub fn filter_completed(tasks: &[Task]) -> Vec<&Task> {
    tasks
        .iter()
        .filter(|t| !t.deleted && t.status == TaskStatus::Completed)
        .collect()
}

/// Filter tasks by project ID
pub fn filter_by_project<'a>(tasks: &'a [Task], project_id: &str) -> Vec<&'a Task> {
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.project_id.as_deref() == Some(project_id)
        })
        .collect()
}

/// Filter tasks by tag ID
pub fn filter_by_tag<'a>(tasks: &'a [Task], tag_id: &str) -> Vec<&'a Task> {
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.tags.iter().any(|tid| tid == tag_id)
        })
        .collect()
}

/// Filter tasks for Review view (overdue tasks)
pub fn filter_review(tasks: &[Task]) -> Vec<&Task> {
    let today = Utc::now().date_naive();
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && t.status != TaskStatus::Completed
                && t.due_date.is_some_and(|due| due < today)
        })
        .collect()
}

/// Search tasks by title or notes
pub fn search_tasks<'a>(tasks: &'a [Task], query: &str) -> Vec<&'a Task> {
    let query_lower = query.to_lowercase();
    tasks
        .iter()
        .filter(|t| {
            !t.deleted
                && (t.title.to_lowercase().contains(&query_lower)
                    || t.notes
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower)))
        })
        .collect()
}

/// Sort tasks by due date (ascending, nulls last)
pub fn sort_by_due_date(tasks: &mut [&Task]) {
    tasks.sort_by(|a, b| {
        match (&a.due_date, &b.due_date) {
            (Some(a_date), Some(b_date)) => a_date.cmp(b_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.order_index.cmp(&b.order_index),
        }
    });
}

/// Sort tasks by priority (descending)
pub fn sort_by_priority(tasks: &mut [&Task]) {
    tasks.sort_by(|a, b| {
        let a_prio = priority_value(&a.priority);
        let b_prio = priority_value(&b.priority);
        b_prio.cmp(&a_prio)
    });
}

fn priority_value(priority: &crate::models::TaskPriority) -> u8 {
    use crate::models::TaskPriority;
    match priority {
        TaskPriority::High => 3,
        TaskPriority::Medium => 2,
        TaskPriority::Low => 1,
        TaskPriority::None => 0,
    }
}

/// Group tasks by due date
pub fn group_by_date(tasks: Vec<&Task>) -> Vec<(Option<NaiveDate>, Vec<&Task>)> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<Option<NaiveDate>, Vec<&Task>> = BTreeMap::new();

    for task in tasks {
        groups.entry(task.due_date).or_default().push(task);
    }

    groups.into_iter().collect()
}
