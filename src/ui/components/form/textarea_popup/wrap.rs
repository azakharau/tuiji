/// Word-wrap a line to fit within the given width.
/// Breaks on word boundaries, keeping whole words together.
pub(super) fn wrap_line(line: &str, max_width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in line.split_whitespace() {
        let word_len = word.len();

        if word_len > max_width {
            if !current_line.is_empty() {
                result.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            let mut remaining = word;
            while remaining.len() > max_width {
                result.push(remaining[..max_width].to_string());
                remaining = &remaining[max_width..];
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
                current_width = remaining.len();
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
    if text.is_empty() {
        return (0, 0);
    }

    let mut original_pos = 0;
    let mut row = 0;

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line_len = 0;

    for (word_idx, word) in words.iter().enumerate() {
        let word_len = word.len();

        if original_pos <= cursor_position && cursor_position <= original_pos + word_len {
            let col = current_line_len + (cursor_position - original_pos);
            return (row, col);
        }

        let space_needed = if current_line_len == 0 { 0 } else { 1 };

        if word_len > max_width {
            let mut remaining_len = word_len;
            let mut word_start = original_pos;

            while remaining_len > 0 {
                let chunk_len = remaining_len.min(max_width - current_line_len);

                if word_start <= cursor_position && cursor_position <= word_start + chunk_len {
                    let col = current_line_len + (cursor_position - word_start);
                    return (row, col);
                }

                word_start += chunk_len;
                remaining_len -= chunk_len;

                if remaining_len > 0 {
                    row += 1;
                    current_line_len = 0;
                } else {
                    current_line_len += chunk_len;
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
