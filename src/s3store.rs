#[allow(dead_code)]
use std::{collections::HashMap, path::Path};

use anyhow::Error;
use async_trait::async_trait;
use rusoto_core::{ByteStream, Region};
use rusoto_s3::{
    CreateMultipartUploadRequest, GetObjectRequest, ListObjectsV2Request, PutObjectRequest,
    S3Client, UploadPartRequest, S3,
};
use structopt::StructOpt;
use tokio::io::AsyncReadExt;

use crate::store::{
    FileOperationOutput, FileStore, FileStoreResult, StoreError, UploadConfig, UploadResult,
};

#[derive(Debug, StructOpt)]
pub struct BucketOpt {
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    bucket: String,

    #[structopt(short, long)]
    verbose: bool,
}

struct S3Store {
    region: Region,
    s3: S3Client,
    bucket_name: String,
}

impl S3Store {
    pub fn new(bucket_name: &str, region: Region, s3: S3Client) -> Self {
        S3Store {
            region,
            s3,
            bucket_name: bucket_name.to_string(),
        }
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
                                Some(x) => x.to_string_lossy().into(),
                            }
                        } else {
                            p.to_string_lossy().into()
                        },
                        file_type: p.extension().unwrap_or_default().to_string_lossy().into(),
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

    async fn get_object(&self, path: &str) -> Result<Vec<u8>, anyhow::Error> {
        // https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/s3.rs
        let get_obj_req = GetObjectRequest {
            bucket: self.bucket_name,
            key: path.to_string(),
            ..Default::default()
        };
        let res = self.s3.get_object(get_obj_req).await?;
        let mut stream = match res.body {
            None => Err(StoreError::S3Error),
            Some(x) => {
                let mut buf = Vec::new();
                let mut reader = x.into_async_read();
                let bytes = reader.read_to_end(&mut buf).await?;
                Ok(buf)
            }
        };
        Ok(stream?)
    }

    async fn put_object(
        &self,
        path: &str,
        data: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<FileOperationOutput, anyhow::Error> {
        let put_request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: path.to_owned(),
            body: Some(data.into()),
            metadata,
            ..Default::default()
        };
        let res = self.s3.put_object(put_request).await?;
        let md5sum = res.e_tag.unwrap_or("".into());
        let mut s: [u8; 16] = Default::default();
        s.copy_from_slice(md5sum.as_bytes());
        Ok(FileOperationOutput { md5: s })
    }

    async fn init_object_upload(&self, u: UploadConfig) -> Result<UploadResult, Error> {
        let upload_input = CreateMultipartUploadRequest {
            bucket: self.bucket_name,
            key: u.object_path.strip_prefix("/").unwrap_or("").to_string(),
            ..Default::default()
        };
        let res = self.s3.create_multipart_upload(upload_input).await?;
        let id = res.upload_id.unwrap_or("".to_string());
        let mut o = UploadResult {
            id,
            ..Default::default()
        };
        Ok(o)
    }

    async fn write_chunk(&self, u: UploadConfig) -> Result<UploadResult, Error> {
        let input = UploadPartRequest {
            bucket: self.bucket_name,
            key: u.object_path.strip_prefix("/").unwrap_or("").to_string(),
            body: Some(ByteStream::from(u.data)),
            ..Default::default()
        };
        let res = self.s3.upload_part(input).await?;
        Ok(UploadResult{
            id: res.e_tag.unwrap_or("".to_string()),
            write_size: u.data.len(),
            is_complete: false,
        })
    }

    fn file_md5sum(path: &str) -> Result<Box<crate::store::FileOperationOutput>, anyhow::Error> {
        todo!()
    }

    fn delete_object(path: &str) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn delete_objects(path: Vec<String>) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn walk(
        path: &str,
        visit_fn: fn(path: walkdir::DirEntry) -> Result<(), anyhow::Error>,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
}
