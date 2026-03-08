/// Word-wrap a line to fit within the given width.
/// Breaks on word boundaries, keeping whole words together.
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

    for word in line.split_whitespace() {
        let word_len = word.chars().count();

        if word_len > max_width {
            if !current_line.is_empty() {
                result.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            let mut remaining = word;
            while remaining.chars().count() > max_width {
                let (chunk, rest) = split_at_char_width(remaining, max_width);
                if chunk.is_empty() {
                    break;
                }
                result.push(chunk.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
                current_width = remaining.chars().count();
            }
            continue;
        }

        let space_needed = if current_line.is_empty() { 0 } else { 1 };
        if current_width + space_needed + word_len > max_width {
            result.push(current_line.clone());
            current_line = word.to_string();
            current_width = word_len;
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
                current_width += 1;
            }
            current_line.push_str(word);
            current_width += word_len;
        }
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

/// Calculate cursor position after word wrapping.
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

    let mut original_pos = 0;
    let mut row = 0;

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line_len = 0;

    for (word_idx, word) in words.iter().enumerate() {
        let word_len = word.chars().count();

        if word_len <= max_width
            && original_pos <= cursor_position
            && cursor_position <= original_pos + word.len()
        {
            let col = current_line_len + char_count(&word[..cursor_position - original_pos]);
            return (row, col);
        }

        let space_needed = if current_line_len == 0 { 0 } else { 1 };

        if word_len > max_width {
            let mut word_start = original_pos;
            let mut remaining = *word;

            while !remaining.is_empty() {
                let available_width = if current_line_len >= max_width {
                    max_width
                } else {
                    max_width - current_line_len
                };
                let chunk_width = available_width.max(1).min(remaining.chars().count());
                let (chunk, rest) = split_at_char_width(remaining, chunk_width);
                let chunk_end = word_start + chunk.len();

                if word_start <= cursor_position && cursor_position <= chunk_end {
                    let col = current_line_len + char_count(&chunk[..cursor_position - word_start]);
                    return (row, col);
                }

                word_start = chunk_end;
                remaining = rest;

                if !remaining.is_empty() {
                    row += 1;
                    current_line_len = 0;
                } else {
                    current_line_len += char_count(chunk);
                }
            }
        } else if current_line_len + space_needed + word_len > max_width {
            row += 1;
            current_line_len = word_len;
        } else {
            if current_line_len > 0 {
                current_line_len += 1;
            }
            current_line_len += word_len;
        }

        original_pos += word_len;
        if word_idx < words.len() - 1 {
            original_pos += 1;
        }
    }

    (row, current_line_len)
}

fn split_at_char_width(s: &str, width: usize) -> (&str, &str) {
    if width == 0 {
        return ("", s);
    }

    let mut split_idx = s.len();
    let mut seen = 0;

    for (idx, ch) in s.char_indices() {
        seen += 1;
        split_idx = idx + ch.len_utf8();
        if seen == width {
            break;
        }
    }

    s.split_at(split_idx)
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::{calculate_wrapped_cursor_position, wrap_line};

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
}
