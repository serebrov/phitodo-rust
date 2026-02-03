use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::Config;
use crate::ui::components::InputState;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    GitHubToken,
    GitHubRepos,
    TogglToken,
    TogglHiddenProjects,
}

impl SettingsField {
    pub fn all() -> &'static [SettingsField] {
        &[
            SettingsField::GitHubToken,
            SettingsField::GitHubRepos,
            SettingsField::TogglToken,
            SettingsField::TogglHiddenProjects,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsField::GitHubToken => "GitHub Token",
            SettingsField::GitHubRepos => "GitHub Repos (comma-separated)",
            SettingsField::TogglToken => "Toggl Token",
            SettingsField::TogglHiddenProjects => "Toggl Hidden Projects (comma-separated)",
        }
    }
}

pub struct SettingsView {
    pub config: Config,
    pub current_field: SettingsField,
    pub editing: bool,
    pub input: InputState,
    pub saved_message: Option<String>,
}

impl SettingsView {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_field: SettingsField::GitHubToken,
            editing: false,
            input: InputState::new(""),
            saved_message: None,
        }
    }

    pub fn next_field(&mut self) {
        if self.editing {
            return;
        }
        let fields = SettingsField::all();
        if let Some(pos) = fields.iter().position(|&f| f == self.current_field) {
            self.current_field = fields[(pos + 1) % fields.len()];
        }
    }

    pub fn prev_field(&mut self) {
        if self.editing {
            return;
        }
        let fields = SettingsField::all();
        if let Some(pos) = fields.iter().position(|&f| f == self.current_field) {
            self.current_field = fields[(pos + fields.len() - 1) % fields.len()];
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
        let value = match self.current_field {
            SettingsField::GitHubToken => self.config.github_token.clone().unwrap_or_default(),
            SettingsField::GitHubRepos => self.config.github_repos.join(", "),
            SettingsField::TogglToken => self.config.toggl_token.clone().unwrap_or_default(),
            SettingsField::TogglHiddenProjects => self.config.toggl_hidden_projects.join(", "),
        };
        self.input = InputState::new("").with_value(value);
    }

    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.input.clear();
    }

    pub fn save_field(&mut self) {
        let value = self.input.value.clone();
        match self.current_field {
            SettingsField::GitHubToken => {
                self.config.github_token = if value.is_empty() { None } else { Some(value) };
            }
            SettingsField::GitHubRepos => {
                self.config.github_repos = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            SettingsField::TogglToken => {
                self.config.toggl_token = if value.is_empty() { None } else { Some(value) };
            }
            SettingsField::TogglHiddenProjects => {
                self.config.toggl_hidden_projects = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        self.editing = false;
        self.input.clear();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Settings ")
            .title_style(Theme::title_style())
            .borders(Borders::ALL)
            .border_style(Theme::border_style(true));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::vertical([
            Constraint::Length(3), // GitHub Token
            Constraint::Length(3), // GitHub Repos
            Constraint::Length(3), // Toggl Token
            Constraint::Length(3), // Toggl Hidden Projects
            Constraint::Min(1),    // Help text
        ])
        .split(inner);

        // GitHub Token
        render_settings_field(
            frame,
            chunks[0],
            SettingsField::GitHubToken,
            mask_token(self.config.github_token.as_deref()),
            self.current_field == SettingsField::GitHubToken,
            self.editing && self.current_field == SettingsField::GitHubToken,
            &self.input,
        );

        // GitHub Repos
        render_settings_field(
            frame,
            chunks[1],
            SettingsField::GitHubRepos,
            if self.config.github_repos.is_empty() {
                "(none)".to_string()
            } else {
                self.config.github_repos.join(", ")
            },
            self.current_field == SettingsField::GitHubRepos,
            self.editing && self.current_field == SettingsField::GitHubRepos,
            &self.input,
        );

        // Toggl Token
        render_settings_field(
            frame,
            chunks[2],
            SettingsField::TogglToken,
            mask_token(self.config.toggl_token.as_deref()),
            self.current_field == SettingsField::TogglToken,
            self.editing && self.current_field == SettingsField::TogglToken,
            &self.input,
        );

        // Toggl Hidden Projects
        render_settings_field(
            frame,
            chunks[3],
            SettingsField::TogglHiddenProjects,
            if self.config.toggl_hidden_projects.is_empty() {
                "(none)".to_string()
            } else {
                self.config.toggl_hidden_projects.join(", ")
            },
            self.current_field == SettingsField::TogglHiddenProjects,
            self.editing && self.current_field == SettingsField::TogglHiddenProjects,
            &self.input,
        );

        // Help text
        let help = if self.editing {
            Line::from(vec![
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Save | "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Cancel"),
            ])
        } else {
            Line::from(vec![
                Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Navigate | "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Edit | "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Save config"),
            ])
        };
        let help_para = Paragraph::new(help).style(Theme::muted_style());
        frame.render_widget(help_para, chunks[4]);

        // Saved message
        if let Some(ref msg) = self.saved_message {
            let msg_para = Paragraph::new(Line::from(Span::styled(
                msg,
                Style::default().fg(Theme::SUCCESS),
            )));
            frame.render_widget(
                msg_para,
                Rect {
                    y: chunks[4].y + 1,
                    height: 1,
                    ..chunks[4]
                },
            );
        }
    }
}

fn render_settings_field(
    frame: &mut Frame,
    area: Rect,
    field: SettingsField,
    display_value: String,
    selected: bool,
    editing: bool,
    input: &InputState,
) {
    let label_style = if selected {
        Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Theme::dimmed_style()
    };

    let value_style = if selected {
        Style::default().fg(Theme::FG)
    } else {
        Theme::dimmed_style()
    };

    let indicator = if selected { "› " } else { "  " };

    let value = if editing {
        let cursor_pos = input.cursor.min(input.value.len());
        let before = &input.value[..cursor_pos];
        let cursor_char = input.value.chars().nth(cursor_pos).unwrap_or(' ');
        let after = if cursor_pos < input.value.len() {
            &input.value[cursor_pos + 1..]
        } else {
            ""
        };
        format!("{}{}{}▏", before, cursor_char, after)
    } else {
        display_value
    };

    let content = Paragraph::new(vec![
        Line::from(vec![
            Span::raw(indicator),
            Span::styled(field.label(), label_style),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(value, value_style),
        ]),
    ]);
    frame.render_widget(content, area);
}

fn mask_token(token: Option<&str>) -> String {
    match token {
        Some(t) if !t.is_empty() => {
            let len = t.len();
            if len <= 8 {
                "*".repeat(len)
            } else {
                format!("{}...{}", &t[..4], &t[len - 4..])
            }
        }
        _ => "(not set)".to_string(),
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
