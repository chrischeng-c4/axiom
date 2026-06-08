//! Query the migrations tracking table and format the status for stdout.

use super::{MigrationEntry, MigrationRunner, MigrationSource};
use crate::{Connection, Result};
use std::path::Path;

// -- MigrationStatusReport ----------------------------------------------------

/// Snapshot of the migration state: applied entries + pending IDs.
#[derive(Debug, Clone)]
pub struct MigrationStatusReport {
    /// All entries currently in the migrations tracking table, sorted by ID.
    pub applied: Vec<MigrationEntry>,
    /// Migration IDs present on disk but absent from the tracking table.
    pub pending: Vec<String>,
}

impl MigrationStatusReport {
    /// Build a status snapshot by comparing the tracking table against the
    /// SQL files on disk.
    pub async fn load(conn: Connection, migrations_dir: &Path) -> Result<Self> {
        let runner = MigrationRunner::new(conn, None);
        runner.init().await?;

        let applied = runner.all_entries().await?;
        let applied_ids: std::collections::HashSet<_> =
            applied.iter().map(|e| e.migration_id.clone()).collect();

        // Load SQL files from disk (if the directory exists)
        let pending = if migrations_dir.exists() {
            MigrationRunner::load_from_directory(migrations_dir)
                .unwrap_or_default()
                .into_iter()
                .filter(|m| !applied_ids.contains(&m.version))
                .map(|m| m.version)
                .collect()
        } else {
            Vec::new()
        };

        Ok(Self { applied, pending })
    }

    /// Format the status as a human-readable table for stdout.
    pub fn to_table(&self) -> String {
        let mut lines = Vec::new();

        lines.push("Migration Status".to_string());
        lines.push("===============".to_string());
        lines.push(format!(
            "Applied: {}  (legacy: {}, native: {})",
            self.applied.len(),
            self.applied
                .iter()
                .filter(|e| e.source == MigrationSource::Legacy)
                .count(),
            self.applied
                .iter()
                .filter(|e| e.source == MigrationSource::Native)
                .count(),
        ));
        lines.push(format!("Pending: {}", self.pending.len()));
        lines.push(String::new());

        if !self.applied.is_empty() {
            lines.push(format!(
                "{:<35}  {:<10}  {}",
                "MIGRATION ID", "SOURCE", "APPLIED AT"
            ));
            lines.push(format!("{:-<35}  {:-<10}  {:-<25}", "", "", ""));

            for entry in &self.applied {
                lines.push(format!(
                    "{:<35}  {:<10}  {}",
                    entry.migration_id,
                    entry.source.as_str(),
                    entry.applied_at.format("%Y-%m-%d %H:%M:%S UTC"),
                ));
            }
            lines.push(String::new());
        }

        if !self.pending.is_empty() {
            lines.push("Pending:".to_string());
            for id in &self.pending {
                lines.push(format!("  [ ] {}", id));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(id: &str, source: MigrationSource) -> MigrationEntry {
        MigrationEntry {
            migration_id: id.to_string(),
            source,
            applied_at: Utc::now(),
            checksum: "abc".to_string(),
        }
    }

    #[test]
    fn status_table_shows_counts() {
        let status = MigrationStatusReport {
            applied: vec![
                make_entry("20260101_001", MigrationSource::Legacy),
                make_entry("20260322_001", MigrationSource::Native),
            ],
            pending: vec!["20260322_002_add_index".to_string()],
        };
        let table = status.to_table();
        assert!(table.contains("Applied: 2"));
        assert!(table.contains("Pending: 1"));
        assert!(table.contains("legacy"));
        assert!(table.contains("native"));
        assert!(table.contains("20260322_002_add_index"));
    }

    #[test]
    fn status_table_empty() {
        let status = MigrationStatusReport {
            applied: vec![],
            pending: vec![],
        };
        let table = status.to_table();
        assert!(table.contains("Applied: 0"));
        assert!(table.contains("Pending: 0"));
    }
}
