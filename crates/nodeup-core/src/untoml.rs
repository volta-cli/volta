use std::collections::HashMap;
use std::path::Path;

use toml::value::{Value, Table};

pub trait ParseToml {
    fn table<F: FnOnce(String) -> ::ErrorKind>(self, key: &str, mk_err: F) -> ::Result<Table>;
    fn string<F: FnOnce(String) -> ::ErrorKind>(self, key: &str, mk_err: F) -> ::Result<String>;
}

impl ParseToml for Value {
    fn table<F: FnOnce(String) -> ::ErrorKind>(self, key: &str, mk_err: F) -> ::Result<Table> {
        if let Value::Table(map) = self {
            Ok(map)
        } else {
            bail!(mk_err(String::from(key)));
        }
    }

    fn string<F: FnOnce(String) -> ::ErrorKind>(self, key: &str, mk_err: F) -> ::Result<String> {
        if let Value::String(string) = self {
            Ok(string)
        } else {
            bail!(mk_err(String::from(key)));
        }
    }
}

pub trait Extract {
    fn extract<F: FnOnce(String) -> ::ErrorKind>(&mut self, key: &str, mk_err: F) -> ::Result<Value>;
}

impl Extract for Table {
    fn extract<F: FnOnce(String) -> ::ErrorKind>(&mut self, key: &str, mk_err: F) -> ::Result<Value> {
        self.remove(key).ok_or(mk_err(String::from(key)).into())
    }
}
