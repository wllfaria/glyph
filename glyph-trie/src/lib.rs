#[derive(Debug, Default)]
struct TrieNode {
    value: char,
    data: Option<usize>,
    children: Vec<TrieNode>,
}

#[derive(Debug)]
pub struct QueryResult<'trie, T> {
    pub continues: bool,
    pub data: &'trie T,
}

#[derive(Debug)]
pub struct Trie<T> {
    values: Vec<T>,
    root: TrieNode,
}

impl<T> Default for Trie<T> {
    fn default() -> Trie<T> {
        Trie {
            values: Default::default(),
            root: Default::default(),
        }
    }
}

impl<T> Trie<T> {
    pub fn new() -> Trie<T> {
        Trie::<T>::default()
    }

    pub fn add_word<S>(&mut self, word: S, data: T)
    where
        S: AsRef<str>,
    {
        if word.as_ref().is_empty() {
            return;
        }

        let idx = self.values.len();
        self.values.push(data);

        self.root.add_word(word, idx);
    }

    pub fn find_word<S>(&self, word: S) -> Option<QueryResult<T>>
    where
        S: AsRef<str>,
    {
        if word.as_ref().is_empty() {
            return None;
        }

        match self.root.find_word(word) {
            Some((idx, continues)) => {
                let data = self.values.get(idx).unwrap();
                Some(QueryResult { continues, data })
            }
            None => None,
        }
    }
}

impl TrieNode {
    pub fn add_word<S>(&mut self, word: S, idx: usize)
    where
        S: AsRef<str>,
    {
        if word.as_ref().is_empty() {
            return;
        }

        let end = word.as_ref().len() == 1;
        let value = word.as_ref().chars().nth(0).unwrap();
        let rest = &word.as_ref()[1..];
        let data = if end { Some(idx) } else { None };

        if !self.children.iter().any(|node| node.value == value) {
            self.children.push(TrieNode {
                value,
                data,
                children: Default::default(),
            });
        }

        self.children
            .iter_mut()
            .find(|node| node.value == value)
            .unwrap()
            .add_word(rest, idx);
    }

    pub fn find_word<S>(&self, word: S) -> Option<(usize, bool)>
    where
        S: AsRef<str>,
    {
        if word.as_ref().is_empty() {
            return None;
        }

        let end = word.as_ref().len() == 1;
        let value = word.as_ref().chars().nth(0).unwrap();
        let rest = &word.as_ref()[1..];

        let existing = self.children.iter().find(|node| node.value == value)?;

        match end {
            true => Some((existing.data.unwrap(), !existing.children.is_empty())),
            false => existing.find_word(rest),
        }
    }
}
