use base64::Engine;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{AppError, Result};

const TOGGL_API_BASE: &str = "https://api.track.toggl.com/api/v9";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogglTimeEntry {
    pub id: i64,
    pub description: Option<String>,
    pub duration: i64, // seconds (negative if running)
    pub start: String,
    pub stop: Option<String>,
    #[serde(alias = "pid")]
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
}

impl TogglTimeEntry {
    /// Get the start date as NaiveDate
    pub fn start_date(&self) -> Option<NaiveDate> {
        DateTime::parse_from_rfc3339(&self.start)
            .ok()
            .map(|dt| dt.date_naive())
    }

    /// Get duration in seconds (0 if running)
    pub fn duration_secs(&self) -> i64 {
        if self.duration < 0 {
            // Running timer - calculate from start
            let start = DateTime::parse_from_rfc3339(&self.start).ok();
            if let Some(start) = start {
                let now = Utc::now();
                (now - start.with_timezone(&Utc)).num_seconds()
            } else {
                0
            }
        } else {
            self.duration
        }
    }

    /// Format duration as HH:MM:SS
    pub fn format_duration(&self) -> String {
        let secs = self.duration_secs();
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Format duration as short string (e.g., "2h 30m")
    pub fn format_duration_short(&self) -> String {
        let secs = self.duration_secs();
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogglProject {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct TogglData {
    pub entries: Vec<TogglTimeEntry>,
    pub projects: HashMap<i64, String>,
}

impl TogglData {
    /// Get total duration for a specific date
    pub fn duration_for_date(&self, date: NaiveDate) -> i64 {
        self.entries
            .iter()
            .filter(|e| e.start_date() == Some(date))
            .map(|e| e.duration_secs())
            .sum()
    }

    /// Get duration by project
    pub fn duration_by_project(&self) -> Vec<(String, i64)> {
        let mut by_project: HashMap<String, i64> = HashMap::new();

        for entry in &self.entries {
            let project_name = entry
                .project_name
                .clone()
                .or_else(|| {
                    entry
                        .project_id
                        .and_then(|id| self.projects.get(&id).cloned())
                })
                .unwrap_or_else(|| "No Project".to_string());

            *by_project.entry(project_name).or_default() += entry.duration_secs();
        }

        let mut result: Vec<_> = by_project.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by duration descending
        result
    }

    /// Get entries grouped by date
    pub fn entries_by_date(&self) -> Vec<(NaiveDate, Vec<&TogglTimeEntry>)> {
        use std::collections::BTreeMap;

        let mut by_date: BTreeMap<NaiveDate, Vec<&TogglTimeEntry>> = BTreeMap::new();

        for entry in &self.entries {
            if let Some(date) = entry.start_date() {
                by_date.entry(date).or_default().push(entry);
            }
        }

        by_date.into_iter().rev().collect() // Most recent first
    }
}

pub struct TogglService {
    client: reqwest::Client,
    token: String,
}

impl TogglService {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    /// Fetch time entries for a date range
    pub async fn fetch_time_entries(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<TogglTimeEntry>> {
        let url = format!(
            "{}/me/time_entries?start_date={}&end_date={}&meta=true",
            TOGGL_API_BASE,
            start_date,
            end_date
        );

        let response = self.fetch_with_auth(&url).await?;
        let entries: Vec<TogglTimeEntry> = self.parse_entries(&response)?;

        Ok(entries)
    }

    /// Fetch project names
    pub async fn fetch_projects(&self) -> Result<HashMap<i64, String>> {
        let url = format!("{}/me/projects", TOGGL_API_BASE);
        let response = self.fetch_with_auth(&url).await?;

        let projects: Vec<TogglProject> = serde_json::from_str(&response).unwrap_or_default();
        let map: HashMap<i64, String> = projects.into_iter().map(|p| (p.id, p.name)).collect();

        Ok(map)
    }

    /// Fetch all Toggl data for the past N days
    pub async fn fetch_all(&self, days: i64) -> Result<TogglData> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days(days);

        let (entries, projects) = tokio::try_join!(
            self.fetch_time_entries(start_date, end_date),
            self.fetch_projects(),
        )?;

        // Enrich entries with project names
        let entries: Vec<TogglTimeEntry> = entries
            .into_iter()
            .map(|mut e| {
                if e.project_name.is_none() {
                    if let Some(pid) = e.project_id {
                        e.project_name = projects.get(&pid).cloned();
                    }
                }
                e
            })
            .collect();

        Ok(TogglData { entries, projects })
    }

    async fn fetch_with_auth(&self, url: &str) -> Result<String> {
        let auth = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:api_token", self.token));

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let status = response.status();
        if status.as_u16() == 402 {
            return Err(AppError::Toggl("Request limit reached. Try again later.".to_string()));
        }
        if status.as_u16() == 403 {
            return Err(AppError::Toggl("Invalid token. Check Settings.".to_string()));
        }
        if !status.is_success() {
            return Err(AppError::Toggl(format!("HTTP error: {}", status)));
        }

        let text = response.text().await?;
        Ok(text)
    }

    fn parse_entries(&self, response: &str) -> Result<Vec<TogglTimeEntry>> {
        // Toggl API can return entries directly or wrapped in { items: [...] }
        if let Ok(entries) = serde_json::from_str::<Vec<TogglTimeEntry>>(response) {
            return Ok(entries);
        }

        #[derive(Deserialize)]
        struct Wrapped {
            items: Vec<TogglTimeEntry>,
        }

        if let Ok(wrapped) = serde_json::from_str::<Wrapped>(response) {
            return Ok(wrapped.items);
        }

        Ok(Vec::new())
    }
}

/// Format seconds as hours with one decimal (e.g., "2.5h")
pub fn format_hours(seconds: i64) -> String {
    let hours = seconds as f64 / 3600.0;
    format!("{:.1}h", hours)
}
