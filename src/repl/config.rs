use crate::error::Result;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ReplConfig {
    pub data_dir: PathBuf,
}

impl ReplConfig {
    /// Returns the available databases found in data_dir.
    pub fn dbs(&self) -> Result<Vec<PathBuf>> {
        Ok(fs::read_dir(&self.data_dir)?
            .filter_map(|dir_entry| dir_entry.ok())
            .map(|dir_entry| dir_entry.path())
            .filter(|path| path.extension() == Some(OsStr::new("db")))
            .collect())
    }
}
