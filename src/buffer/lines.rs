#[derive(Debug, PartialEq)]
pub struct Lines<'a> {
    pub buffer: &'a Vec<char>,
    pub start: usize,
    pub end: usize,
}

impl<'a> Iterator for Lines<'a> {
    type Item = Vec<char>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        let start = self.start;
        self.start = (start..self.end)
            .filter(|i| *i + 1 == self.end || self.buffer[*i] == '\n')
            .take(1)
            .next()
            .unwrap()
            + 1;

        let line = (start..self.start)
            .map(|i| self.buffer[i])
            .filter(|&c| c != '\0')
            .collect::<Vec<_>>();

        Some(line)
    }
}
