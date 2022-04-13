use std::io::BufRead;

use anyhow::Error;
use bytes::Bytes;
use chrono::{self, Utc};
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

pub struct FileOperationOutput {
    pub sha256: [u8; 32], // sha256 is unbroken as of Jan 2022
}

pub struct FileStoreResult {
    pub id: u32, // u32 is good - is there going to be more than 2**32 entries in a folder
    pub name: String, // use String for mutability and ownership
    pub size: u64,
    pub path: String,
    pub file_type: String,
    pub is_dir: bool,
    pub modified: chrono::DateTime<Utc>,
    pub modified_by: String,
}

pub trait FileStore {
    // Box<Error> can handle any error derived from std::error::Error
    fn get_dir(path: &str) -> Result<Vec<FileStoreResult>, Error>;
    fn get_object(path: &str) -> Result<Box<dyn BufRead>, Error>;
    fn put_object(path: &str, data: Bytes) -> Result<Box<FileOperationOutput>, Error>;
    fn upload_file(path: &str) -> Result<Vec<FileStoreResult>, Error>;
    fn init_object_upload(path: &str) -> Result<Vec<FileStoreResult>, Error>;
    fn write_chunk(path: &str) -> Result<Vec<FileStoreResult>, Error>;
    fn file_sha256sum(path: &str) -> Result<Box<FileOperationOutput>, Error>;
    fn delete_object(path: &str) -> Result<(), Error>;
    fn delete_objects(path: Vec<String>) -> Result<(), Error>;
    fn walk(path: &str, visit_fn: fn(path: DirEntry) -> Result<(), Error>) -> Result<(), Error>;
}
