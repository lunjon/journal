use anyhow::{bail, Result};
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

    pub fn open(&self, path: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.editor);
        // cmd.arg(self.path.to_str().unwrap());
        cmd.arg(path);

        match cmd.status() {
            Ok(_) => Ok(()),
            Err(err) => bail!("error editing file {:?}: {}", &path, err),
        }
    }
}
