use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::theme::Theme;

#[derive(Debug, Clone)]
pub struct ConfirmModal {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub selected: bool, // true = confirm, false = cancel
}

impl ConfirmModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_text: "Confirm".to_string(),
            cancel_text: "Cancel".to_string(),
            selected: false,
        }
    }

    pub fn delete(item_name: impl Into<String>) -> Self {
        let name = item_name.into();
        Self {
            title: "Delete".to_string(),
            message: format!("Are you sure you want to delete \"{}\"?", name),
            confirm_text: "Delete".to_string(),
            cancel_text: "Cancel".to_string(),
            selected: false,
        }
    }

    pub fn toggle(&mut self) {
        self.selected = !self.selected;
    }
}

pub fn render_confirm_modal(frame: &mut Frame, area: Rect, modal: &ConfirmModal) {
    // Center the modal
    let width = area.width.min(50);
    let height = 8;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(" {} ", modal.title))
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(true))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let chunks = Layout::vertical([
        Constraint::Min(3),    // Message
        Constraint::Length(2), // Buttons
    ])
    .split(inner);

    // Message
    let message = Paragraph::new(&*modal.message)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Theme::FG));
    frame.render_widget(message, chunks[0]);

    // Buttons
    let confirm_style = if modal.selected {
        Style::default()
            .fg(Theme::BG)
            .bg(Theme::ERROR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::FG_DIM)
    };

    let cancel_style = if !modal.selected {
        Style::default()
            .fg(Theme::BG)
            .bg(Theme::PRIMARY)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::FG_DIM)
    };

    let buttons = Line::from(vec![
        Span::raw("  "),
        Span::styled(format!(" {} ", modal.cancel_text), cancel_style),
        Span::raw("   "),
        Span::styled(format!(" {} ", modal.confirm_text), confirm_style),
    ]);

    let buttons_para = Paragraph::new(buttons);
    frame.render_widget(buttons_para, chunks[1]);
}

#[derive(Debug, Clone)]
pub struct NotificationModal {
    pub message: String,
    pub is_error: bool,
}

impl NotificationModal {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: false,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: true,
        }
    }
}

pub fn render_notification(frame: &mut Frame, area: Rect, notification: &NotificationModal) {
    // Bottom of screen
    let width = area.width.min(60);
    let height = 3;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + area.height - height - 1;
    let notif_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, notif_area);

    let (border_color, icon) = if notification.is_error {
        (Theme::ERROR, "")
    } else {
        (Theme::SUCCESS, "")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(notif_area);
    frame.render_widget(block, notif_area);

    let content = Paragraph::new(Line::from(vec![
        Span::styled(icon, Style::default().fg(border_color)),
        Span::raw(" "),
        Span::styled(&notification.message, Style::default().fg(Theme::FG)),
    ]));
    frame.render_widget(content, inner);
}
