use crate::ui::components::logo::AsciiLogoComponent;

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{self, Constraint, Flex, Layout, Rect},
    widgets::Widget,
};

#[derive(Default, Debug)]
pub struct HomeScreen {
    logo: AsciiLogoComponent,
}

impl Widget for HomeScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.logo.render(area, buf);
    }
}

impl HomeScreen {
    pub fn draw(self, frame: &mut Frame) {
        let screen_layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .flex(Flex::Center)
            .split(frame.area());

        frame.render_widget(self.logo, screen_layout[1]);
    }
}
