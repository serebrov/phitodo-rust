use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::services::TogglData;
use crate::ui::components::{render_toggl_view, TogglChartState};
use crate::ui::theme::Theme;

pub struct TogglView {
    pub chart_state: TogglChartState,
    pub loading: bool,
    pub error: Option<String>,
}

impl TogglView {
    pub fn new() -> Self {
        Self {
            chart_state: TogglChartState::default(),
            loading: false,
            error: None,
        }
    }

    pub fn set_data(&mut self, data: TogglData) {
        self.chart_state.data = data;
        self.loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.loading {
            let loading = Paragraph::new("Loading Toggl data...")
                .style(Theme::dimmed_style());
            frame.render_widget(loading, area);
            return;
        }

        if let Some(ref error) = self.error {
            let error_msg = Paragraph::new(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Theme::ERROR)),
                Span::raw(error.clone()),
            ]));
            frame.render_widget(error_msg, area);
            return;
        }

        render_toggl_view(frame, area, &self.chart_state);
    }
}

impl Default for TogglView {
    fn default() -> Self {
        Self::new()
    }
}
