pub(super) fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

pub(super) fn prev_char_boundary(value: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }

    value[..pos]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

pub(super) fn next_char_boundary(value: &str, pos: usize) -> usize {
    if pos >= value.len() {
        return value.len();
    }

    let mut iter = value[pos..].char_indices();
    let _ = iter.next();
    iter.next().map(|(idx, _)| pos + idx).unwrap_or(value.len())
}

pub(super) fn word_forward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }

    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());

    if idx >= chars.len() {
        return len;
    }

    if is_word_char(chars[idx].1) {
        while idx < chars.len() && is_word_char(chars[idx].1) {
            idx += 1;
        }
    }

    while idx < chars.len() && !is_word_char(chars[idx].1) {
        idx += 1;
    }

    if idx < chars.len() { chars[idx].0 } else { len }
}

pub(super) fn word_end(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    let len = value.len();
    if pos >= len {
        return len;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len().saturating_sub(1));

    if idx >= chars.len() {
        return len;
    }

    if !is_word_char(chars[idx].1) {
        while idx < chars.len() && !is_word_char(chars[idx].1) {
            idx += 1;
        }
    }
    if idx >= chars.len() {
        return len;
    }
    while idx + 1 < chars.len() && is_word_char(chars[idx + 1].1) {
        idx += 1;
    }
    chars[idx].0
}

pub(super) fn word_backward(value: &str, pos: usize) -> usize {
    let chars = value.char_indices().collect::<Vec<(usize, char)>>();
    if chars.is_empty() {
        return 0;
    }
    if pos == 0 {
        return 0;
    }
    let mut idx = chars
        .iter()
        .position(|(i, _)| *i >= pos)
        .unwrap_or(chars.len());
    idx = idx.saturating_sub(1);

    if !is_word_char(chars[idx].1) {
        while idx > 0 && !is_word_char(chars[idx].1) {
            idx -= 1;
        }
        if !is_word_char(chars[idx].1) {
            return 0;
        }
    }

    while idx > 0 && is_word_char(chars[idx - 1].1) {
        idx -= 1;
    }
    chars[idx].0
}

/// Convert (row, col) to absolute position in text.
pub(super) fn row_col_to_position(text: &str, row: usize, col: usize) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let mut pos = 0;

    for (i, line) in lines.iter().enumerate() {
        if i == row {
            return pos + col.min(line.len());
        }
        pos += line.len() + 1; // +1 for newline.
    }

    // If row is beyond available lines, return end of text.
    text.len()
}

pub(super) fn line_len_at_row(text: &str, row: usize) -> usize {
    text.lines().nth(row).map(|line| line.len()).unwrap_or(0)
}
