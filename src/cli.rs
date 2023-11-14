use crate::validate::valid_workspace_name;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Starts the REPL.
    #[command()]
    Repl,
    /// Opens an existing journal.
    #[command(visible_alias = "o")]
    Open(OpenArgs),
    /// Creates a new journal.
    #[command(visible_alias = "c")]
    Create(CreateArgs),
    /// Prints a journal to stdout.
    Print(OpenArgs),
    /// Lists journals.
    #[command(visible_alias = "ls")]
    List(ListArgs),
    /// Removes a journal.
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),
    /// Search in your journals.
    /// Searches across all workspaces by default.
    #[command()]
    Search(SearchArgs),
    /// Export journals.
    #[command()]
    Export(ExportArgs),
}

#[derive(Args)]
pub struct OpenArgs {
    /// Name of the journal to open.
    /// You can also try to open a file by pattern with the `--matches` option.
    #[arg()]
    pub name: Option<String>,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
    /// Opens the FIRST journal matching this name.
    #[arg(long, short, conflicts_with = "name")]
    pub matches: Option<String>,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Name of the journal to open.
    #[arg()]
    pub name: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// The name of the journal to remove.
    #[arg()]
    pub name: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// List all journals across all workspaces.
    #[arg(long, short = 'a')]
    pub all: bool,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
}

#[derive(Args)]
pub struct SearchArgs {
    /// Regular expression to search for in the journals.
    pub pattern: String,
    /// Ignore case when searching.
    #[arg(long, short = 'i')]
    pub case_insensitive: bool,
    /// Optional workspace to use, else search across all workspaces.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
}

#[derive(Args)]
pub struct ExportArgs {
    /// The target to use for exporting.
    #[arg(long, short, value_parser = ["aws-s3", "zip"])]
    pub target: String,
    /// Perform a dry-run, i.e. do not actually export the files.
    #[arg(long = "dry-run")]
    pub dryrun: bool,
    /// Output the results to a directory.
    /// Defaults to current working directory.
    #[arg(long, short)]
    pub dir: Option<String>,
}
