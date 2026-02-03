use ratatui::style::{Color, Modifier, Style};

/// App color theme
pub struct Theme;

impl Theme {
    // Base colors - Light theme with better contrast
    pub const BG: Color = Color::Rgb(255, 255, 255);        // White background
    pub const BG_SECONDARY: Color = Color::Rgb(250, 250, 250); // Very light gray
    pub const FG: Color = Color::Rgb(30, 30, 30);           // Near black text
    pub const FG_DIM: Color = Color::Rgb(70, 70, 70);       // Dimmed but readable
    pub const FG_MUTED: Color = Color::Rgb(100, 100, 100);  // Muted but visible

    // Accent colors - vivid for light background
    pub const PRIMARY: Color = Color::Rgb(0, 90, 180);      // Strong blue
    pub const SECONDARY: Color = Color::Rgb(20, 140, 50);   // Strong green
    pub const ACCENT: Color = Color::Rgb(160, 60, 130);     // Strong magenta

    // Status colors - vivid
    pub const SUCCESS: Color = Color::Rgb(20, 140, 50);     // Green
    pub const WARNING: Color = Color::Rgb(180, 120, 0);     // Orange
    pub const ERROR: Color = Color::Rgb(190, 30, 30);       // Red
    pub const INFO: Color = Color::Rgb(0, 90, 180);         // Blue

    // Priority colors - vivid
    pub const PRIORITY_HIGH: Color = Color::Rgb(190, 30, 30);    // Red
    pub const PRIORITY_MEDIUM: Color = Color::Rgb(180, 120, 0);  // Orange
    pub const PRIORITY_LOW: Color = Color::Rgb(20, 140, 50);     // Green
    pub const PRIORITY_NONE: Color = Color::Rgb(100, 100, 100);  // Gray

    // Task kind colors - vivid and distinct
    pub const KIND_TASK: Color = Color::Rgb(0, 90, 180);      // Blue
    pub const KIND_BUG: Color = Color::Rgb(190, 30, 30);      // Red
    pub const KIND_FEATURE: Color = Color::Rgb(20, 140, 50);  // Green
    pub const KIND_CHORE: Color = Color::Rgb(180, 120, 0);    // Orange

    // Border colors - more visible
    pub const BORDER: Color = Color::Rgb(180, 180, 180);
    pub const BORDER_FOCUSED: Color = Color::Rgb(0, 90, 180);

    // Selection - strong blue highlight
    pub const SELECTION_BG: Color = Color::Rgb(0, 90, 180);
    pub const SELECTION_FG: Color = Color::Rgb(255, 255, 255);

    // Styles
    pub fn default_style() -> Style {
        Style::default().fg(Self::FG).bg(Self::BG)
    }

    pub fn dimmed_style() -> Style {
        Style::default().fg(Self::FG_DIM).bg(Self::BG)
    }

    pub fn muted_style() -> Style {
        Style::default().fg(Self::FG_MUTED).bg(Self::BG)
    }

    pub fn selected_style() -> Style {
        Style::default()
            .fg(Self::SELECTION_FG)
            .bg(Self::SELECTION_BG)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlighted_style() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style() -> Style {
        Style::default()
            .fg(Self::FG)
            .add_modifier(Modifier::BOLD)
    }

    pub fn border_style(focused: bool) -> Style {
        if focused {
            Style::default().fg(Self::BORDER_FOCUSED)
        } else {
            Style::default().fg(Self::BORDER)
        }
    }

    pub fn priority_color(priority: &crate::models::TaskPriority) -> Color {
        use crate::models::TaskPriority;
        match priority {
            TaskPriority::High => Self::PRIORITY_HIGH,
            TaskPriority::Medium => Self::PRIORITY_MEDIUM,
            TaskPriority::Low => Self::PRIORITY_LOW,
            TaskPriority::None => Self::PRIORITY_NONE,
        }
    }

    pub fn kind_color(kind: &crate::models::TaskKind) -> Color {
        use crate::models::TaskKind;
        match kind {
            TaskKind::Task => Self::KIND_TASK,
            TaskKind::Bug => Self::KIND_BUG,
            TaskKind::Feature => Self::KIND_FEATURE,
            TaskKind::Chore => Self::KIND_CHORE,
            TaskKind::GhIssue => Self::KIND_BUG,     // Red - like bugs
            TaskKind::GhPr => Self::KIND_FEATURE,    // Green - like features
            TaskKind::GhReview => Self::KIND_CHORE,  // Orange - like chores
        }
    }

    pub fn status_style(completed: bool, overdue: bool) -> Style {
        if completed {
            Style::default()
                .fg(Self::FG_DIM)
                .add_modifier(Modifier::CROSSED_OUT)
        } else if overdue {
            Style::default().fg(Self::ERROR)
        } else {
            Style::default().fg(Self::FG)
        }
    }
}

/// Sidebar navigation items with their shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarItem {
    Inbox,
    Today,
    Upcoming,
    Anytime,
    Completed,
    Review,
    GitHub,
    Toggl,
    Settings,
}

impl SidebarItem {
    pub fn all() -> &'static [SidebarItem] {
        &[
            SidebarItem::Inbox,
            SidebarItem::Today,
            SidebarItem::Upcoming,
            SidebarItem::Anytime,
            SidebarItem::Completed,
            SidebarItem::Review,
            SidebarItem::GitHub,
            SidebarItem::Toggl,
            SidebarItem::Settings,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SidebarItem::Inbox => "Inbox",
            SidebarItem::Today => "Today",
            SidebarItem::Upcoming => "Upcoming",
            SidebarItem::Anytime => "Anytime",
            SidebarItem::Completed => "Completed",
            SidebarItem::Review => "Review",
            SidebarItem::GitHub => "GitHub",
            SidebarItem::Toggl => "Toggl",
            SidebarItem::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SidebarItem::Inbox => "󰇯",
            SidebarItem::Today => "󰃭",
            SidebarItem::Upcoming => "󰃮",
            SidebarItem::Anytime => "󰔚",
            SidebarItem::Completed => "󰄲",
            SidebarItem::Review => "󰑓",
            SidebarItem::GitHub => "󰊤",
            SidebarItem::Toggl => "󱎫",
            SidebarItem::Settings => "󰒓",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            SidebarItem::Inbox => "1",
            SidebarItem::Today => "2",
            SidebarItem::Upcoming => "3",
            SidebarItem::Anytime => "4",
            SidebarItem::Completed => "5",
            SidebarItem::Review => "6",
            SidebarItem::GitHub => "7",
            SidebarItem::Toggl => "8",
            SidebarItem::Settings => "9",
        }
    }

    pub fn from_index(index: usize) -> Option<SidebarItem> {
        Self::all().get(index).copied()
    }
}
