use crate::fs::FileEntry;
use std::collections::HashMap;

/// A map of workspace names to workspaces.
/// A workspace consists of a list of files in
/// that particular workspace.
pub type Workspaces = HashMap<String, Workspace>;

// pub type WorkspacesRef<'a> = HashMap<&'a String, &'a Workspace>;

pub struct Workspace {
    /// Name of the workspace.
    pub name: String,
    /// The files (journals) in this workspace.
    pub files: Vec<FileEntry>,
}

impl Workspace {
    pub(crate) fn new(name: String, files: Vec<FileEntry>) -> Workspace {
        Self { name, files }
    }
}
