/// Soft-wrap a line to fit within the given width while preserving whitespace exactly.
pub(super) fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }

    if line.is_empty() {
        return vec![String::new()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for ch in line.chars() {
        if current_width == max_width {
            result.push(std::mem::take(&mut current_line));
            current_width = 0;
        }
        current_line.push(ch);
        current_width += 1;
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    if result.is_empty() {
        vec![String::new()]
    } else {
        result
    }
}

/// Calculate cursor position after soft wrapping.
/// Returns (row, col) in wrapped text.
pub(super) fn calculate_wrapped_cursor_position(
    text: &str,
    cursor_position: usize,
    max_width: usize,
) -> (usize, usize) {
    if max_width == 0 {
        return (0, 0);
    }

    if text.is_empty() {
        return (0, 0);
    }

    let mut row = 0;
    let mut col = 0;
    let mut byte_pos = 0;

    for ch in text.chars() {
        if byte_pos >= cursor_position {
            return (row, col);
        }

        if ch == '\n' {
            row += 1;
            col = 0;
            byte_pos += ch.len_utf8();
            continue;
        }

        if col == max_width {
            row += 1;
            col = 0;
        }

        col += 1;
        byte_pos += ch.len_utf8();

        if col == max_width && byte_pos < cursor_position {
            row += 1;
            col = 0;
        }
    }

    (row, col)
}

pub(super) fn calculate_wrapped_textarea_cursor_position(
    text: &str,
    row: usize,
    col: usize,
    max_width: usize,
) -> (usize, usize) {
    let cursor_position = row_col_to_absolute_position(text, row, col);
    calculate_wrapped_cursor_position(text, cursor_position, max_width)
}

fn row_col_to_absolute_position(text: &str, row: usize, col: usize) -> usize {
    let mut absolute = 0;
    let mut lines = text.split('\n').peekable();
    let mut current_row = 0;

    while let Some(line) = lines.next() {
        if current_row == row {
            return absolute + col.min(line.len());
        }

        absolute += line.len();
        if lines.peek().is_some() {
            absolute += 1;
        }
        current_row += 1;
    }

    text.len()
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_wrapped_cursor_position, calculate_wrapped_textarea_cursor_position, wrap_line,
    };

    #[test]
    fn wrap_line_should_handle_zero_width_without_looping() {
        assert_eq!(wrap_line("hello", 0), vec![String::new()]);
    }

    #[test]
    fn wrap_line_should_split_unicode_on_char_boundaries() {
        assert_eq!(
            wrap_line("приветмир", 4),
            vec!["прив".to_string(), "етми".to_string(), "р".to_string()]
        );
    }

    #[test]
    fn wrapped_cursor_position_should_handle_zero_width() {
        assert_eq!(calculate_wrapped_cursor_position("hello", 3, 0), (0, 0));
    }

    #[test]
    fn wrapped_cursor_position_should_progress_when_line_is_full() {
        assert_eq!(calculate_wrapped_cursor_position("abcdefgh", 7, 4), (1, 3));
    }

    #[test]
    fn wrap_line_should_preserve_repeated_spaces() {
        assert_eq!(wrap_line("a  b", 4), vec!["a  b".to_string()]);
        assert_eq!(
            wrap_line("a  bc", 4),
            vec!["a  b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn wrapped_cursor_position_should_preserve_leading_whitespace() {
        assert_eq!(calculate_wrapped_cursor_position("  abc", 2, 4), (0, 2));
    }

    #[test]
    fn wrapped_textarea_cursor_position_should_follow_soft_wrapped_line() {
        assert_eq!(
            calculate_wrapped_textarea_cursor_position("abcdef", 0, 5, 4),
            (1, 1)
        );
    }

    #[test]
    fn wrapped_textarea_cursor_position_should_keep_trailing_empty_line() {
        assert_eq!(
            calculate_wrapped_textarea_cursor_position("abc\n", 1, 0, 4),
            (1, 0)
        );
    }
}
