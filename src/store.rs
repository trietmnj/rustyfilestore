use chrono::{self, Utc};
use anyhow::Error;

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

pub struct PathConfig {
    pub path: String,
    pub paths: Vec<String>,
}

pub struct FileStoreResult {
    pub id: u32,      // u32 is good - is there going to be more than 2**32 entries in a folder
    pub name: String, // use String for mutability and ownership
    pub size: u64,
    pub path: String,
    pub file_type: String,
    pub is_dir: bool,
    pub modified: chrono::DateTime<Utc>,
}

pub trait FileStore {
    // Box<Error> can handle any error derived from std::error::Error
    fn get_dir(path: PathConfig) -> Result<Vec<FileStoreResult>, Error>;
    fn get_obj(path: PathConfig) -> Result<Vec<FileStoreResult>, Error>;
    fn put_obj(path: PathConfig) -> Result<Vec<FileStoreResult>, Error>;
    fn upload_file(path: PathConfig) -> Result<Vec<FileStoreResult>, Error>;
}
