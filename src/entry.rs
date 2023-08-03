/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the [entry] method.
///
/// [entry]: Map::entry
///
pub enum Entry<'e> {
    Vacant(VacantEntry<'e>),
    Occupied(OccupiedEntry<'e>),
}

impl<'e> Entry<'e> {
    #[inline(always)]
    /// Ensures a value is in the entry by inserting the default if empty, and returns a reference to the value in the entry.
    pub fn or_insert(self, default: &'e [u8]) -> &'e [u8] {
        self.or_insert_with(|| default)
    }

    #[inline(always)]
    /// Ensures a value is in the entry by inserting the result of the default function if empty, and returns   reference to the value in the entry.
    pub fn or_insert_with<F>(self, default: F) -> &'e [u8]
    where
        F: FnOnce() -> &'e [u8],
    {
        self.or_insert_with_key(|_| default())
    }

    #[inline]
    /// Ensures a value is in the entry by inserting, if empty, the result of the default function. This method allows for generating key-derived values for insertion by providing the default function a reference to the key that was moved during the `.entry(key)` method call.
    pub fn or_insert_with_key<F>(self, default: F) -> &'e [u8]
    where
        F: FnOnce(&[u8]) -> &'e [u8],
    {
        match self {
            Entry::Occupied(entry) => entry.get(),
            Entry::Vacant(entry) => {
                let value = default(entry.1);
                entry.insert(value)
            }
        }
    }

    #[inline(always)]
    /// Ensures a value is in the entry by inserting the default value if empty, and returns a reference to the value in the entry.
    pub fn or_default(self) -> &'e [u8] {
        self.or_insert(Default::default())
    }

    #[inline(always)]
    /// Returns a reference to this entry’s key.
    pub fn key(&self) -> &'e [u8] {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    #[inline]
    /// Provides in-place mutable access to an occupied entry before any potential inserts into the map.
    pub fn and_modify<F>(self, modify: F) -> Entry<'e>
    where
        F: FnOnce(&mut std::borrow::Cow<'e, [u8]>),
    {
        match &self {
            Entry::Vacant(_) => self,
            Entry::Occupied(occupied) => {
                let mut entry = occupied.0.borrow_mut();
                let val = entry.val().unwrap();

                let mut value = std::borrow::Cow::Borrowed(val);
                modify(&mut value);

                match value {
                    std::borrow::Cow::Borrowed(_) => {} // no changes were made
                    std::borrow::Cow::Owned(value) => {
                        entry.replace(&value).unwrap();
                    }
                }

                drop(entry);
                self
            }
        }
    }
}

use crate::range::Bound;
use std::cell::RefCell;

pub struct VacantEntry<'e>(Bound<'e>, &'e [u8]);

impl<'e> VacantEntry<'e> {
    #[inline(always)]
    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &'e [u8] {
        self.1
    }

    #[inline(always)]
    /// Take ownership of the key.
    pub fn into_key(self) -> &'e [u8] {
        self.key()
    }

    #[inline(always)]
    /// Sets the value of the entry with the VacantEntry’s key, and returns a reference to it.
    pub fn insert(self, value: &'e [u8]) -> &'e [u8] {
        let mut entry = self.0;
        entry.insert(self.1, value).unwrap();
        value
    }
}

pub struct OccupiedEntry<'e>(RefCell<Bound<'e>>);

impl<'e> OccupiedEntry<'e> {
    #[inline]
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &'e [u8] {
        self.0.borrow_mut().key().unwrap()
    }

    #[inline]
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &'e [u8] {
        self.0.borrow_mut().val().unwrap()
    }

    #[inline]
    /// Sets the value of the entry with the OccupiedEntry’s key, and returns the entry’s old value.
    pub fn insert(&self, value: &'e [u8]) -> Vec<u8> {
        let mut entry = self.0.borrow_mut();
        let old = entry.val().unwrap().to_vec();
        entry.replace(value).unwrap();

        old
    }

    #[inline]
    /// Takes the value of the entry out of the map, and returns it.
    pub fn remove(self) -> Vec<u8> {
        let mut entry = self.0.borrow_mut();
        let old = entry.val().unwrap().to_vec();
        entry.remove().unwrap();

        old
    }

    #[inline]
    /// Take ownership of the key and value from the map.
    pub fn remove_entry(self) -> (Vec<u8>, Vec<u8>) {
        let mut entry = self.0.borrow_mut();
        let key = entry.key().unwrap().to_vec();
        let val = entry.val().unwrap().to_vec();
        entry.remove().unwrap();

        (key, val)
    }
}
