mod app;
mod config;
mod db;
mod error;
mod events;
mod models;
mod services;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    Terminal,
};

use app::{App, AppMode, CurrentView, FocusArea};
use config::Config;
use events::handle_key_event;
use ui::components::{
    render_confirm_modal, render_help_overlay, render_input_modal, render_notification,
    render_sidebar, render_task_form, render_status_bar, StatusBarContext,
};
use ui::theme::Theme;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ensure directories exist
    Config::ensure_dirs()?;

    // Load configuration
    let config = Config::load()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(config)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Poll async messages
        app.poll_async_messages();

        // Draw UI
        terminal.draw(|frame| {
            draw_ui(frame, app);
        })?;

        // Handle events with timeout for async polling
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Clear notification on any key
                app.clear_notification();

                if !handle_key_event(app, key) {
                    return Ok(());
                }
            }
        }
    }
}

fn draw_ui(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.size();

    // Clear background
    frame.render_widget(
        ratatui::widgets::Block::default().style(Theme::default_style()),
        area,
    );

    // Main layout: sidebar | content
    let chunks = Layout::horizontal([
        Constraint::Length(25),
        Constraint::Min(40),
    ])
    .split(area);

    // Update sidebar focus state
    app.sidebar.focused = app.focus == FocusArea::Sidebar;

    // Only sync sidebar selection with current view when sidebar is not focused
    // This allows j/k navigation in the sidebar to work independently
    if app.focus != FocusArea::Sidebar {
        app.sidebar.selected_item = match app.current_view {
            CurrentView::Inbox => ui::theme::SidebarItem::Inbox,
            CurrentView::Today => ui::theme::SidebarItem::Today,
            CurrentView::Upcoming => ui::theme::SidebarItem::Upcoming,
            CurrentView::Anytime => ui::theme::SidebarItem::Anytime,
            CurrentView::Completed => ui::theme::SidebarItem::Completed,
            CurrentView::Review => ui::theme::SidebarItem::Review,
            CurrentView::GitHub => ui::theme::SidebarItem::GitHub,
            CurrentView::Toggl => ui::theme::SidebarItem::Toggl,
            CurrentView::Settings => ui::theme::SidebarItem::Settings,
            _ => app.sidebar.selected_item,
        };
    }

    // Render sidebar
    render_sidebar(frame, chunks[0], &app.sidebar);

    // Determine if we should show status bar (for task views)
    let show_status_bar = matches!(
        app.current_view,
        CurrentView::Inbox
            | CurrentView::Today
            | CurrentView::Upcoming
            | CurrentView::Anytime
            | CurrentView::Completed
            | CurrentView::Project
            | CurrentView::Tag
            | CurrentView::Review
    );

    // Split content area to include status bar at bottom
    let content_chunks = if show_status_bar {
        Layout::vertical([Constraint::Min(5), Constraint::Length(1)])
            .split(chunks[1])
    } else {
        Layout::vertical([Constraint::Min(5), Constraint::Length(0)])
            .split(chunks[1])
    };

    let content_area = content_chunks[0];
    let status_area = content_chunks[1];

    // Update list focus state
    let list_focused = app.focus == FocusArea::List;
    let detail_focused = app.focus == FocusArea::Detail;

    // Get selected task info for status bar
    let selected_task_info = match app.current_view {
        CurrentView::Inbox => app.inbox_view.selected_task(),
        CurrentView::Today => app.today_view.selected_task(),
        CurrentView::Upcoming => app.upcoming_view.selected_task(),
        CurrentView::Anytime => app.anytime_view.selected_task(),
        CurrentView::Completed => app.completed_view.selected_task(),
        CurrentView::Project => app.project_view.selected_task(),
        CurrentView::Tag => app.tag_view.selected_task(),
        CurrentView::Review => app.review_view.selected_task(),
        _ => None,
    };

    let has_selection = selected_task_info.is_some();
    let is_completed = selected_task_info.map(|t| t.is_completed()).unwrap_or(false);

    match app.current_view {
        CurrentView::Inbox => {
            app.inbox_view.task_list.focused = list_focused;
            app.inbox_view.detail_focused = detail_focused;
            app.inbox_view.render(frame, content_area);
        }
        CurrentView::Today => {
            app.today_view.task_list.focused = list_focused;
            app.today_view.detail_focused = detail_focused;
            app.today_view.render(frame, content_area);
        }
        CurrentView::Upcoming => {
            app.upcoming_view.task_list.focused = list_focused;
            app.upcoming_view.detail_focused = detail_focused;
            app.upcoming_view.render(frame, content_area);
        }
        CurrentView::Anytime => {
            app.anytime_view.task_list.focused = list_focused;
            app.anytime_view.detail_focused = detail_focused;
            app.anytime_view.render(frame, content_area);
        }
        CurrentView::Completed => {
            app.completed_view.task_list.focused = list_focused;
            app.completed_view.detail_focused = detail_focused;
            app.completed_view.render(frame, content_area);
        }
        CurrentView::Project => {
            app.project_view.task_list.focused = list_focused;
            app.project_view.detail_focused = detail_focused;
            app.project_view.render(frame, content_area);
        }
        CurrentView::Tag => {
            app.tag_view.task_list.focused = list_focused;
            app.tag_view.detail_focused = detail_focused;
            app.tag_view.render(frame, content_area);
        }
        CurrentView::Review => {
            app.review_view.task_list.focused = list_focused;
            app.review_view.detail_focused = detail_focused;
            app.review_view.render(frame, content_area);
        }
        CurrentView::GitHub => {
            app.github_view.render(frame, chunks[1]);
        }
        CurrentView::Toggl => {
            app.toggl_view.render(frame, chunks[1]);
        }
        CurrentView::Settings => {
            app.settings_view.render(frame, chunks[1]);
        }
    }

    // Render status bar for task views
    if show_status_bar {
        let focus_str = match app.focus {
            FocusArea::Sidebar => "sidebar",
            FocusArea::List => "list",
            FocusArea::Detail => "detail",
        };
        let ctx = StatusBarContext {
            has_selection,
            is_completed,
            focus: focus_str,
        };
        render_status_bar(frame, status_area, &ctx);
    }

    // Render overlays
    if let Some(ref form) = app.task_form {
        render_task_form(frame, area, form);
    }

    if let Some(ref modal) = app.confirm_modal {
        render_confirm_modal(frame, area, modal);
    }

    if app.mode == AppMode::Input {
        render_input_modal(frame, area, &app.input, "Input");
    }

    if let Some(ref notification) = app.notification {
        render_notification(frame, area, notification);
    }

    if app.show_help {
        render_help_overlay(frame, area);
    }
}
