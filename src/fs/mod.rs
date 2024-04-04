pub mod editor;
pub use editor::Editor;

use anyhow::{bail, Result};
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[allow(unused)]
pub fn digest(data: &[u8]) -> Result<String> {
    let mut context = Context::new(&SHA256);
    context.update(data);

    let digest = context.finish();
    let s = HEXLOWER.encode(digest.as_ref());
    Ok(s)
}

pub fn list_files(dir: &Path) -> Result<Vec<FileEntry>> {
    let entries = internal_list_dir(dir)?;
    let entries = entries.into_iter().filter(|e| !e.is_dir).collect();
    Ok(entries)
}

pub fn list_dirs(dir: &Path) -> Result<Vec<FileEntry>> {
    let entries = internal_list_dir(dir)?;
    let entries = entries.into_iter().filter(|e| e.is_dir).collect();
    Ok(entries)
}

pub fn read_file(path: &Path) -> Result<String> {
    let mut buf = String::new();
    let mut file = OpenOptions::new().read(true).open(path)?;
    file.read_to_string(&mut buf)?;
    Ok(buf)
}

#[derive(Clone)]
pub struct FileEntry {
    filepath: PathBuf,
    is_dir: bool,
}

impl fmt::Debug for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.filepath)
    }
}

impl FileEntry {
    pub fn new(path: &Path) -> Self {
        Self {
            filepath: PathBuf::from(path),
            is_dir: path.is_dir(),
        }
    }

    pub fn extension(&self) -> Option<String> {
        self.filepath
            .extension()
            .map(|ext| ext.to_string_lossy().into())
    }

    pub fn filename(&self) -> String {
        self.filepath
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    pub fn path(&self) -> &Path {
        &self.filepath
    }

    pub fn exists(&self) -> bool {
        self.filepath.exists()
    }

    pub fn push(&self, path: &str) -> Self {
        let root = self.filepath.join(path);
        Self::new(&root)
    }

    pub fn mkdir(&self) -> Result<()> {
        if self.filepath.exists() {
            return Ok(());
        }

        fs::create_dir_all(&self.filepath)?;
        Ok(())
    }

    pub fn read_bytes(&self) -> Result<Vec<u8>> {
        if self.is_dir {
            bail!("cannot read directory");
        }

        let mut buf: Vec<u8> = Vec::new();
        let mut file = OpenOptions::new().read(true).open(&self.filepath)?;
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

impl fmt::Display for FileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.filename())
    }
}

impl From<&str> for FileEntry {
    fn from(s: &str) -> Self {
        let p = PathBuf::from(s);
        Self::new(&p)
    }
}

fn internal_list_dir(dir: &Path) -> Result<Vec<FileEntry>> {
    let entries = fs::read_dir(dir)?;
    let entries = entries
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(_) => None,
        })
        .map(|e| {
            let filepath = e.path();
            let is_dir = filepath.is_dir();
            FileEntry { filepath, is_dir }
        })
        .collect();
    Ok(entries)
}

impl AsRef<Path> for FileEntry {
    fn as_ref(&self) -> &Path {
        &self.filepath
    }
}

pub struct TempFile {
    file_entry: FileEntry,
}

impl TempFile {
    #[allow(unused)]
    pub fn create(path: &Path, bytes: &[u8]) -> Result<Self> {
        let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
        file.write_all(bytes)?;

        Ok(Self {
            file_entry: FileEntry::new(path),
        })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.file_entry.exists() {
            let _ = fs::remove_file(self.file_entry.as_ref());
        }
    }
}
