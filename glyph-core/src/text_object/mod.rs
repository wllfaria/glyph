use ropey::iter::Lines;
use ropey::{Rope, RopeSlice};

use crate::geometry::Point;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextObject {
    inner: Rope,
}

const OPENING_PAIRS: &[char] = &['(', '{', '[', '<'];
const CLOSING_PAIRS: &[char] = &[')', '}', ']', '>'];

fn is_pairable_character(char: &char) -> bool {
    OPENING_PAIRS.contains(char) || CLOSING_PAIRS.contains(char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn is_punctuation_char(ch: char) -> bool {
    !ch.is_whitespace() && !is_word_char(ch)
}

fn is_matching_pair(needle: char, char: char) -> bool {
    matches!(
        (needle, char),
        ('(', ')')
            | ('{', '}')
            | ('[', ']')
            | ('<', '>')
            | (')', '(')
            | ('}', '{')
            | (']', '[')
            | ('>', '<')
    )
}

fn find_pair_search_direction(char: char) -> SearchDirection {
    if OPENING_PAIRS.contains(&char) {
        return SearchDirection::Forward;
    }

    SearchDirection::Backward
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SearchDirection {
    Forward,
    Backward,
}

impl TextObject {
    pub fn new(content: String) -> Self {
        Self {
            inner: Rope::from(content),
        }
    }

    pub fn get_line(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        self.inner.get_line(line_idx)
    }

    pub fn line(&self, line_idx: usize) -> RopeSlice<'_> {
        assert!(line_idx < self.len_lines());
        self.inner.line(line_idx)
    }

    pub fn line_len(&self, line_idx: usize) -> usize {
        self.line(line_idx).len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.inner.len_lines()
    }

    pub fn lines(&self) -> Lines<'_> {
        self.inner.lines()
    }

    pub fn delete_whole_line(&mut self, line: usize) {
        let len_lines = self.len_lines();
        assert!(line < len_lines);

        let line_start = self.inner.line_to_char(line);
        let line_end = self.inner.line_to_char(line + 1);
        self.inner.remove(line_start.saturating_sub(1)..line_end);
    }

    pub fn find_matching_pair(&self, point: Point<usize>) -> Point<usize> {
        let line = self.line(point.y);
        assert!(point.x < line.len_chars());

        let Some((start_offset, pair_to_match)) = line
            .chars_at(point.x)
            .enumerate()
            .find(|(_, ch)| is_pairable_character(ch))
        else {
            return point;
        };

        let search_direction = find_pair_search_direction(pair_to_match);
        let search_start_char = self.inner.line_to_char(point.y) + point.x + start_offset;
        let mut open_pairs = 0;

        match search_direction {
            SearchDirection::Forward => {
                for (idx, ch) in self.inner.chars_at(search_start_char).enumerate() {
                    if ch == pair_to_match {
                        open_pairs += 1;
                    }

                    if is_matching_pair(pair_to_match, ch) {
                        open_pairs -= 1;
                    }

                    if open_pairs == 0 {
                        let pair_char_idx = search_start_char + idx;
                        let pair_line_idx = self.inner.char_to_line(pair_char_idx);
                        let pair_line_start_char = self.inner.line_to_char(pair_line_idx);
                        let pair_line_col = pair_char_idx - pair_line_start_char;
                        return Point::new(pair_line_col, pair_line_idx);
                    }
                }
            }
            SearchDirection::Backward => {
                for idx in (0..=search_start_char).rev() {
                    let ch = self.inner.char(idx);

                    if ch == pair_to_match {
                        open_pairs += 1;
                    }

                    if is_matching_pair(pair_to_match, ch) {
                        open_pairs -= 1;
                    }

                    if open_pairs == 0 {
                        let pair_line_idx = self.inner.char_to_line(idx);
                        let pair_line_start_char = self.inner.line_to_char(pair_line_idx);
                        let pair_line_col = idx - pair_line_start_char;
                        return Point::new(pair_line_col, pair_line_idx);
                    }
                }
            }
        }

        point
    }

    pub fn find_first_non_space_character(&self, line_idx: usize) -> Point<usize> {
        let line = self.line(line_idx);
        let first_non_space_char_idx = line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0);
        Point::new(first_non_space_char_idx, line_idx)
    }

    pub fn find_last_non_space_character(&self, line_idx: usize) -> Point<usize> {
        let line = self.line(line_idx);

        let last_non_space_char_idx = (0..line.len_chars())
            .rev()
            .find(|&idx| !line.char(idx).is_whitespace())
            .unwrap_or(0);

        Point::new(last_non_space_char_idx, line_idx)
    }

    pub fn find_next_paragraph(&self, line_idx: usize) -> Point<usize> {
        let is_on_last_line = line_idx == self.len_lines().saturating_sub(1);
        if is_on_last_line {
            let line_len = self.line(line_idx).len_chars().saturating_sub(1);
            return Point::new(line_len, line_idx);
        }

        let line_idx = self
            .inner
            .lines_at(line_idx + 1)
            .enumerate()
            .find(|(_, line)| line.len_chars() == 1 && line.char(0) == '\n')
            .map(|(idx, _)| line_idx + idx + 1)
            .unwrap_or(self.len_lines().saturating_sub(1));

        Point::new(0, line_idx)
    }

    pub fn find_prev_paragraph(&self, line_idx: usize) -> Point<usize> {
        let line_idx = (0..line_idx)
            .rev()
            .find(|&idx| self.line(idx).len_chars() == 1 && self.line(idx).char(0) == '\n')
            .unwrap_or(0);

        Point::new(0, line_idx)
    }

    pub fn delete_prev_char(&mut self, position: Point<usize>) {
        let line_start_char = self.inner.line_to_char(position.y);
        let position_char = line_start_char + position.x;
        self.inner.remove(position_char - 1..position_char);
    }

    pub fn delete_curr_char(&mut self, position: Point<usize>) {
        let line_start_char = self.inner.line_to_char(position.y);
        let position_char = line_start_char + position.x;
        self.inner.remove(position_char..position_char + 1);
    }

    pub fn insert_char_at(&mut self, position: Point<usize>, ch: char) {
        let line_start_char = self.inner.line_to_char(position.y);
        let position_char = line_start_char + position.x;
        self.inner.insert_char(position_char, ch);
    }

    pub fn find_next_word_boundary(&self, point: Point<usize>) -> Point<usize> {
        if point.y >= self.len_lines() {
            return point;
        }

        let line = self.line(point.y);
        if point.x >= line.len_chars().saturating_sub(2) {
            return self.find_next_line_start(point.y + 1);
        }

        let current_char = line.char(point.x);
        let line_start_char = self.inner.line_to_char(point.y);
        let mut char_idx = line_start_char + point.x;

        // Step 1: Skip current word/punctuation group
        if is_word_char(current_char) {
            // Skip all word characters
            for ch in self.inner.chars_at(char_idx) {
                if !is_word_char(ch) {
                    break;
                }
                char_idx += 1;
            }
        } else if is_punctuation_char(current_char) {
            // Skip all punctuation characters
            for ch in self.inner.chars_at(char_idx) {
                if !is_punctuation_char(ch) {
                    break;
                }
                char_idx += 1;
            }
        }

        // Step 2: Skip whitespace
        for ch in self.inner.chars_at(char_idx) {
            if !ch.is_whitespace() {
                break;
            }
            char_idx += 1;
        }

        // Convert back to line/column coordinates
        if char_idx >= self.inner.len_chars() {
            // At EOF
            let last_line = self.len_lines().saturating_sub(1);
            let last_line_len = self.line_len(last_line).saturating_sub(1);
            return Point::new(last_line_len, last_line);
        }

        let result_line = self.inner.char_to_line(char_idx);
        let result_line_start = self.inner.line_to_char(result_line);
        let result_col = char_idx - result_line_start;

        Point::new(result_col, result_line)
    }

    fn find_next_line_start(&self, line_idx: usize) -> Point<usize> {
        if line_idx >= self.len_lines() {
            // At EOF
            let last_line = self.len_lines().saturating_sub(1);
            let last_line_len = self.line_len(last_line).saturating_sub(1);
            return Point::new(last_line_len, last_line);
        }

        self.line(line_idx)
            .chars()
            .enumerate()
            .find(|(_, ch)| !ch.is_whitespace()) // find first non-whitespace character
            .map(|(col, _)| Point::new(col, line_idx)) // that's where the cursor should move
            .unwrap_or(Point::new(0, line_idx)) // if only whitespace, then return start of line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pair() {
        let code_sample = [
            "let (a, (b, c, d)) = match foo {",
            "    Ok(result) => result",
            "    Err(e) => unreachable!()",
            "}",
        ]
        .join("\n");

        let text_object = TextObject::new(code_sample);

        // matching on forward pair
        let pair_position = text_object.find_matching_pair(Point::new(0, 0));
        assert_eq!(pair_position, Point::new(17, 0));

        // matching on forward pair multiline
        let pair_position = text_object.find_matching_pair(Point::new(23, 0));
        assert_eq!(pair_position, Point::new(0, 3));

        // matching on backward pair
        let pair_position = text_object.find_matching_pair(Point::new(12, 0));
        assert_eq!(pair_position, Point::new(8, 0));

        // matching on backward pair multiline
        let pair_position = text_object.find_matching_pair(Point::new(0, 3));
        assert_eq!(pair_position, Point::new(31, 0));
    }
}
