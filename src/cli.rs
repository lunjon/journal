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
    /// Removes a journal or workspace.
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),
    /// Renames a journal or workspace.
    #[command(visible_alias = "mv")]
    Rename(RenameArgs),
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
    /// Name of the journal to open. Can be part of the name.
    /// Opens first match or if there's multiple matches it
    /// queries the user for a match.
    #[arg()]
    pub name: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
    /// Use as key for decryption. NOTE: when supplying a key
    /// on a journal which is not prior encrypted it will be encrypted
    /// after specifying a key.
    #[arg(long, short = 'k')]
    pub key: Option<String>,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Name of the journal to create.
    #[arg()]
    pub name: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
    /// Encrypt the journal using this key.
    /// The key have length 8 <= key <= 32;
    #[arg(long, short = 'k')]
    pub key: Option<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// The name of the journal to remove.
    #[arg()]
    pub name: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
    /// Remove `name` as a workspace instead of a journal.
    #[arg(long, conflicts_with = "workspace")]
    pub remove_workspace: bool,
}

#[derive(Args)]
pub struct RenameArgs {
    /// The name of the journal to rename.
    #[arg()]
    pub old: String,
    #[arg()]
    pub new: String,
    /// Optional workspace to use, else use the default workspace.
    #[arg(long, short = 'w', value_parser = valid_workspace_name)]
    pub workspace: Option<String>,
    /// Rename `name` as a workspace instead of a journal.
    #[arg(long, conflicts_with = "workspace")]
    pub rename_workspace: bool,
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
    /// Use as key for decryption.
    /// If this is omitted encrypted files will be skipped.
    #[arg(long, short = 'k')]
    pub key: Option<String>,
}

#[derive(Args)]
pub struct ExportArgs {
    /// The target to use for exporting.
    #[arg(long, short, value_parser = ["zip"])]
    pub target: String,
    /// Output the results to a directory.
    /// Defaults to current working directory.
    #[arg(long, short)]
    pub dir: Option<String>,
    /// Use as key for decryption.
    /// If this is omitted encrypted files will be skipped.
    #[arg(long, short = 'k')]
    pub key: Option<String>,
}
