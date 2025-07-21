use std::ops::{Index, IndexMut};

pub type SlotKey = usize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlotStatus<T> {
    Occupied(T),
    Empty(Option<SlotKey>),
}

#[derive(Debug)]
pub struct Slab<T> {
    items: Vec<SlotStatus<T>>,
    next_slot: Option<SlotKey>,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_slot: None,
        }
    }
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: T) -> SlotKey {
        let mut item = SlotStatus::Occupied(value);

        match self.next_slot.take() {
            Some(index) => {
                let SlotStatus::Empty(next_slot) = &mut self.items[index] else {
                    unreachable!("next_slot is not an empty slot");
                };

                self.next_slot = next_slot.take();
                std::mem::swap(&mut self.items[index], &mut item);

                index
            }
            None => {
                let index = self.items.len();
                self.items.push(item);
                index
            }
        }
    }

    pub fn get(&self, index: SlotKey) -> Option<&T> {
        match self.items.get(index)? {
            SlotStatus::Occupied(value) => Some(value),
            SlotStatus::Empty(_) => None,
        }
    }

    pub fn get_mut(&mut self, index: SlotKey) -> Option<&mut T> {
        match self.items.get_mut(index)? {
            SlotStatus::Occupied(value) => Some(value),
            SlotStatus::Empty(_) => None,
        }
    }

    pub fn remove(&mut self, index: SlotKey) -> T {
        let mut entry = SlotStatus::Empty(self.next_slot.take());
        self.next_slot = Some(index);
        std::mem::swap(&mut self.items[index], &mut entry);

        match entry {
            SlotStatus::Occupied(value) => value,
            SlotStatus::Empty(_) => unreachable!("attempted to remove an empty slot"),
        }
    }

    pub fn iter_values(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|item| match item {
            SlotStatus::Occupied(value) => Some(value),
            SlotStatus::Empty(_) => None,
        })
    }

    pub fn iter_mut_values(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().filter_map(|item| match item {
            SlotStatus::Occupied(value) => Some(value),
            SlotStatus::Empty(_) => None,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotKey, &T)> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(i, node)| match node {
                SlotStatus::Occupied(value) => Some((i, value)),
                SlotStatus::Empty(_) => None,
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlotKey, &mut T)> {
        self.items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, node)| match node {
                SlotStatus::Occupied(value) => Some((i, value)),
                SlotStatus::Empty(_) => None,
            })
    }
}

impl<T> Index<SlotKey> for Slab<T> {
    type Output = T;

    fn index(&self, index: SlotKey) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            None => panic!("attempted to index into an empty slot"),
        }
    }
}

impl<T> IndexMut<SlotKey> for Slab<T> {
    fn index_mut(&mut self, index: SlotKey) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(value) => value,
            None => panic!("attempted to index into an empty slot"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut slab = Slab::new();
        let a = slab.insert(1);
        let b = slab.insert(2);
        let c = slab.insert(3);

        assert_eq!(slab.get(a), Some(&1));
        assert_eq!(slab.get(b), Some(&2));
        assert_eq!(slab.get(c), Some(&3));
    }

    #[test]
    fn test_remove() {
        let mut slab = Slab::new();
        let a = slab.insert(1);
        let b = slab.insert(2);
        let c = slab.insert(3);

        assert_eq!(slab.get(a), Some(&1));
        assert_eq!(slab.get(b), Some(&2));
        assert_eq!(slab.get(c), Some(&3));

        slab.remove(a);
        assert_eq!(slab.get(a), None);
        assert_eq!(slab.get(b), Some(&2));
        assert_eq!(slab.get(c), Some(&3));
        assert_eq!(slab.next_slot, Some(a));
        assert_eq!(slab.items[a], SlotStatus::Empty(None));

        slab.remove(b);
        assert_eq!(slab.get(a), None);
        assert_eq!(slab.get(b), None);
        assert_eq!(slab.get(c), Some(&3));
        assert_eq!(slab.next_slot, Some(b));
        assert_eq!(slab.items[b], SlotStatus::Empty(Some(a)));

        slab.remove(c);
        assert_eq!(slab.get(a), None);
        assert_eq!(slab.get(b), None);
        assert_eq!(slab.get(c), None);
        assert_eq!(slab.next_slot, Some(c));
        assert_eq!(slab.items[c], SlotStatus::Empty(Some(b)));
    }

    #[test]
    fn test_add_take_empty_slots() {
        let mut slab = Slab::new();
        slab.insert(1);
        let b = slab.insert(2);
        slab.insert(3);

        slab.remove(b);

        let d = slab.insert(4);
        assert_eq!(d, b);
        assert_eq!(slab.next_slot, None);
        assert_eq!(slab.items[b], SlotStatus::Occupied(4));
        assert_eq!(slab.items[d], SlotStatus::Occupied(4));
    }
}
