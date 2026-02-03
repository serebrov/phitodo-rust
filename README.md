# phitodo-tui

A terminal UI clone of phitodo task management, built with Rust using ratatui.

## Features

- Full task management (create, edit, complete, delete)
- Multiple views: Inbox, Today, Upcoming, Anytime, Completed, Review
- Project and tag organization
- GitHub integration (PR reviews, My PRs, Assigned Issues)
- Toggl time tracking integration with bar charts
- Vim-like keyboard navigation
- Configurable settings

## Installation

```bash
cargo build --release
./target/release/phitodo-tui
```

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `Alt+1-9` | Switch views (Inbox, Today, Upcoming, Anytime, Completed, Review, GitHub, Toggl, Settings) |
| `j/k` or `Down/Up` | Navigate list |
| `g/G` | Go to first/last item |
| `Tab` | Cycle focus (sidebar -> list -> detail) |
| `Enter` | Open selected item |

### Task Actions
| Key | Action |
|-----|--------|
| `Space` | Toggle task completion |
| `n` | New task |
| `N` | New project |
| `e` | Edit selected |
| `d` | Delete (with confirmation) |
| `1-4` | Set priority (None/Low/Medium/High) |
| `i/a/s` | Move to Inbox/Active/Scheduled |

### Other
| Key | Action |
|-----|--------|
| `/` | Search/filter |
| `r` | Refresh data |
| `?` | Show/hide help |
| `q` | Quit |

## Configuration

Configuration is stored at `~/.config/phitodo-tui/config.toml`:

```toml
shortcut_modifier = "alt"
github_token = "ghp_..."
github_repos = ["owner/repo"]
toggl_token = "..."
toggl_hidden_projects = ["Internal"]
```

Database is stored at `~/.local/share/phitodo-tui/phitodo.db`

## Views

1. **Inbox** - Tasks with status=inbox
2. **Today** - Tasks due today or overdue
3. **Upcoming** - Tasks with future due dates
4. **Anytime** - Tasks with no due date
5. **Completed** - Completed tasks
6. **Review** - Overdue tasks
7. **GitHub** - 3-column view: Review PRs | My PRs | Assigned Issues
8. **Toggl** - Time entries with bar chart and project distribution
9. **Settings** - GitHub token, Toggl token configuration

## Tech Stack

- **TUI Framework**: ratatui + crossterm
- **Async Runtime**: tokio
- **Database**: rusqlite (bundled SQLite)
- **HTTP Client**: reqwest
- **Serialization**: serde, serde_json, toml
- **Date/Time**: chrono
