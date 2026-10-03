use self::internal::ffi::{MDB_dbi, MDB_envinfo, MDB_stat};

mod internal;

pub type EnvInfo = MDB_envinfo;
pub type StatInfo = MDB_stat;
pub type DbHandle = MDB_dbi;

pub mod cursor;
pub mod db_name;
pub mod env;
pub mod error;
pub mod flag;
pub mod reader_info;
pub mod transaction;
pub mod value;
