#[derive(Debug, PartialEq)]
pub struct Lines<'a> {
    pub buffer: &'a [char],
    pub start: usize,
    pub end: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Vec<char>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let start = self.start;
        let range = start..self.end;
        let filtered = range.filter(|i| *i + 1 == self.end || self.buffer[*i] == '\n');
        self.start = filtered.take(1).next().unwrap() + 1;

        let line = (start..self.start)
            .map(|i| self.buffer[i])
            .filter(|&c| c != '\0')
            .collect::<Vec<_>>();

        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_correct_lines_without_newline_at_end() {
        let text =
            "The quick brown fox\njumps over the lazy dog\nThe five boxing wizards\njump quickly.";
        let mut lines = Lines {
            buffer: &text.chars().collect::<Vec<_>>(),
            start: 0,
            end: text.len(),
        };

        let first = lines.next().unwrap();
        let second = lines.next().unwrap();
        let third = lines.next().unwrap();
        let fourth = lines.next().unwrap();
        let empty = lines.next();

        assert_eq!(first, "The quick brown fox\n".chars().collect::<Vec<_>>());
        assert_eq!(
            second,
            "jumps over the lazy dog\n".chars().collect::<Vec<_>>()
        );
        assert_eq!(
            third,
            "The five boxing wizards\n".chars().collect::<Vec<_>>()
        );
        assert_eq!(fourth, "jump quickly.".chars().collect::<Vec<_>>());
        assert_eq!(empty, None);
    }

    #[test]
    fn test_return_correct_lines_with_newline_at_end() {
        let text =
            "The quick brown fox\njumps over the lazy dog\nThe five boxing wizards\njump quickly.\n";
        let mut lines = Lines {
            buffer: &text.chars().collect::<Vec<_>>(),
            start: 0,
            end: text.len(),
        };

        let first = lines.next().unwrap();
        let second = lines.next().unwrap();
        let third = lines.next().unwrap();
        let fourth = lines.next().unwrap();
        let empty = lines.next();

        assert_eq!(first, "The quick brown fox\n".chars().collect::<Vec<_>>());
        assert_eq!(
            second,
            "jumps over the lazy dog\n".chars().collect::<Vec<_>>()
        );
        assert_eq!(
            third,
            "The five boxing wizards\n".chars().collect::<Vec<_>>()
        );
        assert_eq!(fourth, "jump quickly.\n".chars().collect::<Vec<_>>());
        assert_eq!(empty, None);
    }

    #[test]
    fn test_return_none_with_empty_lines() {
        let text = "";
        let mut lines = Lines {
            buffer: &text.chars().collect::<Vec<_>>(),
            start: 0,
            end: text.len(),
        };

        let empty = lines.next();

        assert_eq!(empty, None);
    }

    #[test]
    fn test_return_empty_string_with_only_newlines() {
        let text = "\n\n\n\n";
        let mut lines = Lines {
            buffer: &text.chars().collect::<Vec<_>>(),
            start: 0,
            end: text.len(),
        };

        let empty = lines.next().unwrap();

        assert_eq!(empty, "\n".chars().collect::<Vec<_>>());
    }
}
