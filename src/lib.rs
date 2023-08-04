#![allow(non_camel_case_types, dead_code)]
#![doc = include_str!("../README.md")]

extern crate lsm_ext;
use lsm_ext::*;

mod entry;
mod map;
mod range;

use std::ffi::CString;
use std::ptr::null_mut;

pub struct Tree {
    db: *mut lsm_db,
}

impl Tree {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut db: *mut lsm_db = null_mut();
        let path = CString::new(path).unwrap();

        unsafe {
            lsm_new(null_mut(), &mut db).ok()?;
            lsm_open(db, path.as_ptr() as *const u8).ok()?;
        }

        Ok(Tree { db })
    }

    pub fn entry<'e>(&self, key: &'e [u8]) -> entry::Entry<'e> {
        entry::Entry::new_in(self.db, key.as_ref())
    }

    pub fn range<'a, R>(&self, range: R) -> impl Iterator + 'a
    where
        R: std::ops::RangeBounds<&'a [u8]>,
    {
        range::RangeBounds::new_in(self.db, range).unwrap()
    }

    pub fn iter(&self) -> impl Iterator {
        self.range(..)
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
