extern crate lsm_ext;
use lsm_ext::*;

use std::marker::PhantomData;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

enum Bound<'a> {
    Included(*mut lsm_db, *mut lsm_cursor),
    Unbounded(
        *mut lsm_db,
        unsafe extern "C" fn(cursor: *mut lsm_cursor) -> Error,
        PhantomData<&'a u8>,
    ),
}

impl<'a> Bound<'a> {
    #[inline(never)]
    /// Unbounded bounds refrain from allocating a cursor until it is needed.
    fn new(
        key: std::ops::Bound<&[u8]>,
        seek: Seek,
        next: unsafe extern "C" fn(cursor: *mut lsm_cursor) -> Error,
        db: *mut lsm_db,
    ) -> Result<Self, Error> {
        unsafe {
            let mut cursor = null_mut();

            // using an inner closure to allow ?-syntax to be used
            let bound = (|| -> Result<Self, Error> {
                match key {
                    std::ops::Bound::Unbounded => {
                        Ok(Bound::Unbounded(db, next, Default::default()))
                    }
                    std::ops::Bound::Included(key) => {
                        lsm_csr_open(db, &mut cursor).ok()?;
                        lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, seek).ok()?;

                        Ok(Bound::Included(db, cursor))
                    }
                    std::ops::Bound::Excluded(key) => {
                        lsm_csr_open(db, &mut cursor).ok()?;
                        lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, seek).ok()?;

                        let mut cmp = 0;
                        lsm_csr_cmp(cursor, key.as_ptr(), key.len() as u32, &mut cmp).ok()?;

                        if cmp == 0 {
                            next(cursor).ok()?;
                        }

                        Ok(Bound::Included(db, cursor))
                    }
                }
            })();

            // close cursor on errors
            bound.map_err(|error| {
                let _ = lsm_csr_close(cursor); // ignores null ptrs properly
                error
            })
        }
    }

    fn db(&self) -> *mut lsm_db {
        match self {
            Bound::Included(db, ..) => *db,
            Bound::Unbounded(db, ..) => *db,
        }
    }

    fn cursor(&mut self) -> Result<*mut lsm_cursor, Error> {
        match self {
            Bound::Included(_, cursor) => Ok(*cursor),
            Bound::Unbounded(db, position, ..) => {
                unsafe {
                    let mut cursor = null_mut();
                    lsm_csr_open(*db, &mut cursor).ok()?;
                    position(cursor).ok().map_err(|error| {
                        let _ = lsm_csr_close(cursor);
                        error
                    })?;

                    // Unbounded bounds are lazily loaded; right here
                    *self = Bound::Included(*db, cursor);
                    Ok(cursor)
                }
            }
        }
    }

    pub fn is_included(&self) -> bool {
        match &self {
            Bound::Included(..) => true,
            Bound::Unbounded(..) => false,
        }
    }

    pub fn key(&mut self) -> Result<&'a [u8], Error> {
        let mut ptr: *const u8 = null_mut();
        let mut len: u32 = 0;

        unsafe {
            lsm_csr_key(self.cursor()?, &mut ptr, &mut len).ok()?;
            Ok(from_raw_parts(ptr, len as usize))
        }
    }

    pub fn val(&mut self) -> Result<&'a [u8], Error> {
        let mut ptr: *const u8 = null_mut();
        let mut len: u32 = 0;

        unsafe {
            lsm_csr_value(self.cursor()?, &mut ptr, &mut len).ok()?;
            Ok(from_raw_parts(ptr, len as usize))
        }
    }

    pub fn replace(&mut self, value: &[u8]) -> Result<(), Error> {
        let key = self.key()?;
        let ptr = key.as_ptr();
        let len = key.len() as u32;

        unsafe {
            lsm_insert(self.db(), ptr, len, value.as_ptr(), value.len() as u32).ok()?;
        }

        Ok(())
    }

    pub fn remove(&mut self) -> Result<(), Error> {
        let key = self.key()?;
        let ptr = key.as_ptr();
        let len = key.len() as u32;

        unsafe {
            lsm_delete(self.db(), ptr, len).ok()?;
        }

        Ok(())
    }
}

struct RangeBounds<'a> {
    start_bound: Bound<'a>,
    end_bound: Bound<'a>,
}

impl<'a> RangeBounds<'a> {
    pub(crate) fn new(
        bounds: impl std::ops::RangeBounds<[u8]>,
        db: *mut lsm_db,
    ) -> Result<Self, Error> {
        let start_bound = Bound::new(bounds.start_bound(), Seek::GE, lsm_csr_next, db)?;
        let end_bound = Bound::new(bounds.end_bound(), Seek::LE, lsm_csr_prev, db)?;

        Ok(RangeBounds {
            start_bound,
            end_bound,
        })
    }
}

impl<'a> Iterator for RangeBounds<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.start_bound.cursor().ok()?;

        unsafe {
            if lsm_csr_valid(cursor) == false {
                return None;
            }

            if self.end_bound.is_included() {
                let mut cmp = 0;
                let key = self.end_bound.key().ok()?;

                lsm_csr_cmp(cursor, key.as_ptr(), key.len() as u32, &mut cmp)
                    .ok()
                    .ok()?;

                if cmp >= 0 {
                    return None;
                }
            }

            let value = self
                .start_bound
                .val()
                .and_then(|val| {
                    lsm_csr_next(cursor).ok()?;
                    Ok(val)
                })
                .ok()?;

            Some(value)
        }
    }
}

impl<'a> DoubleEndedIterator for RangeBounds<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let cursor = self.end_bound.cursor().ok()?;

        unsafe {
            if lsm_csr_valid(cursor) == false {
                return None;
            }

            if self.start_bound.is_included() {
                let mut cmp = 0;
                let key = self.start_bound.key().ok()?;

                lsm_csr_cmp(cursor, key.as_ptr(), key.len() as u32, &mut cmp)
                    .ok()
                    .ok()?;

                if cmp <= 0 {
                    return None;
                }
            }

            let value = self
                .end_bound
                .val()
                .and_then(|val| {
                    lsm_csr_next(cursor).ok()?;
                    Ok(val)
                })
                .ok()?;

            Some(value)
        }
    }
}