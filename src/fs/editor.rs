use anyhow::{bail, Result};
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

pub fn try_get_env(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

pub struct Editor {
    editor: String,
}

impl Editor {
    pub fn new() -> Self {
        let editor = if let Some(e) = try_get_env("EDITOR") {
            e
        } else if let Some(e) = try_get_env("VISUAL") {
            e
        } else {
            String::from("nano")
        };

        Self { editor }
    }

    /// Edit the file at `path`.
    pub fn edit(&self, path: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.editor);
        cmd.arg(path);

        match cmd.status() {
            Ok(_) => Ok(()),
            Err(err) => bail!("error editing file {:?}: {}", &path, err),
        }
    }

    /// Edit ...
    pub fn edit_temp(&self, filename: &str, content: &[u8]) -> Result<Vec<u8>> {
        let mut path = env::temp_dir();
        path.push(filename);

        {
            // Write in block so file gets closed
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)?;
            file.write_all(content)?;
        }

        self.edit(&path)?;

        let mut buf = Vec::new();
        let mut file = OpenOptions::new().read(true).open(&path)?;
        file.read_to_end(&mut buf)?;

        // Remove temporary file.
        std::fs::remove_file(&path)?;

        Ok(buf)
    }
}
