use crate::crypto;
use crate::fs::{Editor, FileEntry};
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

/// A map of workspace names to workspaces.
/// A workspace consists of a list of files in
/// that particular workspace.
pub type Workspaces = HashMap<String, Workspace>;

pub struct Workspace {
    /// Name of the workspace.
    pub name: String,
    /// The files (journals) in this workspace.
    pub files: Vec<FileEntry>,
}

impl Workspace {
    pub fn new(name: String, files: Vec<FileEntry>) -> Workspace {
        Self { name, files }
    }
}

/// Header is the first couple of bytes in a journal written to disk.
/// The first byte is the version and currently the only version is
/// 0x01. If not such version is found as the first byte, it is assumed
/// to be a prior version. That is, flags is not supported.
///
/// After the version comes the following.
///
/// encrypted─┐
///           00000000[nonce:12 bytes]
///
/// If `encrypted` was set the following two bytes are:
///   - nonce length in bytes
///   - tag length in bytes
/// Then those two bytes are immediately followed by
/// the nonce and tag, respectively.
///
/// Then the actual content starts.
/// If no encryption is set, the content starts immediately.
struct Header {
    /// Size of the header in bytes.
    size: usize,
    version: u8,
    /// Whether this file was encrypted or not.
    /// FIXME: remove this as have a field called `flags` (u8) instead.
    encrypted: bool,
    /// The nonce bytes. Empty if not encrypted.
    nonce: Vec<u8>,
    /// Authentication tag used when encrypting/decrypting.
    /// Empty if not encrypted.
    tag: Vec<u8>,
}

impl Header {
    fn v0() -> Self {
        Self {
            version: 0x00,
            encrypted: false,
            nonce: vec![],
            tag: vec![],
            size: 0,
        }
    }

    fn new_non_encrypted() -> Self {
        Self {
            size: 2,
            version: 0x01,
            encrypted: false,
            nonce: vec![],
            tag: vec![],
        }
    }

    fn new_encrypted(nonce: Vec<u8>, tag: Vec<u8>) -> Self {
        Self {
            size: 4 + nonce.len() + tag.len(),
            version: 0x01,
            encrypted: true,
            nonce,
            tag,
        }
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut buf = Vec::with_capacity(self.size);
        buf.push(self.version);

        let mut flags = 0x00;

        if self.encrypted {
            flags |= 0x80; // Set the encrypted bit (MSB)
        }

        buf.push(flags);

        if self.encrypted {
            buf.push(self.nonce.len() as u8);
            buf.push(self.tag.len() as u8);
            buf.extend_from_slice(&self.nonce);
            buf.extend_from_slice(&self.tag);
        }

        writer.write_all(&buf)?;

        Ok(())
    }

    fn decode(value: &[u8]) -> Result<Self> {
        let version = match value.first() {
            Some(b) => *b,
            None => return Ok(Header::v0()),
        };

        if version != 0x01 {
            return Ok(Header::v0());
        }

        let flags = match value.get(1) {
            Some(b) => *b,
            None => return Ok(Header::v0()),
        };

        let mut size = 2;
        let mut nonce: Vec<u8> = vec![];
        let mut tag: Vec<u8> = vec![];
        let mut encrypted = false;

        // Is the MSB set?
        if (flags & 0x80) == 0x80 {
            encrypted = true;

            // File was encrypted.
            // The next byte is the size of the nonce in bytes.
            let nonce_size = value
                .get(2)
                .context("failed to decode header: missing nonce size")?;
            let nonce_size = *nonce_size as usize;

            // The next byte is the size of the tag in bytes.
            let tag_size = value
                .get(3)
                .context("failed to decode header: missing tag size")?;
            let tag_size = *tag_size as usize;

            size += 2;

            nonce.extend_from_slice(&value[size..(size + nonce_size)]);
            size += nonce_size;

            tag.extend_from_slice(&value[size..(size + tag_size)]);
            size += tag_size;
        }

        Ok(Self {
            size,
            version,
            encrypted,
            nonce,
            tag,
        })
    }
}

pub struct Journal {
    filepath: FileEntry,
    key: Option<String>,
    header: Header,
    contents: Vec<u8>,
}

impl Journal {
    pub fn create(filepath: &FileEntry, key: Option<String>, content: &[u8]) -> Result<()> {
        let editor = Editor::new();

        let filename = filepath.filename();
        let content = editor.edit_temp(&filename, content)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filepath.as_ref())?;

        Self::write(&mut file, key.as_ref(), &content)?;

        Ok(())
    }

    pub fn open(file_entry: &FileEntry, key: Option<String>) -> Result<Self> {
        let contents = file_entry.read_bytes()?;
        let header = Header::decode(contents.as_ref())?;
        Ok(Self {
            filepath: file_entry.clone(),
            key,
            header,
            contents,
        })
    }

    pub fn bytes(&self) -> Result<Vec<u8>> {
        if self.header.encrypted {
            self.decrypt()
        } else {
            let data = &self.contents[self.header.size..];
            let mut bs = Vec::with_capacity(data.len());
            bs.extend_from_slice(data);
            Ok(bs)
        }
    }

    pub fn edit(&self) -> Result<()> {
        let editor = Editor::new();
        let content = self.bytes()?;

        let filename = self.filepath.filename();
        let content = editor.edit_temp(&filename, &content)?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.filepath.as_ref())?;
        Self::write(&mut file, self.key.as_ref(), &content)?;

        Ok(())
    }

    fn write<W: Write>(writer: &mut W, key: Option<&String>, content: &[u8]) -> Result<()> {
        if let Some(key) = &key {
            // When writing the file it may not be encrypted before,
            // so the header must be updated accordingly.
            let res = crypto::encrypt(content, key)?;
            let header = Header::new_encrypted(res.nonce, res.tag);
            header.encode(writer)?;

            writer.write_all(&res.ciphertext)?;
        } else {
            let header = Header::new_non_encrypted();
            header.encode(writer)?;
            writer.write_all(content)?;
        }

        Ok(())
    }

    fn decrypt(&self) -> Result<Vec<u8>> {
        let key = self.require_key()?;

        let data = &self.contents[self.header.size..];
        let plaintext = crypto::decrypt(
            key,
            self.header.nonce.as_ref(),
            self.header.tag.as_ref(),
            data,
        )?;

        Ok(plaintext)
    }

    fn require_key(&self) -> Result<&str> {
        match &self.key {
            Some(key) => Ok(key.as_str()),
            None => bail!("key required for encrypted file"),
        }
    }
}
