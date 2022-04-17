use std::{path::Path, io::BufRead};

use anyhow::Error;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_s3::{ListObjectsV2Request, S3Client, S3, GetObjectRequest};
use structopt::StructOpt;
use tokio::io::AsyncReadExt;

use crate::store::{FileStore, FileStoreResult, StoreError};

#[derive(Debug, StructOpt)]
pub struct BucketOpt {
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    bucket: String,

    #[structopt(short, long)]
    verbose: bool,
}

pub struct S3Store {
    region: Region,
    s3: S3Client,
    bucket_name: String,
}

impl S3Store {
    pub fn new(bucket_name: &str, region: Region, s3: S3Client) -> Result<Self, Error> {
        Ok(S3Store {
            region,
            s3,
            bucket_name: bucket_name.to_string(),
        })
    }
}

#[async_trait]
impl FileStore for S3Store {
    async fn get_dir(&self, path: &str) -> Result<Vec<FileStoreResult>, Error> {
        let list_obj_opt = ListObjectsV2Request {
            bucket: self.bucket_name.clone(),
            prefix: Some(path.to_string()),
            ..Default::default()
        };
        let mut ret = Vec::<FileStoreResult>::new();
        let res = self.s3.list_objects_v2(list_obj_opt).await?;
        match res.contents {
            None => Ok(ret),
            Some(objs) => {
                for (i, obj) in objs.into_iter().enumerate() {
                    let p = Path::new(&obj.key.unwrap_or_default());
                    p.file_name();
                    p.is_dir();
                    p.metadata()?;
                    let o = FileStoreResult {
                        id: i.try_into()?,
                        name: if p.is_dir() {
                            "".to_string()
                        } else {
                            p.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        },
                        size: obj.size.unwrap_or_default().try_into()?,
                        path: if !p.is_dir() {
                            match p.parent() {
                                None => "".to_string(),
                                Some(x) => x.to_string_lossy().to_string(),
                            }
                        } else {
                            p.to_string_lossy().to_string()
                        },
                        file_type: p
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        is_dir: p.is_dir(),
                        modified: chrono::DateTime::from(p.metadata()?.modified()?),
                        modified_by: None,
                    };
                    ret.push(o);
                }
                Ok(ret)
            }
        }
    }
