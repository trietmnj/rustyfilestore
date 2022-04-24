use std::collections::HashMap;

use anyhow::Error;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{self, Utc};
use uuid::Uuid;
use walkdir::DirEntry;

// #[derive(Debug)]
// pub enum Error {
//     IoError(io::Error),
// }

// // convert underlying error to the custom store::Error
// // https://doc.rust-lang.org/std/convert/trait.From.html
// impl From<io::Error> for Error {
//     fn from(e: io::ErrorKind) -> Self {
//         Error::IoError(e)
//     }
// }

// impl error::Error for Error {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         match self {
//             Error::IoError(err) => Some(err),
//         }
//     }
// }

// impl fmt::Display for FileStoreError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Error::IoError(err) => write!(f, "{}", err),
//         }
//     }
// }

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("s3 access error")]
    S3Error,
}

pub struct FileOperationOutput {
    pub md5: [u8; 16],
}

pub struct FileStoreResult {
    pub id: u64,
    pub name: String, // use String for mutability and ownership
    pub size: u64,
    pub path: String,
    pub file_type: String,
    pub is_dir: bool,
    pub modified: chrono::DateTime<Utc>,
    pub modified_by: Option<String>,
}

pub struct UploadConfig {
    pub object_path: String,
    pub chunk_id: i64,
    pub upload_id: String,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct UploadResult {
    pub id: String,
    pub write_size: usize, // # of bytes written
    pub is_complete: bool,
}

#[async_trait]
pub trait FileStore {
    // Box<Error> can handle any error derived from std::error::Error
    async fn get_dir(&self, path: &str) -> Result<Vec<FileStoreResult>, Error>;
    // Vec<u8> is can used as a container of bytes
    async fn get_object(&self, path: &str) -> Result<Vec<u8>, Error>;
    async fn put_object(&self, path: &str, data: Vec<u8>, metadata: Option<HashMap<String,String>>) -> Result<FileOperationOutput, Error>;
    async fn init_object_upload(&self, u: UploadConfig) -> Result<UploadResult, Error>;
    async fn write_chunk(&self, u: UploadConfig) -> Result<UploadResult, Error>;
    fn file_md5sum(path: &str) -> Result<Box<FileOperationOutput>, Error>;
    fn delete_object(path: &str) -> Result<(), Error>;
    fn delete_objects(path: Vec<String>) -> Result<(), Error>;
    fn walk(path: &str, visit_fn: fn(path: DirEntry) -> Result<(), Error>) -> Result<(), Error>;
}
