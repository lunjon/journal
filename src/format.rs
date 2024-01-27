use crate::fs::FileEntry;
use crossterm::style::Stylize;

/// Output represents things that can be presented to the
/// user in various formats.
pub enum Output {
    /// List of shallow representations of journals.
    WorkspaceJournals(String, Vec<FileEntry>),
    /// The result of performing an export.
    ExportResult {
        exported: Vec<String>,
        skipped: Vec<String>,
    },
}

pub struct TextFormatter {}

impl TextFormatter {
    pub fn format(&self, output: Output) -> String {
        match output {
            Output::WorkspaceJournals(wrk_sp, entries) => {
                let lines: Vec<String> = entries
                    .iter()
                    .map(|entry| format!("    {}", entry.filename()))
                    .collect();
                format!("{}/\n{}", wrk_sp.bold(), lines.join("\n"))
            }
            Output::ExportResult {
                exported: synced,
                skipped,
            } => {
                let synced: Vec<String> = synced
                    .iter()
                    .map(|entry| format!("  {}", entry.to_string().green()))
                    .collect();
                let skipped: Vec<String> = skipped
                    .iter()
                    .map(|entry| format!("  {} ", entry.to_string().blue()))
                    .collect();

                let mut lines: Vec<String> = Vec::new();

                if !synced.is_empty() {
                    lines.push("Exported files:".to_string());
                    lines.extend(synced);
                }
                if !skipped.is_empty() {
                    lines.push("Skipped files:".to_string());
                    lines.extend(skipped);
                }

                lines.join("\n")
            }
        }
    }
}
