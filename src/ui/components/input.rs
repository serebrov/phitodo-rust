use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;

pub struct InputState {
    pub value: String,
    pub cursor: usize,
    pub prompt: String,
    pub placeholder: String,
}

impl InputState {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            prompt: prompt.into(),
            placeholder: String::new(),
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor = self.value.len();
        self
    }

    pub fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
}

pub fn render_input(frame: &mut Frame, area: Rect, state: &InputState) {
    let display_value = if state.value.is_empty() {
        Span::styled(&state.placeholder, Theme::muted_style())
    } else {
        // Show cursor
        let before = &state.value[..state.cursor];
        let cursor_char = state.value.chars().nth(state.cursor).unwrap_or(' ');
        let after = if state.cursor < state.value.len() {
            &state.value[state.cursor + 1..]
        } else {
            ""
        };

        Span::raw(format!("{}{}{}", before, cursor_char, after))
    };

    let content = Line::from(vec![
        Span::styled(&state.prompt, Style::default().fg(Theme::PRIMARY)),
        Span::raw(" "),
        display_value,
    ]);

    let para = Paragraph::new(content);
    frame.render_widget(para, area);
}

pub fn render_input_modal(frame: &mut Frame, area: Rect, state: &InputState, title: &str) {
    // Center the modal
    let width = area.width.min(50);
    let height = 5;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let modal_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(Theme::title_style())
        .borders(Borders::ALL)
        .border_style(Theme::border_style(true))
        .style(Style::default().bg(Theme::BG_SECONDARY));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Input field
    let display = if state.value.is_empty() {
        vec![Span::styled(&state.placeholder, Theme::muted_style())]
    } else {
        let before = &state.value[..state.cursor];
        let cursor_char = state.value.chars().nth(state.cursor);
        let after = if state.cursor < state.value.len() {
            &state.value[state.cursor + 1..]
        } else {
            ""
        };

        let mut spans = vec![Span::raw(before.to_string())];

        if let Some(c) = cursor_char {
            spans.push(Span::styled(
                c.to_string(),
                Style::default()
                    .fg(Theme::BG)
                    .bg(Theme::FG)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                " ",
                Style::default().fg(Theme::BG).bg(Theme::FG),
            ));
        }

        spans.push(Span::raw(after.to_string()));
        spans
    };

    let content = Paragraph::new(Line::from(display));
    frame.render_widget(content, inner);
}

pub fn render_search_bar(frame: &mut Frame, area: Rect, state: &InputState, focused: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Theme::border_style(focused));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let icon = "";
    let display = if state.value.is_empty() {
        vec![
            Span::styled(icon, Style::default().fg(Theme::FG_DIM)),
            Span::raw(" "),
            Span::styled(&state.placeholder, Theme::muted_style()),
        ]
    } else {
        vec![
            Span::styled(icon, Style::default().fg(Theme::PRIMARY)),
            Span::raw(" "),
            Span::raw(state.value.clone()),
        ]
    };

    let content = Paragraph::new(Line::from(display));
    frame.render_widget(content, inner);
}
