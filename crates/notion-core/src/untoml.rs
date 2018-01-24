use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io::{self, Read};
use std::str::FromStr;
use std::convert::From;

use toml::value::{Value, Table};
use failure::{self, Fail};

pub fn touch(path: &Path) -> io::Result<File> {
    if !path.is_file() {
        let basedir = path.parent().unwrap();
        create_dir_all(basedir)?;
        File::create(path)?;
    }
    File::open(path)
}

pub fn load<T: FromStr>(path: &Path) -> Result<T, T::Err>
  where T::Err: From<io::Error>
{
    let mut file = touch(path)?;
    let mut source = String::new();
    file.read_to_string(&mut source)?;
    source.parse()
}

pub trait ParseToml {
    fn table<E: Fail, F: FnOnce(String) -> E>(self, key: &str, mk_err: F) -> Result<Table, failure::Error>;
    fn string<E: Fail, F: FnOnce(String) -> E>(self, key: &str, mk_err: F) -> Result<String, failure::Error>;
}

impl ParseToml for Value {
    fn table<E: Fail, F: FnOnce(String) -> E>(self, key: &str, mk_err: F) -> Result<Table, failure::Error> {
        if let Value::Table(map) = self {
            Ok(map)
        } else {
            Err(mk_err(String::from(key)).into())
        }
    }

    fn string<E: Fail, F: FnOnce(String) -> E>(self, key: &str, mk_err: F) -> Result<String, failure::Error> {
        if let Value::String(string) = self {
            Ok(string)
        } else {
            Err(mk_err(String::from(key)).into())
        }
    }
}

pub trait Extract {
    fn extract<E: Fail, F: FnOnce(String) -> E>(&mut self, key: &str, mk_err: F) -> Result<Value, failure::Error>;
}

impl Extract for Table {
    fn extract<E: Fail, F: FnOnce(String) -> E>(&mut self, key: &str, mk_err: F) -> Result<Value, failure::Error> {
        self.remove(key).ok_or(mk_err(String::from(key)).into())
    }
}
