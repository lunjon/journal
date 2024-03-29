use crate::cli::{
    Cli, Command, CreateArgs, ExportArgs, ListArgs, OpenArgs, RemoveArgs, SearchArgs,
};
use crate::config::Config;
use crate::export::zip;
use crate::format::{Output, TextFormatter};
use crate::fs::{list_dirs, list_files, read_lines, Editor, FileEntry};
use crate::template;
use crate::types::{Workspace, Workspaces};
use crate::validate::valid_workspace_name;
use anyhow::{bail, Result};
use crossterm::style::Stylize;

use regex::RegexBuilder;
use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Write};

type CmdResult = Result<()>;

pub struct Handler {
    config: Config,
    /// The root directory of all workspaces.
    /// On the filesystem it: {root_dir}/workspaces
    workspaces_dir: FileEntry,
    /// The default workspace directory.
    /// On the filesystem it: {root_dir}/{workspaces_dir}/{default_workspace_dir}
    default_workspace_dir: FileEntry,
    formatter: TextFormatter,
}

impl Handler {
    pub fn new() -> Result<Self> {
        let basedir = match directories::BaseDirs::new() {
            Some(basedir) => basedir,
            None => bail!("failed to resolver user base directories"),
        };

        let config_dir = FileEntry::new(basedir.config_dir()).push("journal");
        config_dir.mkdir()?;

        let config_path = config_dir.push("config.toml");
        let config = Config::load(&config_path)?;

        let root_dir = match &config.root {
            Some(d) => FileEntry::from(d.as_str()),
            None => FileEntry::new(basedir.data_dir()).push("journal"),
        };

        let workspaces_dir = root_dir.push("workspaces");

        let default_workspace = match &config.default_workspace {
            Some(w) => {
                valid_workspace_name(w)?;
                workspaces_dir.push(w)
            }
            None => workspaces_dir.push("default"),
        };

        Ok(Self {
            config,
            workspaces_dir,
            default_workspace_dir: default_workspace,
            formatter: TextFormatter {},
        })
    }

    pub fn handle(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Command::Open(args) => self.handle_open(args, false)?,
            Command::Print(args) => self.handle_open(args, true)?,
            Command::Create(args) => self.handle_create(args)?,
            Command::List(args) => self.handle_list(args)?,
            Command::Remove(args) => self.handle_remove(args)?,
            Command::Search(args) => self.handle_search(args)?,
            Command::Export(args) => self.handle_export(args)?,
            _ => bail!("unsupport here"),
        };

