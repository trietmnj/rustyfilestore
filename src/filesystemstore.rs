use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{self, create_dir, create_dir_all, metadata, remove_dir_all, remove_file, File},
    io::{self, BufReader, Read, Write},
    path::PathBuf,
};

use crate::store::{self, FileOperationOutput, FileStoreResult, UploadConfig, UploadResult};
use anyhow::{Error, Result};
use async_trait::async_trait;
use md5::Md5;
use sha2::Digest;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

pub struct FileSystemStore {}

impl FileSystemStore {
    pub fn new(&self) -> Self {
        FileSystemStore {}
    }
}

#[async_trait]
impl store::FileStore for FileSystemStore {
    async fn get_dir(&self, path: &str) -> Result<Vec<store::FileStoreResult>, Error> {
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
                id: u64::try_from(i).unwrap(),
                name: p.file_name().to_string_lossy().to_string(),
                size: meta.len(),
                path: p.path().to_string_lossy().to_string(),
                file_type,
                is_dir: meta.is_dir(),
                modified: chrono::DateTime::from(meta.modified()?),
                modified_by: None,
            };
            v.push(r);
        }
        Ok(v)
    }

    async fn get_object(&self, path: &str) -> Result<Vec<u8>, Error> {
        let buf = Vec::<u8>::new();
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        let bytes = reader.read_to_end(&mut buf)?;
        Ok(buf)
    }

    async fn put_object(
        &self,
        path: &str,
        data: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<FileOperationOutput, Error> {
        let mut o = FileOperationOutput { md5: [0; 16] };
        if data.len() == 0 {
            // no data -> create dir
            create_dir(path)?;
        } else {
            // create file with data
            let mut f = File::create(path)?;
            f.write_all(&data)?;
            o.md5 = get_file_md5sum(&mut f)?;
        }
        Ok(o)
    }

    // init_object_upload creates folder and file
    async fn init_object_upload(&self, u: UploadConfig) -> Result<UploadResult, Error> {
        let mut res = UploadResult::default();
        let path = PathBuf::from(u.object_path);
        let dir = path.parent();
        match dir {
            Some(x) => create_dir_all(x)?,
            None => (),
        }
        File::create(path)?;
        res.id = Uuid::new_v4().to_string();
        Ok(res)
    }

    async fn write_chunk(&self, u: UploadConfig) -> Result<UploadResult, Error> {
        let mut res = UploadResult::default();
        let f = File::options().write(true).open(u.object_path)?;
        res.write_size = f.write(&u.data)?;
        if res.write_size == u.data.len() {
            res.is_complete = true;
        }
        Ok(res)
    }

    // fn file_sha256sum(path: &str) -> Result<Box<FileOperationOutput>, Error> {
    //     let mut o = FileOperationOutput { sha256: [0; 32] };
    //     let mut f = File::create(path)?;
    //     o.sha256 = get_file_sha256(&mut f)?;
    //     Ok(Box::new(o))
    // }

    fn file_md5sum(path: &str) -> Result<Box<FileOperationOutput>, Error> {
        let mut o = FileOperationOutput { md5: [0; 16] };
        let mut f = File::create(path)?;
        o.md5 = get_file_md5sum(&mut f)?;
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

// get_file_md5sum calculates the sha256sum of file
fn get_file_md5sum(f: &mut File) -> Result<[u8; 16], Error> {
    let mut hasher = Md5::new();
    let _n = io::copy(f, &mut hasher)?;
    let hash = hasher.finalize();
    // sh256 hash should have 32 bytes -> 8*32=256 bits
    let mut a: [u8; 16] = [0; 16];
    a.copy_from_slice(&hash[0..15]);
    Ok(a)
}

// get_file_sha256 calculates the sha256sum of file
// fn get_file_sha256(f: &mut File) -> Result<[u8; 32], Error> {
//     let mut hasher = Sha256::new();
//     let _n = io::copy(f, &mut hasher)?;
//     let hash = hasher.finalize();
//     // sh256 hash should have 32 bytes -> 8*32=256 bits
//     let mut a: [u8; 32] = [0; 32];
//     a.copy_from_slice(&hash[0..31]);
//     Ok(a)
// }
