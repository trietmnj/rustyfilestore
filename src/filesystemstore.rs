use std::{ffi::OsString, fs};

use anyhow::{Error, Result};

use crate::store::{self, FileStoreResult};

pub struct LocalStore {}

impl LocalStore {
    fn new(self) -> Self {
        self
    }
}

impl store::FileStore for LocalStore {
    fn get_dir(path: store::PathConfig) -> Result<Vec<store::FileStoreResult>, Error> {
        let entries = fs::read_dir(path.path)?;
        let mut v: Vec<FileStoreResult> = Vec::new();
        for (i, en) in entries.enumerate() {
            // p borrows from en and propagates away the errors
            let p = &en?;
            let meta = p.metadata()?;
            let full_path = p.path();
            let file_type = full_path
                .extension()
                .unwrap_or(&OsString::new())
                .to_string_lossy()
                .to_string();
            let r = FileStoreResult {
                id: u32::try_from(i).unwrap(),
                name: p.file_name().to_string_lossy().to_string(),
                size: meta.len(),
                path: p.path().to_string_lossy().to_string(),
                file_type,
                is_dir: meta.is_dir(),
                modified: chrono::DateTime::from(meta.modified()?),
            };
            v.push(r);
        }
        Ok(v)
    }

    fn get_obj(path: store::PathConfig) -> Result<Vec<store::FileStoreResult>, Error> {
        todo!()
    }

    fn put_obj(path: store::PathConfig) -> Result<Vec<store::FileStoreResult>, Error> {
        todo!()
    }

    fn upload_file(path: store::PathConfig) -> Result<Vec<store::FileStoreResult>, Error> {
        todo!()
    }
}
