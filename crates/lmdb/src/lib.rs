use self::ffi::{MDB_dbi, MDB_envinfo, MDB_stat};

mod bitflag;
mod ffi;

pub type EnvInfo = MDB_envinfo;
pub type StatInfo = MDB_stat;
pub type DbHandle = MDB_dbi;

pub mod cursor;
pub mod database;
pub mod db_name;
pub mod env;
pub mod error;
pub mod flag;
pub mod transaction;
pub mod value;

#[cfg(test)]
mod test {
  use crate::flag::WriteFlag;
  use crate::transaction::Transaction;
  use crate::value::IntoValueRef;
  use crate::{db_name::DbName, env::Env, flag::DbFlag};
  use std::num::{NonZeroU32, NonZeroUsize};

  #[test]
  fn db_test() {
    let db_name = unsafe { DbName::from_static("bruh") };
    let mut env = Env::new()
      .set_max_dbs(unsafe { NonZeroU32::new_unchecked(2) })
      .set_permissions(0o644)
      .set_map_size(unsafe { NonZeroUsize::new_unchecked(8096 * 20) })
      .open("./test.db")
      .unwrap();

    let mut db = env.open_database(&db_name, DbFlag::CREATE).unwrap();
    let mut txn = db.begin_rw_transaction().unwrap();
    let mut child = txn.begin_rw_child().unwrap();

    for i in 0..100 {
      let value = format!("data__{i}");
      child
        .put(
          format!("skey__{i}").as_value_ref(),
          value.as_value_ref(),
          WriteFlag::default(),
        )
        .unwrap();
    }

    child.commit().unwrap();

    if let Some(a) = txn.get("skey__0".as_value_ref()).unwrap() {
      let data = a.as_bytes();
      let meta_len = a.len();
      let real_len = data.len();

      println!("data: {data:?} meta_len: {meta_len}, real_len: {real_len}");
    } else {
      println!("key does not exists");
    }

    txn.commit().unwrap();

    println!("{:?}", db.stats());
    println!("{:?}", env.stats());
    println!("{:?}", env.info());

    env.close_database(&db_name).unwrap();
    env.close();
  }
}
