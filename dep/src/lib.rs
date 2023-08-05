#![allow(non_camel_case_types, dead_code)]

use std::num::NonZeroU32;

pub enum lsm_compress {}
pub enum lsm_compress_factory {}
pub enum lsm_cursor {}
pub enum lsm_db {}
pub enum lsm_env {}
pub enum lsm_file {}
pub enum lsm_mutex {}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Lock {
    Unlock = 0,
    Shared = 1,
    Excl = 2,
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Open {
    ReadWrite = 0x0000,
    ReadOnly = 0x0001,
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Mutex {
    Global = 1,
    Heap = 2,
}

#[must_use]
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Ok = 0,
    Error = 1,
    Busy = 5,
    Nomem = 7,
    IoErr = 10,
    Corrupt = 11,
    Full = 13,
    CantOpen = 14,
    Protocol = 15,
    Misuse = 21,
    NoEnt = (10 | (1 << 8)),
}

impl Error {
    pub fn ok(self) -> Result<(), Self> {
        match self {
            Error::Ok => Ok(()),
            _ => Err(self),
        }
    }
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Config {
    AutoFlush = 1,
    PageSize = 2,
    Safety = 3,
    BlockSize = 4,
    AutoWork = 5,
    Mmap = 7,
    UseLog = 8,
    AutoMerge = 9,
    MaxFreelist = 10,
    MultipleProcesses = 11,
    AutoCheckpoint = 12,
    SetCompression = 13,
    GetCompression = 14,
    SetCompressionFactory = 15,
    Readonly = 16,
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Safety {
    Off = 0,
    Normal = 1,
    Full = 2,
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Compression {
    Empty = 0,
    None = 1,
    LZ4 = i32::from_be_bytes(*b"LZ4 "),
    Zstd = i32::from_be_bytes(*b"zstd"),
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Info {
    NumWrite = 1,
    NumRead = 2,
    DbStructure = 3,
    LogStructure = 4,
    ArrayStructure = 5,
    PageAsciiDump = 6,
    PageHexDump = 7,
    Freelist = 8,
    ArrayPages = 9,
    CheckpointSize = 10,
    TreeSize = 11,
    FreelistSize = 12,
    CompressionId = 13,
}

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Seek {
    LEFast = -2,
    LE = -1,
    EQ = 0,
    GE = 1,
}

extern "C" {
    pub fn lsm_new(env: *mut lsm_env, db: *mut *mut lsm_db) -> Error;
    pub fn lsm_close(db: *mut lsm_db) -> Error;
    pub fn lsm_open(db: *mut lsm_db, filename: *const u8) -> Error;

    pub fn lsm_get_env(db: *mut lsm_db) -> *mut lsm_env;
    pub fn lsm_default_env() -> *mut lsm_env;

    pub fn lsm_config(db: *mut lsm_db, config: Config, ...) -> Error;

    pub fn lsm_malloc(env: *mut lsm_env, size: usize) -> *mut u8;
    pub fn lsm_realloc(env: *mut lsm_env, ptr: *mut u8, size: usize) -> *mut u8;
    pub fn lsm_free(env: *mut lsm_env, ptr: *mut u8);

    pub fn lsm_info(db: *mut lsm_db, info: Info, ...) -> Error;
    pub fn lsm_get_user_version(db: *mut lsm_db, version: *mut u32) -> Error;
    pub fn lsm_set_user_version(db: *mut lsm_db, version: u32) -> Error;

    pub fn lsm_begin(db: *mut lsm_db, level: NonZeroU32) -> Error;
    pub fn lsm_commit(db: *mut lsm_db, level: NonZeroU32) -> Error;
    pub fn lsm_rollback(db: *mut lsm_db, level: NonZeroU32) -> Error;

    pub fn lsm_insert(
        db: *mut lsm_db,
        key: *const u8,
        key_len: u32,
        val: *const u8,
        val_len: u32,
    ) -> Error;

    pub fn lsm_delete(db: *mut lsm_db, ket: *const u8, key_len: u32) -> Error;
    pub fn lsm_delete_range(
        db: *mut lsm_db,
        before: *const u8,
        before_len: u32,
        after: *const u8,
        after_len: u32,
    ) -> Error;

    pub fn lsm_work(db: *mut lsm_db, merge: NonZeroU32, kilobytes: u32, written: *mut i32)
        -> Error;

    pub fn lsm_flush(db: *mut lsm_db) -> Error;
    pub fn lsm_checkpoint(db: *mut lsm_db, kilobytes: *mut u32) -> Error;

    pub fn lsm_csr_open(db: *mut lsm_db, cursor: *mut *mut lsm_cursor) -> Error;
    pub fn lsm_csr_close(cursor: *mut lsm_cursor) -> Error;

    pub fn lsm_csr_first(cursor: *mut lsm_cursor) -> Error;
    pub fn lsm_csr_last(cursor: *mut lsm_cursor) -> Error;

    pub fn lsm_csr_seek(cursor: *mut lsm_cursor, key: *const u8, key_len: u32, set: Seek) -> Error;
    pub fn lsm_csr_next(cursor: *mut lsm_cursor) -> Error;
    pub fn lsm_csr_prev(cursor: *mut lsm_cursor) -> Error;

    pub fn lsm_csr_valid(cursor: *mut lsm_cursor) -> bool;
    pub fn lsm_csr_key(cursor: *mut lsm_cursor, key: *mut *const u8, key_len: *mut u32) -> Error;
    pub fn lsm_csr_value(cursor: *mut lsm_cursor, val: *mut *const u8, val_len: *mut u32) -> Error;

    pub fn lsm_csr_cmp(
        cursor: *mut lsm_cursor,
        key: *const u8,
        key_len: u32,
        cmp: *mut i32,
    ) -> Error;
}
