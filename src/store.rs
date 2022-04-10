use core::fmt;
use std::{error, io};

use chrono;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
}

// convert underlying error to the custom store::Error
// https://doc.rust-lang.org/std/convert/trait.From.html
impl From<io::Error> for Error {
    fn from(e: io::ErrorKind) -> Self {
        Error::IoError(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IoError(err) => Some(err),
        }
    }
}

impl fmt::Display for FileStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "{}", err),
        }
    }
}

pub struct PathConfig {
    pub path: String,
    pub paths: Vec<String>,
}

pub struct FileStoreResult {
    id: u32,      // u32 is good - is there going to be more than 2**32 entries in a folder
    name: String, // use String for mutability and ownership
    size: i64,
    path: String,
    file_type: String,
    is_dir: bool,
    modified: chrono::NaiveDateTime,
}

pub trait FileStore {
    // Box<Error> can handle any error derived from std::error::Error
    fn get_dir(path: PathConfig) -> Result<Vec<FileStoreResult>, Box<dyn Error>>;
    fn get_obj(path: PathConfig) -> Result<Vec<FileStoreResult>, Box<dyn Error>>;
    fn put_obj(path: PathConfig) -> Result<Vec<FileStoreResult>, Box<dyn Error>>;
    fn upload_file(path: PathConfig) -> Result<Vec<FileStoreResult>, Box<dyn Error>>;
}
