use std::{
    ffi::OsString,
    fs::{self, create_dir, metadata, remove_dir_all, remove_file, File, create_dir_all},
    io::{self, BufRead, BufReader, Write}, path::PathBuf,
};

use crate::store::{self, FileOperationOutput, FileStoreResult, UploadConfig, UploadResult};
use anyhow::{Error, Result};
use bytes::Bytes;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

pub struct FileSystemStore {}

impl FileSystemStore {
    pub fn new(&self) -> Self {
        FileSystemStore {}
    }
}

impl store::FileStore for FileSystemStore {
    fn get_dir(path: &str) -> Result<Vec<store::FileStoreResult>, Error> {
        let entries = fs::read_dir(path)?;
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

    fn get_object(path: &str) -> Result<Box<dyn BufRead>, Error> {
        let f = File::open(path)?;
        Ok(Box::new(BufReader::new(f)))
    }

    fn put_object(path: &str, data: Bytes) -> Result<Box<FileOperationOutput>, Error> {
        let mut o = FileOperationOutput { sha256: [0; 32] };
        if data.len() == 0 {
            // no data -> create dir
            create_dir(path)?;
        } else {
            // create file with data
            let mut f = File::create(path)?;
            f.write_all(&data)?;
            o.sha256 = get_file_sha256(&mut f)?;
        }
        Ok(Box::new(o))
    }

    fn upload_file(_path: &str) -> Result<Vec<store::FileStoreResult>, Error> {
        todo!()
    }

    // init_object_upload creates folder and file
    fn init_object_upload(u: UploadConfig) -> Result<UploadResult, Error> {
        let mut res = UploadResult::default();
        let path = PathBuf::from(u.object_path);
        let dir = path.parent();
        match dir {
            Some(x) => create_dir_all(x)?,
            None => (),
        }
        File::create(path)?;
        res.id = Uuid::new_v4();
        Ok(res)
    }

    fn write_chunk(u: UploadConfig) -> Result<UploadResult, Error> {
        let mut res = UploadResult::default();
        let f = File::options().write(true).open(u.object_path)?;
        res.write_size = f.write(&u.data)?;
        if res.write_size == u.data.len() {
            res.is_complete = true;
        }
        Ok(res)
    }

    fn file_sha256sum(path: &str) -> Result<Box<FileOperationOutput>, Error> {
        let mut o = FileOperationOutput { sha256: [0; 32] };
        let mut f = File::create(path)?;
        o.sha256 = get_file_sha256(&mut f)?;
        Ok(Box::new(o))
    }

    fn delete_object(path: &str) -> Result<()> {
        if metadata(path)?.is_dir() {
            remove_dir_all(path)?;
        } else {
            remove_file(path)?;
        }
        Ok(())
    }

    fn delete_objects(paths: Vec<String>) -> Result<()> {
        for path in paths {
            FileSystemStore::delete_object(path.as_str())?;
        }
        Ok(())
    }

    fn walk(path: &str, visit_fn: fn(dir: DirEntry) -> Result<()>) -> Result<()> {
        for entry in WalkDir::new(path) {
            visit_fn(entry?)?;
        }
        Ok(())
    }
}

// get_file_sha256 calculates the sha256sum of file
fn get_file_sha256(f: &mut File) -> Result<[u8; 32], Error> {
    let mut hasher = Sha256::new();
    let _n = io::copy(f, &mut hasher)?;
    let hash = hasher.finalize();
    // sh256 hash should have 32 bytes -> 8*32=256 bits
    let mut a: [u8; 32] = [0; 32];
    a.copy_from_slice(&hash[0..31]);
    Ok(a)
}
