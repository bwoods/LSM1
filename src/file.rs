use crate::Tree;

extern crate lsm_ext;
use lsm_ext::*;

use std::ffi::CString;
use std::ptr::null_mut;

impl Tree {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut db: *mut lsm_db = null_mut();
        let path = CString::new(path).map_err(|_| Error::NoEnt)?;

        unsafe {
            lsm_new(null_mut(), &mut db).ok()?;
            lsm_open(db, path.as_ptr() as *const u8).ok()?;
        }

        Ok(Tree { db })
    }
}
