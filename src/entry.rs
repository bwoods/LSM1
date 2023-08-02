extern crate lsm_ext;
use lsm_ext::*;

use super::Map;

use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

impl<'e, 'm: 'e> Map<'m> {
    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: &'e [u8]) -> Entry<'e> {
        let mut cursor: *mut lsm_cursor = null_mut();
        let marker = Default::default();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().unwrap();

            match lsm_csr_valid(cursor) {
                true => Entry::Occupied(OccupiedEntry(cursor, self.db, marker)),
                false => {
                    let _ = lsm_csr_close(cursor);
                    Entry::Vacant(VacantEntry(key, self.db))
                }
            }
        }
    }
}

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
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is unnecessary, unlike with `.or_insert_with(|| ... )`.
    pub fn or_insert_with_key<F>(self, default: F) -> &'e [u8]
    where
        F: FnOnce(&[u8]) -> &'e [u8],
    {
        match self {
            Entry::Vacant(entry) => {
                let value = default(entry.0);
                entry.insert(value)
            }
            Entry::Occupied(entry) => entry.get(),
        }
    }

    #[inline(always)]
    /// Returns a reference to this entry’s key.
    pub fn key(&self) -> &'e [u8] {
        match self {
            Entry::Vacant(entry) => entry.key(),
            Entry::Occupied(entry) => entry.key(),
        }
    }

    #[inline]
    /// Provides in-place mutable access to an occupied entry before any potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Entry<'e>
    where
        F: FnOnce(&mut [u8]),
    {
        unsafe {
            match &self {
                Entry::Vacant(_) => self,
                Entry::Occupied(entry) => {
                    let mut ptr: *const u8 = null_mut();
                    let mut len: u32 = 0;

                    lsm_csr_value(entry.0, &mut ptr, &mut len).ok().unwrap();
                    let mut value = from_raw_parts(ptr, len as usize).to_vec();
                    f(&mut value);

                    lsm_csr_key(entry.0, &mut ptr, &mut len).ok().unwrap();
                    lsm_insert(entry.1, ptr, len, value.as_ptr(), value.len() as u32)
                        .ok()
                        .unwrap();

                    self
                }
            }
        }
    }

    #[inline(always)]
    /// Ensures a value is in the entry by inserting the default value if empty, and returns a reference to the value in the entry.
    pub fn or_default(self) -> &'e [u8] {
        self.or_insert(Default::default())
    }
}

pub struct VacantEntry<'e>(&'e [u8], *mut lsm_db);

impl<'e> VacantEntry<'e> {
    #[inline(always)]
    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &'e [u8] {
        self.0
    }

    #[inline(always)]
    /// Take ownership of the key.
    pub fn into_key(self) -> &'e [u8] {
        self.key()
    }

    #[inline(always)]
    /// Sets the value of the entry with the VacantEntry’s key, and returns a reference to it.
    pub fn insert(self, value: &'e [u8]) -> &'e [u8] {
        unsafe {
            lsm_insert(
                self.1,
                self.0.as_ptr(),
                self.0.len() as u32,
                value.as_ptr(),
                value.len() as u32,
            )
            .ok()
            .unwrap();
        }

        value
    }
}

pub struct OccupiedEntry<'e>(*mut lsm_cursor, *mut lsm_db, PhantomData<&'e u8>);

impl<'e> OccupiedEntry<'e> {
    #[inline]
    pub fn key(&self) -> &'e [u8] {
        unsafe {
            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_key(self.0, &mut ptr, &mut len).ok().unwrap();
            from_raw_parts(ptr, len as usize)
        }
    }

    pub fn remove_entry(self) -> (Vec<u8>, Vec<u8>) {
        let mut ptr: *const u8 = null_mut();
        let mut len: u32 = 0;

        unsafe {
            lsm_csr_key(self.0, &mut ptr, &mut len).ok().unwrap();
            let key = from_raw_parts(ptr, len as usize).to_vec();

            lsm_csr_value(self.0, &mut ptr, &mut len).ok().unwrap();
            let value = from_raw_parts(ptr, len as usize).to_vec();

            lsm_delete(self.1, key.as_ptr(), key.len() as u32)
                .ok()
                .unwrap();

            (key, value)
        }
    }

    #[inline]
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &'e [u8] {
        unsafe {
            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_value(self.0, &mut ptr, &mut len).ok().unwrap();
            from_raw_parts(ptr, len as usize)
        }
    }

    #[inline]
    /// Sets the value of the entry with the OccupiedEntry’s key, and returns the entry’s old value.
    pub fn insert(&mut self, value: &'e [u8]) -> Vec<u8> {
        unsafe {
            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_value(self.0, &mut ptr, &mut len).ok().unwrap();
            let old = from_raw_parts(ptr, len as usize).to_vec();

            lsm_csr_key(self.0, &mut ptr, &mut len).ok().unwrap();

            lsm_insert(self.1, ptr, len, value.as_ptr(), value.len() as u32)
                .ok()
                .unwrap();

            old
        }
    }

    #[inline(always)]
    /// Takes the value of the entry out of the map, and returns it.
    pub fn remove(self) -> Vec<u8> {
        unsafe {
            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_value(self.0, &mut ptr, &mut len).ok().unwrap();
            let value = from_raw_parts(ptr, len as usize).to_vec();

            lsm_csr_key(self.0, &mut ptr, &mut len).ok().unwrap();
            lsm_delete(self.1, ptr, len as u32).ok().unwrap();

            value
        }
    }
}

impl Drop for OccupiedEntry<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let _ = lsm_csr_close(self.0);
        }
    }
}
