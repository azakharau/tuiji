use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Paragraph, Wrap},
};

use crate::ui::{components::logo::AsciiLogoComponent, context::RenderContext};

use super::state::{HomeState, HomeVariant};

pub struct HomeView;

impl HomeView {
    pub fn draw(
        frame: &mut Frame,
        state: &HomeState,
        logo: &AsciiLogoComponent,
        context: &RenderContext,
    ) {
        let base_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().background);
        frame.render_widget(Block::default().style(base_style), frame.area());

        let highlight_style = Style::default()
            .fg(context.colors().text)
            .bg(context.colors().selection)
            .add_modifier(Modifier::BOLD);
        let logo_style = Style::default().fg(context.colors().logo);

        match state.variant() {
            HomeVariant::Welcome => {
                let screen_layout = Layout::vertical([
                    Constraint::Length(10),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                    Constraint::Length(state.menu().height()),
                    Constraint::Fill(1),
                ])
                .split(frame.area());

                frame.render_widget(logo.clone().with_style(logo_style), screen_layout[1]);

                let content = Paragraph::new(Text::from(state.welcome_text()))
                    .style(base_style)
                    .alignment(Alignment::Center)
                    .wrap(Wrap::default());
                frame.render_widget(content, screen_layout[2]);

                render_menu(
                    state.menu(),
                    screen_layout[3],
                    frame.buffer_mut(),
                    base_style,
                    highlight_style,
                );
            }
            HomeVariant::Default => {
                let screen_layout = Layout::vertical([
                    Constraint::Length(10),
                    Constraint::Fill(1),
                    Constraint::Length(state.menu().height()),
                    Constraint::Fill(1),
                ])
                .split(frame.area());

                frame.render_widget(logo.clone().with_style(logo_style), screen_layout[1]);
                render_menu(
                    state.menu(),
                    screen_layout[2],
                    frame.buffer_mut(),
                    base_style,
                    highlight_style,
                );
            }
        }
    }
}

fn render_menu(
    menu: &crate::ui::components::menu::Menu,
    area: Rect,
    buf: &mut Buffer,
    style: Style,
    highlight_style: Style,
) {
    menu.render_with_style(area, buf, style, highlight_style);
}
