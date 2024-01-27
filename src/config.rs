use crate::fs::read_file;
use crate::fs::FileEntry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    /// Optional root of where to create journals.
    pub root: Option<String>,
    /// Optional default workspace. If not set the default
    /// workspace is used.
    #[serde(rename = "default-workspace")]
    pub default_workspace: Option<String>,
    pub template: Option<HashMap<String, String>>,
}

impl Config {
    pub fn load(path: &FileEntry) -> Result<Self> {
        let config = if path.exists() {
            let content = read_file(path.path())?;
            let config: Config = toml::from_str(&content)?;
            config
        } else {
            Self::default()
        };

        Ok(config)
    }
}
