use std::{
    ffi::OsString,
    fs::{self, create_dir, File},
    io::{self, BufRead, BufReader, Write},
};

use crate::store::{self, FileOprationOuput, FileStoreResult};
use anyhow::{Error, Result};
use bytes::Bytes;
use sha2::{Digest, Sha256};

pub struct FileSystemStore {}

impl FileSystemStore {
    // pub fn new(&self) -> Self {
    //     self
    // }
}

impl store::FileStore for FileSystemStore {
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
                modified_by: "".to_string(),
            };
            v.push(r);
        }
        Ok(v)
    }

    fn get_object(path: store::PathConfig) -> Result<Box<dyn BufRead>, Error> {
        let f = File::open(path.path)?;
        Ok(Box::new(BufReader::new(f)))
    }

    fn put_object(path: store::PathConfig, data: Bytes) -> Result<Box<FileOprationOuput>, Error> {
        let mut o = FileOprationOuput { sha256: [0; 32] };
        if data.len() == 0 {
            // no data -> create dir
            create_dir(path.path)?;
        } else {
            // create file with data
            let mut f = File::create(path.path)?;
            f.write_all(&data)?;
            o.sha256 = get_file_sha256(&mut f)?;
        }
        Ok(Box::new(o))
    }

    fn upload_file(path: store::PathConfig) -> Result<Vec<store::FileStoreResult>, Error> {
        todo!()
    }

    fn init_object_upload(path: store::PathConfig) -> Result<Vec<FileStoreResult>, Error> {
        todo!()
    }

    fn write_chunk(path: store::PathConfig) -> Result<Vec<FileStoreResult>, Error> {
        todo!()
    }

    fn complete_object_upload(path: store::PathConfig) -> Result<Vec<FileStoreResult>, Error> {
        todo!()
    }

    fn delete_object(path: store::PathConfig) -> Result<Vec<FileStoreResult>, Error> {
        todo!()
    }

    fn delete_objects(path: store::PathConfig) -> Result<Vec<FileStoreResult>, Error> {
        todo!()
    }
}

// get_file_sha256 calculates the sha256 of file
fn get_file_sha256(f: &mut File) -> Result<[u8; 32], Error> {
    let mut hasher = Sha256::new();
    let _n = io::copy(f, &mut hasher)?;
    let hash = hasher.finalize();
    // sh256 hash should have 32 bytes -> 8*32=256 bits
    let mut a: [u8; 32] = [0; 32];
    a.copy_from_slice(&hash[0..31]);
    Ok(a)
}
