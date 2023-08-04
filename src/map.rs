use crate::{entry::Entry, Tree};
use std::marker::PhantomData;

struct Map<'a> {
    tree: Tree,

    marker: PhantomData<&'a [u8]>,
}

impl<'a> Map<'a> {
    #[inline(always)]
    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: &'a [u8]) -> Option<&'a [u8]> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.get()),
        }
    }

    #[inline(always)]
    /// Returns the key-value pair corresponding to the supplied key.
    pub fn get_key_value(&self, key: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some((entry.key(), entry.get())),
        }
    }

    #[inline(always)]
    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &'a [u8]) -> bool {
        match self.tree.entry(key) {
            Entry::Vacant(_) => false,
            Entry::Occupied(_) => true,
        }
    }

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

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &[u8]) -> Option<impl AsRef<[u8]>> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.remove()),
        }
    }

    /// Removes a key from the map, returning the stored key and value if the key was previously in the map.
    pub fn remove_entry(&mut self, key: &[u8]) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        match self.tree.entry(key) {
            Entry::Vacant(_) => None,
            Entry::Occupied(entry) => Some(entry.remove_entry()),
        }
    }

    pub fn range<'r, R>(&self, range: R) -> impl Iterator + 'r
    where
        R: std::ops::RangeBounds<&'r [u8]>,
    {
        self.tree.range(range)
    }

    pub fn entry<'e>(&self, key: &'e [u8]) -> Entry<'e> {
        self.tree.entry(key)
    }
}
