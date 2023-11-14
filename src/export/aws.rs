use crate::{
    format::Output,
    fs::{self, TempFile},
    types::{Workspace, Workspaces},
};
use anyhow::{bail, Result};
use aws_sdk_s3::{primitives::ByteStream, types::ChecksumAlgorithm, Client};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufReader, path::PathBuf};

/*
Interesting topics:
- Using checksums/digest: https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/types/struct.Checksum.html
*/

/// Manifest tracks the exported files and their state.
/// This can be used to check if a file needs to be exported
/// by comparing the checksum/digest.
#[derive(Clone, Default, Serialize, Deserialize)]
struct Manifest {
    // The digest of the manifest itself.
    // If this hasn't changed on an export, no
    // files changed as well.
    // checksum: String,
    files: HashMap<String, String>,
}

impl Manifest {
    // Check if the given key (workspace/file) exists
    // in the manifest. If found it returns the digest
    // of the file.
    fn lookup(&self, key: &str) -> Option<&str> {
        match self.files.get(key) {
            Some(d) => Some(d.as_str()),
            None => None,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AwsS3Config {
    pub bucket: String,
    pub workspaces: Option<Vec<String>>,
}

pub struct AwsS3 {
    client: Client,
    config: AwsS3Config,
}

impl AwsS3 {
    pub async fn create(config: &AwsS3Config) -> Self {
        let cfg = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&cfg);
        Self {
            client,
            config: config.clone(),
        }
    }

    pub async fn export(&self, dryrun: bool, ws: Workspaces) -> Result<Output> {
        let workspaces = filter_workspaces(self.config.workspaces.as_ref(), &ws);
        if workspaces.is_empty() {
            return Ok(Output::EmptyExport);
        }

        let old_manifest = self.get_manifest().await?;
        let mut new_manifest = old_manifest.clone();

        let output = self
            .export_files(dryrun, &mut new_manifest, workspaces)
            .await?;

        self.upload_manifest(dryrun, &old_manifest, &new_manifest)
            .await?;

        Ok(output)
    }

    async fn export_files(
        &self,
        dryrun: bool,
        manifest: &mut Manifest,
        ws: HashMap<&String, &Workspace>,
    ) -> Result<Output> {
        let mut exported = Vec::new();
        let mut skipped = Vec::new();

        for (workspace_name, workspace) in ws {
            for file_entry in &workspace.files {
                let bytes = file_entry.read_bytes()?;
                let current_digest = fs::digest(&bytes)?;
                let key = format!("{}/{}", workspace_name, file_entry.filename());

                if let Some(digest) = manifest.lookup(&key) {
                    if digest == current_digest {
                        skipped.push(key.to_string());
                        continue;
                    }
                }

                manifest.files.insert(key.clone(), current_digest);

                if dryrun {
                    exported.push(key.to_string());
                    continue;
                }

                let body = ByteStream::from_path(file_entry.path()).await?;
                self.client
                    .put_object()
                    .bucket(&self.config.bucket)
                    .checksum_algorithm(ChecksumAlgorithm::Sha256)
                    .key(&key)
                    .body(body)
                    .send()
                    .await?;

                exported.push(key);
            }
        }

        Ok(Output::ExportResult { exported, skipped })
    }

    async fn upload_manifest(&self, dryrun: bool, old: &Manifest, new: &Manifest) -> Result<()> {
        let old: Vec<u8> = serde_json::to_string_pretty(&old)?.bytes().collect();
        let new: Vec<u8> = serde_json::to_string_pretty(&new)?.bytes().collect();
        if fs::digest(&old)? == fs::digest(&new)? {
            println!("Manifest matches old - skipping");
            return Ok(());
        }

        if dryrun {
            return Ok(());
        }

        println!("Uploading manifest to bucket {}", self.config.bucket);

        let path = PathBuf::from("manifest.json");
        let _temp_file = TempFile::create(&path, &new)?;

        let body = ByteStream::from_path(&path).await?;
        self.client
            .put_object()
            .bucket(&self.config.bucket)
            .key("manifest.json")
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    async fn get_manifest(&self) -> Result<Manifest> {
        let objects = self
            .client
            .list_objects()
            .bucket(&self.config.bucket)
            .send()
            .await?;
        if objects.contents.is_none() {
            return Ok(Manifest::default());
        }
        let res = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key("manifest.json")
            .send()
            .await;

        match res {
            Ok(object) => {
                let bytes = object.body.collect().await.map(|data| data.into_bytes())?;
                let bytes: Vec<u8> = bytes.into_iter().collect();
                let reader = BufReader::new(&bytes[..]);
                let manifest: Manifest = serde_json::from_reader(reader)?;
                Ok(manifest)
            }
            Err(err) => match err.into_service_error() {
                aws_sdk_s3::operation::get_object::GetObjectError::NoSuchKey(_) => {
                    Ok(Manifest::default())
                }
                _ => bail!(
                    "failed to get manifest from AWS S3 bucket {}",
                    self.config.bucket
                ),
            },
        }
    }
}

fn filter_workspaces<'a>(
    filter: Option<&Vec<String>>,
    ws: &'a HashMap<String, Workspace>,
) -> HashMap<&'a String, &'a Workspace> {
    match filter {
        Some(filter) => ws.iter().filter(|&(k, _)| filter.contains(k)).collect(),
        None => ws.iter().collect(),
    }
}