        Ok(())
    }

    fn handle_open(&self, args: OpenArgs, print: bool) -> CmdResult {
        let dir = self.get_workspace(&args.workspace);
        let filepath = self.find_journal(dir, &args.name)?;

        if !filepath.exists() {
            bail!("journal doesn't exists (hint: jn create --help)")
        }

        if print {
            let bytes = filepath.read_bytes()?;
            let mut stdout = stdout();
            stdout.write_all(&bytes)?;
            Ok(())
        } else {
            let editor = Editor::new();
            editor.open(filepath.as_ref())
        }
    }

    fn handle_create(&self, args: CreateArgs) -> CmdResult {
        let dir = self.get_workspace(&args.workspace);
        dir.mkdir()?;

        let filepath = dir.push(&args.name);
        if filepath.exists() {
            bail!(
                "filepath {} already exists (hint: jn open --help)",
                filepath
            );
        }

        let tmp = match filepath.extension() {
            None => None,
            Some(ext) => match &self.config.template {
                Some(templates) => templates.get(&ext),
                None => None,
            },
        };

        let content = template::create(tmp);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filepath.as_ref())?;
        write!(file, "{}", content)?;

        let editor = Editor::new();
        editor.open(filepath.as_ref())
    }

    fn handle_list(&self, args: ListArgs) -> CmdResult {
        let mut dirs: Vec<FileEntry> = Vec::new();

        if args.all {
            let ds = list_dirs(self.workspaces_dir.as_ref())?;
            dirs.extend(ds);
        } else {
            let d = self.get_workspace(&args.workspace);
            dirs.push(d);
        }

        for w in dirs {
            let entries = self.list_in_dir(&w)?;
            if !entries.is_empty() {
                let output = Output::WorkspaceJournals(w.filename(), entries);
                self.output(output);
            }
        }

        Ok(())
    }

    fn list_in_dir(&self, dir: &FileEntry) -> Result<Vec<FileEntry>> {
        if !dir.exists() {
            return Ok(vec![]);
        }

        let entries = list_files(dir.as_ref())?;
        Ok(entries)
    }

    fn find_journal(&self, dir: FileEntry, name: &str) -> Result<FileEntry> {
        let files = list_files(dir.as_ref())?;
        let mut matches: Vec<FileEntry> = files
            .into_iter()
            .filter(|entry| entry.filename().contains(name))
            .collect();

        if matches.is_empty() {
            bail!("no journal matching: {}", name);
        }

        if matches.len() == 1 {
            return Ok(matches.pop().unwrap());
        }

        let entry = inquire::Select::new("Select journal", matches).prompt()?;
        Ok(entry)
    }

    fn handle_remove(&self, args: RemoveArgs) -> CmdResult {
        let dir = self.get_workspace(&args.workspace);
        let filepath = dir.push(&args.name);

        if filepath.exists() {
            fs::remove_file(filepath.as_ref())?;
        } else {
            let err = format!(
                "journal named '{}' not found in workspace '{}'",
                args.name,
                dir.filename()
            );
            self.output_error(&err);
        }

        Ok(())
    }

    fn handle_search(&self, args: SearchArgs) -> CmdResult {
        let re = RegexBuilder::new(&args.pattern)
            .case_insensitive(args.case_insensitive)
            .build()?;

        let mut workspaces: Workspaces = Workspaces::new();
        match args.workspace {
            Some(w) => {
                let p = self.workspaces_dir.push(&w);
                let files = list_files(p.as_ref())?;
                workspaces.insert(w.to_string(), Workspace::new(w, files));
            }
            None => {
                let ws = self.list_workspaces_files()?;
                workspaces.extend(ws);
            }
        }

        for (name, workspace) in workspaces {
            for jn in workspace.files {
                let filename = jn.filename();
                let lines = read_lines(jn.as_ref())?;

                let matches: Vec<String> = lines
                    .iter()
                    .enumerate()
                    .filter(|(_, line)| re.is_match(line))
                    .map(|(num, line)| {
                        let linenum = format!("{}", num + 1);
                        format!("{}: {}", linenum.green(), line)
                    })
                    .collect();

                if !matches.is_empty() {
                    println!(
                        "{}/{}",
                        name.to_string().bold().magenta(),
                        filename.to_string().bold().magenta()
                    );

                    for line in matches {
                        println!("{}", line);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_export(&self, args: ExportArgs) -> CmdResult {
        let workspaces = self.list_workspaces_files()?;

        let output = match args.target.trim() {
            "zip" => zip::export(args.dryrun, args.dir, workspaces)?,
            target => bail!("unknown export target: {}", target),
        };

        self.output(output);

        Ok(())
    }
}

impl Handler {
    fn output(&self, output: Output) {
        println!("{}", self.formatter.format(output));
    }

    fn output_error(&self, err: &str) {
        eprintln!("{}: {}", "error".red(), err)
    }

    /// Creates a list of tuples with workspace name and it's files.
    fn list_workspaces_files(&self) -> Result<Workspaces> {
        let mut xs: Workspaces = HashMap::new();

        let dirs = list_dirs(self.workspaces_dir.as_ref())?;
        for d in dirs {
            let f = d.filename();
            let files = list_files(d.as_ref())?;
            xs.insert(f.to_string(), Workspace::new(f, files));
        }

        Ok(xs)
    }

    fn get_workspace(&self, workspace: &Option<String>) -> FileEntry {
        match &workspace {
            Some(w) => self.workspaces_dir.push(w),
            None => self.default_workspace_dir.clone(),
        }
    }
}
