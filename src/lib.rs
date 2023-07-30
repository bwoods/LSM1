#![allow(non_camel_case_types, dead_code)]
#![doc = include_str!("../README.md")]

extern crate lsm_ext;
use lsm_ext::*;

extern crate scopeguard;
use scopeguard::defer;

mod iter;
mod range;

use std::ffi::CString;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

pub struct Map<'a> {
    db: *mut lsm_db,
    marker: std::marker::PhantomData<&'a u8>,
}

impl Map<'_> {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut db: *mut lsm_db = null_mut();
        let path = CString::new(path).unwrap();

        unsafe {
            lsm_new(null_mut(), &mut db).ok()?;
            lsm_open(db, path.as_ptr() as *const u8).ok()?;
        }

        Ok(Map {
            db,
            marker: Default::default(),
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let mut cursor: *mut lsm_cursor = null_mut();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().ok()?;
            defer! { let _ = lsm_csr_close(cursor); }

            lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, Seek::EQ)
                .ok()
                .ok()?;

            if lsm_csr_valid(cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_value(cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            Some(val)
        }
    }

    pub fn get_key_value(&self, key: &[u8]) -> Option<(&[u8], &[u8])> {
        let mut cursor: *mut lsm_cursor = null_mut();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().ok()?;
            defer! { let _ = lsm_csr_close(cursor); }

            lsm_csr_seek(cursor, key.as_ptr(), key.len() as u32, Seek::EQ)
                .ok()
                .ok()?;

            if lsm_csr_valid(cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_key(cursor, &mut ptr, &mut len).ok().ok()?;
            let key = from_raw_parts(ptr, len as usize);

            lsm_csr_value(cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            Some((key, val))
        }
    }

    pub fn first_key_value(&self) -> Option<(&[u8], &[u8])> {
        let mut cursor: *mut lsm_cursor = null_mut();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().ok()?;
            defer! { let _ = lsm_csr_close(cursor); }

            lsm_csr_first(cursor).ok().ok()?;

            if lsm_csr_valid(cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_key(cursor, &mut ptr, &mut len).ok().ok()?;
            let key = from_raw_parts(ptr, len as usize);

            lsm_csr_value(cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            Some((key, val))
        }
    }

    pub fn last_key_value(&self) -> Option<(&[u8], &[u8])> {
        let mut cursor: *mut lsm_cursor = null_mut();

        unsafe {
            lsm_csr_open(self.db, &mut cursor).ok().ok()?;
            defer! { let _ = lsm_csr_close(cursor); }

            lsm_csr_last(cursor).ok().ok()?;

            if lsm_csr_valid(cursor) == false {
                return None; // TODO: is this check redundant?
            }

            let mut ptr: *const u8 = null_mut();
            let mut len: u32 = 0;

            lsm_csr_key(cursor, &mut ptr, &mut len).ok().ok()?;
            let key = from_raw_parts(ptr, len as usize);

            lsm_csr_value(cursor, &mut ptr, &mut len).ok().ok()?;
            let val = from_raw_parts(ptr, len as usize);

            Some((key, val))
        }
    }

    pub fn contains_key(&self, key: &[u8]) -> bool {
        let mut cursor: *mut lsm_cursor = null_mut();

        let ptr = key.as_ptr();
        let len = key.len() as u32;

        unsafe {
            let val = lsm_csr_open(self.db, &mut cursor).ok().is_ok()
                && lsm_csr_seek(cursor, ptr, len as u32, Seek::EQ).ok().is_ok()
                && lsm_csr_valid(cursor);

            let _ = lsm_csr_close(cursor);
            val
        }
    }

    pub fn insert(&mut self, key: &[u8], val: &[u8]) {
        unsafe {
            let _ = lsm_insert(
                self.db,
                key.as_ptr(),
                key.len() as u32,
                val.as_ptr(),
                val.len() as u32,
            );
        }
    }

    pub fn remove(&mut self, key: &[u8]) {
        unsafe {
            let _ = lsm_delete(self.db, key.as_ptr(), key.len() as u32);
        }
    }
}

impl Drop for Map<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = lsm_close(self.db);
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Error,
    Busy,
    Nomem,
    IoErr,
    Corrupt,
    Full,
    CantOpen,
    Protocol,
    Misuse,
    NoEnt,
}

impl From<lsm_ext::Error> for Error {
    /// SAFETY: panics if `raw` is `Error::Ok`
    fn from(raw: lsm_ext::Error) -> Self {
        match raw {
            lsm_ext::Error::Error => Error::Error,
            lsm_ext::Error::Busy => Error::Busy,
            lsm_ext::Error::Nomem => Error::Nomem,
            lsm_ext::Error::IoErr => Error::IoErr,
            lsm_ext::Error::Corrupt => Error::Corrupt,
            lsm_ext::Error::Full => Error::Full,
            lsm_ext::Error::CantOpen => Error::CantOpen,
            lsm_ext::Error::Protocol => Error::Protocol,
            lsm_ext::Error::Misuse => Error::Misuse,
            lsm_ext::Error::NoEnt => Error::NoEnt,
            lsm_ext::Error::Ok => unreachable!(),
        }
    }
}
