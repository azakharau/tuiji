use crate::app::{render::RenderStack, state::ScreenType};

use super::is_modal_screen;

pub fn build_render_stack(
    current_screen: ScreenType,
    screen_stack: &[ScreenType],
) -> RenderStack<'_> {
    RenderStack::new(
        current_screen,
        screen_stack,
        is_modal_screen(current_screen),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_render_stack_includes_back_stack() {
        let stack = build_render_stack(
            ScreenType::Settings,
            &[ScreenType::Home, ScreenType::CurrentSprint],
        );

        assert_eq!(
            stack.iter().collect::<Vec<_>>(),
            vec![
                ScreenType::Home,
                ScreenType::CurrentSprint,
                ScreenType::Settings
            ]
        );
    }

    #[test]
    fn non_modal_render_stack_renders_only_current_screen() {
        let stack = build_render_stack(
            ScreenType::CurrentSprint,
            &[ScreenType::Home, ScreenType::Settings],
        );

        assert_eq!(
            stack.iter().collect::<Vec<_>>(),
            vec![ScreenType::CurrentSprint]
        );
    }
}
