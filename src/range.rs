extern crate lsm_ext;
use lsm_ext::*;

use super::Map;

use std::collections::Bound;
use std::ops::RangeBounds;

use std::ptr::null_mut;
use std::slice::from_raw_parts;

pub struct Range<'a, R: RangeBounds<&'a [u8]>> {
    pub(crate) cursor: *mut lsm_cursor,
    pub(crate) bounds: R,
    pub(crate) marker: std::marker::PhantomData<&'a u8>,
}

impl<'a> Map<'a> {
    pub fn range<R: RangeBounds<&'a [u8]>>(&self, bounds: R) -> Range<'a, R> {
        let mut cursor: *mut lsm_cursor = null_mut();
        let marker = Default::default();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().unwrap();

            match bounds.start_bound() {
                Bound::Unbounded => lsm_csr_first(cursor).ok().unwrap(),
                Bound::Included(key) => {
                    lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, Seek::GE)
                        .ok()
                        .unwrap();
                }
                Bound::Excluded(key) => {
                    lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, Seek::GE)
                        .ok()
                        .unwrap();

                    let mut cmp = 0;
                    lsm_csr_cmp(cursor, key.as_ptr(), key.len() as u32, &mut cmp)
                        .ok()
                        .unwrap();

                    if cmp <= 0 {
                        lsm_csr_next(cursor).ok().unwrap();
                    }
                }
            };
        }

        Range {
            cursor,
            bounds,
            marker,
        }
    }
}

impl<'a, R: RangeBounds<&'a [u8]>> Iterator for Range<'a, R> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if lsm_csr_valid(self.cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_key(self.cursor, &mut ptr, &mut len).ok().ok()?;
            let key = from_raw_parts(ptr, len as usize);

            if self.bounds.contains(&key) == false {
                return None;
            }

            lsm_csr_value(self.cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            lsm_csr_next(self.cursor).ok().ok()?;
            Some(val)
        }
    }
}

impl<'a, R: RangeBounds<&'a [u8]>> Drop for Range<'a, R> {
    fn drop(&mut self) {
        unsafe {
            let _ = lsm_csr_close(self.cursor);
        }
    }
}
