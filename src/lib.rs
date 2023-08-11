#![allow(non_camel_case_types, dead_code, clippy::bool_comparison)]
#![doc = include_str!("../README.md")]

extern crate lsm_ext;
use lsm_ext::*;

mod entry;
mod file;
mod map;
mod range;

#[cfg(test)]
mod test;

pub(crate) struct Tree {
    db: *mut lsm_db,
}

impl Tree {
    pub fn entry<'e>(&self, key: &'e [u8]) -> entry::Entry<'e> {
        entry::Entry::new_in(self.db, key)
    }

    pub fn range<'r, R>(&self, range: R) -> range::RangeBounds<'r>
    where
        R: std::ops::RangeBounds<&'r [u8]>,
    {
        range::RangeBounds::new_in(self.db, range).unwrap()
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
