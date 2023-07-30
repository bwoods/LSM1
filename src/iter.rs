extern crate lsm_ext;
use lsm_ext::*;

use super::Map;

use std::ptr::null_mut;
use std::slice::from_raw_parts;

impl Map<'_> {
    pub fn iter(&self) -> Iter<'_> {
        let mut cursor: *mut lsm_cursor = null_mut();
        let marker = Default::default();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().unwrap();
            lsm_csr_first(cursor).ok().unwrap();
        }

        Iter { cursor, marker }
    }
}

pub struct Iter<'a> {
    pub(crate) cursor: *mut lsm_cursor,
    pub(crate) marker: std::marker::PhantomData<&'a u8>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if lsm_csr_valid(self.cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_value(self.cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            lsm_csr_next(self.cursor).ok().ok()?;
            Some(val)
        }
    }
}

impl Drop for Iter<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = lsm_csr_close(self.cursor);
        }
    }
}
