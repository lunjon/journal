use crate::{format::Output, fs::FileEntry, types::Workspaces, util::get_date};
use anyhow::Result;
use std::{fs::OpenOptions, io::Write};

pub fn export(dryrun: bool, dir: Option<String>, ws: Workspaces) -> Result<Output> {
    let dir = match dir {
        Some(dir) => FileEntry::from(dir.as_str()),
        None => FileEntry::from("."),
    };

    let filename = format!("journals.{}.zip", get_date());
    let filepath = dir.push(&filename);

    let zipfile_name = format!("{}", filepath);
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .truncate(true)
        .open(zipfile_name)?;

    let mut zip = zip::ZipWriter::new(&mut file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut exported: Vec<String> = Vec::new();

    for (ws_name, ws) in ws {
        zip.add_directory(&ws_name, options)?;

        for file_entry in ws.files {
            let filename = format!("{}/{}", ws_name, file_entry.filename());
            exported.push(filename.clone());

            if dryrun {
                continue;
            }

            zip.start_file(filename, options)?;
            let bytes = file_entry.read_bytes()?;
            zip.write_all(&bytes)?;
        }
    }

    zip.finish()?;

    Ok(Output::ExportResult {
        exported,
        skipped: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fs::FileEntry, types::Workspace};
    use std::path::PathBuf;

    struct Fixture {
        dir: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let dir = PathBuf::from("./temptestdir");
            let _ = std::fs::create_dir_all(&dir);
            Self { dir }
        }

        fn dirstr(&self) -> String {
            self.dir.to_string_lossy().to_string()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.dir.exists() {
                let _ = std::fs::remove_dir_all(&self.dir);
            }
        }
    }

    #[test]
    fn exporting_zip() -> Result<()> {
        // Arrange
        let fx = Fixture::new();
        let file = FileEntry::new(&PathBuf::from("testdata/rust.md"));
        let files = vec![file];
        let workspace = Workspace::new("testdata".to_string(), files);
        let mut workspaces = Workspaces::new();
        workspaces.insert("testdata".to_string(), workspace);

        // Act
        export(false, Some(fx.dirstr()), workspaces)?;

        Ok(())
    }
}
