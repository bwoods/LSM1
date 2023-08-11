use crate::{entry::*, range::*, Error, Tree};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

pub struct Map<'a> {
    tree: Tree,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Map<'a> {
    pub fn new(path: &str) -> Result<Self, Error> {
        Ok(Map {
            tree: Tree::new(path)?,
            marker: Default::default(),
        })
    }

    #[inline]
    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &'a [u8]) -> Option<&'a [u8]> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.get()),
        }
    }

    #[inline]
    /// Returns the key-value pair corresponding to the supplied key.
    pub fn get_key_value(&self, key: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some((entry.key(), entry.get())),
        }
    }

    #[inline(always)]
    /// Returns the first key-value pair in the map. The key in this pair is the minimum key in the map.
    pub fn first_key_value(&self) -> Option<(&'a [u8], &'a [u8])> {
        self.iter().next()
    }

    #[inline(always)]
    /// Returns the first entry in the map for in-place manipulation. The key of this entry is the minimum key in the map.
    pub fn first_entry(&mut self) -> Option<OccupiedEntry<'a>> {
        let mut bound =
            Bound::new_in(self.tree.db, std::ops::Bound::Unbounded, Direction::Next).ok()?;
        bound.cursor().ok()?; // ensure we are not empty
        Some(OccupiedEntry(RefCell::new(bound)))
    }

    #[inline(always)]
    /// Removes and returns the first element in the map. The key of this element is the minimum key that was in the map.
    pub fn pop_first(&mut self) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        Some(self.first_entry()?.remove_entry())
    }

    #[inline(always)]
    /// Returns the last key-value pair in the map. The key in this pair is the maximum key in the map.
    pub fn last_key_value(&self) -> Option<(&'a [u8], &'a [u8])> {
        self.iter().next_back()
    }

    #[inline(always)]
    /// Returns the last entry in the map for in-place manipulation. The key of this entry is the maximum key in the map.
    pub fn last_entry(&mut self) -> Option<OccupiedEntry<'a>> {
        let mut bound =
            Bound::new_in(self.tree.db, std::ops::Bound::Unbounded, Direction::Prev).ok()?;
        bound.cursor().ok()?; // ensure we are not empty
        Some(OccupiedEntry(RefCell::new(bound)))
    }

    #[inline(always)]
    /// Removes and returns the last element in the map. The key of this element is the maximum key that was in the map.
    pub fn pop_last(&mut self) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        Some(self.last_entry()?.remove_entry())
    }

    #[inline(always)]
    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &'a [u8]) -> bool {
        match self.tree.entry(key) {
            Entry::Vacant(_) => false,
            Entry::Occupied(_) => true,
        }
    }

    #[inline]
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Option<impl AsRef<[u8]>> {
        match self.tree.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
            Entry::Occupied(entry) => Some(entry.insert(value)),
        }
    }

    #[inline]
    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &[u8]) -> Option<impl AsRef<[u8]>> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.remove()),
        }
    }

    #[inline]
    /// Removes a key from the map, returning the stored key and value if the key was previously in the map.
    pub fn remove_entry(&mut self, key: &[u8]) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.remove_entry()),
        }
    }

    #[inline]
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs (k, v) for which f(&k, &mut v) returns false. The elements are visited in ascending key order.
    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(&mut std::borrow::Cow<'_, [u8]>) -> bool,
    {
        for (key, val) in self.iter() {
            let mut value = std::borrow::Cow::Borrowed(val);
            let keep = pred(&mut value);

            match (keep, value) {
                (true, std::borrow::Cow::Borrowed(_)) => {}
                (true, std::borrow::Cow::Owned(changes)) => {
                    self.insert(key, changes.as_ref());
                }
                (false, _) => {
                    self.remove(key);
                }
            }
        }
    }

    #[inline]
    /// Moves all elements from other into self, leaving other empty.
    ///
    /// If a key from other is already present in self, the respective value from self will be overwritten with the respective value from other.
    pub fn append<K, V>(&mut self, other: &mut BTreeMap<K, V>)
    where
        K: AsRef<[u8]> + Ord,
        V: AsRef<[u8]>,
    {
        other.retain(|key, val| {
            self.insert(key.as_ref(), val.as_ref());
            false
        })
    }

    #[inline(always)]
    /// Constructs a double-ended iterator over a sub-range of elements in the map. The simplest way is to use the range syntax `min..max`, thus `range(min..max)` will yield elements from min (inclusive) to max (exclusive). The range may also be entered as `(Bound<T>, Bound<T>)`, so for example `range((Excluded(4), Included(10)))` will yield a left-exclusive, right-inclusive range from 4 to 10.
    pub fn range<'r, R: std::ops::RangeBounds<&'r [u8]>>(&self, range: R) -> RangeBounds<'r> {
        self.tree.range(range)
    }

    #[inline(always)]
    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    pub fn entry<'e>(&self, key: &'e [u8]) -> Entry<'e> {
        self.tree.entry(key)
    }

    #[inline]
    /// Splits the collection into two at the given key. Returns everything after the given key, including the key.
    pub fn split_off(&mut self, key: &[u8]) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut other = BTreeMap::new();
        for (key, val) in self.range(key..) {
            other.insert(key.into(), val.into());
            self.remove(key);
        }

        other
    }

    #[inline(always)]
    /// Creates a consuming iterator visiting all the keys, in sorted order. The map cannot be used after calling this.
    pub fn into_keys(self) -> impl Iterator<Item = &'a [u8]> {
        self.keys()
    }

    #[inline(always)]
    /// Creates a consuming iterator visiting all the values, in order by key. The map cannot be used after calling this.
    pub fn into_values(self) -> impl Iterator<Item = &'a [u8]> {
        self.values()
    }

    #[inline(always)]
    /// Gets an iterator over the entries of the map, sorted by key.
    pub fn iter(&self) -> Iter<'a> {
        Iter {
            range: self.tree.range(..),
        }
    }

    #[inline(always)]
    /// Gets an iterator over the keys of the map, in sorted order.
    pub fn keys(&self) -> impl Iterator<Item = &'a [u8]> {
        self.iter().map(|(key, _)| key)
    }

    #[inline(always)]
    /// Gets an iterator over the values of the map, in order by key.
    pub fn values(&self) -> impl Iterator<Item = &'a [u8]> {
        self.iter().map(|(_, value)| value)
    }

    #[inline(always)]
    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.range(..).is_empty()
    }
}

/// An iterator over the entries of a `Map`.
pub struct Iter<'e> {
    range: RangeBounds<'e>,
}

impl<'e> Iterator for Iter<'e> {
    type Item = (&'e [u8], &'e [u8]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next()
    }
}

impl<'e> DoubleEndedIterator for Iter<'e> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back()
    }
}
