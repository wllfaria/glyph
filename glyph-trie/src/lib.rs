mod slab;

use slab::Slab;

use crate::slab::SlotKey;

#[derive(Debug)]
pub struct Node<T> {
    children: Vec<SlotKey>,
    key: char,
    value: Option<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            children: vec![],
            key: char::default(),
            value: None,
        }
    }
}

impl<T> Node<T> {
    pub fn new(key: char, value: Option<T>) -> Self {
        Self {
            children: vec![],
            key,
            value,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrieQuery<'a, T> {
    pub value: Option<&'a T>,
    pub continues: bool,
}

impl<'a, T> TrieQuery<'a, T> {
    pub fn new(value: Option<&'a T>, continues: bool) -> Self {
        Self { value, continues }
    }
}

#[derive(Debug)]
pub struct Trie<T> {
    nodes: Slab<Node<T>>,
}

impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Trie<T> {
    pub fn new() -> Self {
        let mut nodes = Slab::<Node<T>>::default();
        nodes.insert(Node::default());
        Self { nodes }
    }

    pub fn insert<S: AsRef<str>>(&mut self, needle: S, value: T) {
        let needle = needle.as_ref();
        if needle.is_empty() {
            return;
        }

        assert!(self.nodes.get(0).is_some());
        let mut current_idx = 0;

        for ch in needle.chars() {
            let mut found_child = None;

            for &idx in &self.nodes[current_idx].children {
                if self.nodes[idx].key == ch {
                    found_child = Some(idx);
                    break;
                }
            }

            match found_child {
                Some(child_idx) => current_idx = child_idx,
                None => {
                    let new_node_idx = self.nodes.insert(Node::new(ch, None));
                    self.nodes[current_idx].children.push(new_node_idx);
                    current_idx = new_node_idx;
                }
            }
        }

        self.nodes[current_idx].value = Some(value);
    }

    pub fn get<S: AsRef<str>>(&self, needle: S) -> Option<TrieQuery<'_, T>> {
        let needle = needle.as_ref();
        if needle.is_empty() {
            return None;
        }

        assert!(self.nodes.get(0).is_some());
        let mut current_idx = 0;

        for ch in needle.chars() {
            let mut found_child = None;

            for &child_idx in &self.nodes[current_idx].children {
                if self.nodes[child_idx].key == ch {
                    found_child = Some(child_idx);
                    break;
                }
            }

            match found_child {
                Some(child_idx) => current_idx = child_idx,
                None => return None,
            }
        }

        let node = &self.nodes[current_idx];
        let continues = !node.children.is_empty();

        let query = TrieQuery {
            value: node.value.as_ref(),
            continues,
        };

        Some(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_and_get() {
        let mut trie = Trie::new();
        trie.insert("foo", 1);
        trie.insert("bar", 2);

        assert_eq!(trie.get("foo").unwrap(), TrieQuery::new(Some(&1), false));
        assert_eq!(trie.get("f").unwrap(), TrieQuery::new(None, true));
        assert_eq!(trie.get("x"), None);
    }

    #[test]
    fn test_overlapping_prefixes() {
        let mut trie = Trie::new();
        trie.insert("cat", 1);
        trie.insert("car", 2);
        trie.insert("card", 3);
        trie.insert("care", 4);

        assert_eq!(trie.get("cat").unwrap(), TrieQuery::new(Some(&1), false));
        assert_eq!(trie.get("car").unwrap(), TrieQuery::new(Some(&2), true));
        assert_eq!(trie.get("card").unwrap(), TrieQuery::new(Some(&3), false));
        assert_eq!(trie.get("care").unwrap(), TrieQuery::new(Some(&4), false));
        assert_eq!(trie.get("ca").unwrap(), TrieQuery::new(None, true));
        assert_eq!(trie.get("cards"), None);
    }

    #[test]
    fn test_empty_string() {
        let mut trie = Trie::new();
        trie.insert("", 42);
        assert_eq!(trie.get(""), None);
    }

    #[test]
    fn test_overwrite_value() {
        let mut trie = Trie::new();
        trie.insert("key", 1);
        trie.insert("key", 2);

        assert_eq!(trie.get("key").unwrap(), TrieQuery::new(Some(&2), false));
    }
}