use serde::{Deserialize, Serialize};
use crate::error::{AppError, Result};

const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub body: Option<String>,
    pub repository: Option<GitHubRepository>,
    pub repository_url: Option<String>,
    pub user: Option<GitHubUser>,
    pub pull_request: Option<serde_json::Value>,
}

impl GitHubIssue {
    /// Get the repository full name (owner/repo)
    pub fn repo_name(&self) -> String {
        if let Some(ref repo) = self.repository {
            return repo.full_name.clone();
        }

        // Try to extract from repository_url
        if let Some(ref url) = self.repository_url {
            if let Some(name) = extract_repo_from_url(url) {
                return name;
            }
        }

        // Try to extract from html_url
        if let Some(name) = extract_repo_from_html_url(&self.html_url) {
            return name;
        }

        "unknown".to_string()
    }

    /// Check if this is a pull request
    pub fn is_pr(&self) -> bool {
        self.pull_request.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSearchResult {
    pub total_count: i64,
    pub items: Vec<GitHubIssue>,
}

#[derive(Debug, Clone, Default)]
pub struct GitHubData {
    pub review_prs: Vec<GitHubIssue>,
    pub my_prs: Vec<GitHubIssue>,
    pub assigned_issues: Vec<GitHubIssue>,
}

pub struct GitHubService {
    client: reqwest::Client,
    token: String,
}

impl GitHubService {
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    /// Fetch issues assigned to the authenticated user (excluding PRs)
    pub async fn fetch_assigned_issues(&self) -> Result<Vec<GitHubIssue>> {
        let url = format!("{}/issues?filter=assigned&state=open&per_page=100", GITHUB_API_BASE);
        let response = self.fetch_with_auth(&url).await?;
        let issues: Vec<GitHubIssue> = serde_json::from_str(&response)?;

        // Filter out pull requests
        let issues: Vec<GitHubIssue> = issues
            .into_iter()
            .filter(|i| !i.is_pr())
            .map(normalize_issue)
            .collect();

        Ok(issues)
    }

    /// Fetch PRs requesting review from the authenticated user
    pub async fn fetch_review_requested_prs(&self) -> Result<Vec<GitHubIssue>> {
        let url = format!(
            "{}/search/issues?q=review-requested:@me is:open is:pr&per_page=100",
            GITHUB_API_BASE
        );
        let response = self.fetch_with_auth(&url).await?;
        let search_result: GitHubSearchResult = serde_json::from_str(&response)?;

        let prs: Vec<GitHubIssue> = search_result
            .items
            .into_iter()
            .map(normalize_issue)
            .collect();

        Ok(prs)
    }

    /// Fetch PRs authored by the authenticated user
    pub async fn fetch_my_open_prs(&self) -> Result<Vec<GitHubIssue>> {
        let url = format!(
            "{}/search/issues?q=author:@me is:open is:pr&per_page=100",
            GITHUB_API_BASE
        );
        let response = self.fetch_with_auth(&url).await?;
        let search_result: GitHubSearchResult = serde_json::from_str(&response)?;

        let prs: Vec<GitHubIssue> = search_result
            .items
            .into_iter()
            .map(normalize_issue)
            .collect();

        Ok(prs)
    }

    /// Fetch all GitHub data in parallel
    pub async fn fetch_all(&self) -> Result<GitHubData> {
        let (review_prs, my_prs, assigned_issues) = tokio::try_join!(
            self.fetch_review_requested_prs(),
            self.fetch_my_open_prs(),
            self.fetch_assigned_issues(),
        )?;

        Ok(GitHubData {
            review_prs,
            my_prs,
            assigned_issues,
        })
    }

    async fn fetch_with_auth(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "phitodo-tui")
            .send()
            .await?;

        let status = response.status();
        if status == 401 {
            return Err(AppError::GitHub("Invalid token. Check Settings.".to_string()));
        }
        if !status.is_success() {
            return Err(AppError::GitHub(format!("HTTP error: {}", status)));
        }

        let text = response.text().await?;
        Ok(text)
    }
}

/// Extract owner/repo from repository_url (e.g., https://api.github.com/repos/owner/repo)
fn extract_repo_from_url(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 2 {
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];
        Some(format!("{}/{}", owner, repo))
    } else {
        None
    }
}

/// Extract owner/repo from html_url (e.g., https://github.com/owner/repo/issues/123)
fn extract_repo_from_html_url(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();
    // URL format: https://github.com/owner/repo/...
    if parts.len() >= 5 && parts[2] == "github.com" {
        Some(format!("{}/{}", parts[3], parts[4]))
    } else {
        None
    }
}

/// Normalize an issue to ensure repository.full_name is always populated
fn normalize_issue(mut issue: GitHubIssue) -> GitHubIssue {
    if issue.repository.is_none() {
        let full_name = issue.repo_name();
        issue.repository = Some(GitHubRepository { full_name });
    }
    issue
}
