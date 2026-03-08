use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::ListItem,
};

use crate::ui::context::RenderContext;

use super::super::state::ConflictsState;

pub(super) fn build_list_items(
    state: &ConflictsState,
    context: &RenderContext,
) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(state.issues().len());
    for issue in state.issues() {
        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{} {}", issue.key, issue.summary),
            Style::default().fg(context.colors().text),
        ));
        if issue.conflict {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                "[Issue]",
                Style::default().fg(context.colors().accent),
            ));
        }
        let comment_count = issue.comments.iter().filter(|c| c.conflict).count();
        if comment_count > 0 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[Comments ({comment_count})]"),
                Style::default().fg(context.colors().warning),
            ));
        }
        items.push(ListItem::new(Line::from(spans)));
    }
    items
}
