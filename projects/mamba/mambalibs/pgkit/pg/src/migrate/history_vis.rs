//! Migration history visualization.
//!
//! Provides ASCII tree visualization, branch detection, and export
//! formats for migration history.

use crate::migration::{Migration, MigrationRunner};
use crate::{Connection, Result};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Migration Node
// ============================================================================

/// A node in the migration tree.
#[derive(Debug, Clone)]
pub struct MigrationNode {
    /// Migration version
    pub version: String,
    /// Migration name/description
    pub name: String,
    /// When applied (None if pending)
    pub applied_at: Option<DateTime<Utc>>,
    /// Parent version (None for initial migration)
    pub parent: Option<String>,
    /// Child versions
    pub children: Vec<String>,
    /// Is on main branch
    pub is_main_branch: bool,
    /// Branch name if not on main
    pub branch_name: Option<String>,
}

impl MigrationNode {
    /// Create from a Migration.
    pub fn from_migration(migration: &Migration, parent: Option<String>) -> Self {
        Self {
            version: migration.version.clone(),
            name: migration.name.clone(),
            applied_at: migration.applied_at,
            parent,
            children: Vec::new(),
            is_main_branch: true,
            branch_name: None,
        }
    }

    /// Check if migration is applied.
    pub fn is_applied(&self) -> bool {
        self.applied_at.is_some()
    }

    /// Check if this is a branch point.
    pub fn is_branch_point(&self) -> bool {
        self.children.len() > 1
    }
}

// ============================================================================
// Migration Tree
// ============================================================================

/// Migration tree structure for visualization.
#[derive(Debug, Clone)]
pub struct MigrationTree {
    /// All nodes indexed by version
    nodes: HashMap<String, MigrationNode>,
    /// Root versions (migrations with no parent)
    #[allow(dead_code)]
    roots: Vec<String>,
    /// Branch names
    branches: HashMap<String, String>,
}

impl MigrationTree {
    /// Build a migration tree from a list of migrations.
    pub fn build(migrations: &[Migration]) -> Self {
        let mut nodes: HashMap<String, MigrationNode> = HashMap::new();
        let mut roots: Vec<String> = Vec::new();

        // Create nodes in order
        let mut prev_version: Option<String> = None;

        for migration in migrations {
            let node = MigrationNode::from_migration(migration, prev_version.clone());

            // Update parent's children
            if let Some(ref parent_version) = prev_version {
                if let Some(parent_node) = nodes.get_mut(parent_version) {
                    parent_node.children.push(node.version.clone());
                }
            } else {
                roots.push(node.version.clone());
            }

            nodes.insert(node.version.clone(), node);
            prev_version = Some(migration.version.clone());
        }

        // Detect branches
        let mut tree = Self {
            nodes,
            roots,
            branches: HashMap::new(),
        };
        tree.detect_branches();

        tree
    }

    /// Detect and mark branches.
    fn detect_branches(&mut self) {
        // Find branch points (nodes with multiple children)
        let branch_points: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.children.len() > 1)
            .map(|n| n.version.clone())
            .collect();

        for bp_version in branch_points {
            if let Some(bp_node) = self.nodes.get(&bp_version) {
                let children = bp_node.children.clone();

                // First child is main branch, rest are named branches
                for (i, child_version) in children.iter().enumerate().skip(1) {
                    let branch_name = format!("branch_{}", i);
                    self.mark_branch(child_version, &branch_name);
                    self.branches
                        .insert(child_version.clone(), branch_name.clone());
                }
            }
        }
    }

    /// Mark a node and its descendants as belonging to a branch.
    fn mark_branch(&mut self, version: &str, branch_name: &str) {
        if let Some(node) = self.nodes.get_mut(version) {
            node.is_main_branch = false;
            node.branch_name = Some(branch_name.to_string());

            let children = node.children.clone();
            for child in children {
                self.mark_branch(&child, branch_name);
            }
        }
    }

    /// Get all nodes in order.
    pub fn nodes_in_order(&self) -> Vec<&MigrationNode> {
        let mut nodes: Vec<&MigrationNode> = self.nodes.values().collect();
        nodes.sort_by(|a, b| a.version.cmp(&b.version));
        nodes
    }

    /// Get branch names.
    pub fn branch_names(&self) -> Vec<&str> {
        self.branches.values().map(|s| s.as_str()).collect()
    }

    /// Check if tree has branches.
    pub fn has_branches(&self) -> bool {
        !self.branches.is_empty()
    }
}

// ============================================================================
// ASCII Visualization
// ============================================================================

/// ASCII tree visualization configuration.
#[derive(Debug, Clone)]
pub struct AsciiConfig {
    /// Show timestamps
    pub show_timestamps: bool,
    /// Show status (applied/pending)
    pub show_status: bool,
    /// Maximum description length
    pub max_description_len: usize,
    /// Use Unicode characters
    pub use_unicode: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            show_status: true,
            max_description_len: 40,
            use_unicode: true,
        }
    }
}

/// ASCII tree renderer.
pub struct AsciiRenderer {
    config: AsciiConfig,
}

impl AsciiRenderer {
    /// Create a new renderer with default config.
    pub fn new() -> Self {
        Self {
            config: AsciiConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: AsciiConfig) -> Self {
        Self { config }
    }

    /// Render migration tree to ASCII.
    pub fn render(&self, tree: &MigrationTree) -> String {
        let mut lines = Vec::new();

        lines.push("Migration History".to_string());
        lines.push(self.separator_line());
        lines.push(String::new());

        let nodes = tree.nodes_in_order();

        if nodes.is_empty() {
            lines.push("No migrations found.".to_string());
            return lines.join("\n");
        }

        // Track branch state
        let mut active_branches: HashSet<String> = HashSet::new();

        for (i, node) in nodes.iter().enumerate() {
            let is_last = i == nodes.len() - 1;
            let line = self.render_node(node, is_last, tree, &active_branches);
            lines.push(line);

            // Update active branches
            if let Some(ref branch) = node.branch_name {
                if node.children.is_empty() {
                    active_branches.remove(branch);
                } else {
                    active_branches.insert(branch.clone());
                }
            }
        }

        lines.push(String::new());
        lines.push(self.render_legend(tree));

        lines.join("\n")
    }

    fn separator_line(&self) -> String {
        if self.config.use_unicode {
            "─".repeat(60)
        } else {
            "-".repeat(60)
        }
    }

    fn render_node(
        &self,
        node: &MigrationNode,
        is_last: bool,
        _tree: &MigrationTree,
        _active_branches: &HashSet<String>,
    ) -> String {
        let mut parts = Vec::new();

        // Tree characters
        let (connector, branch_char) = if self.config.use_unicode {
            if is_last {
                ("└", "─")
            } else if node.is_branch_point() {
                ("├", "┬")
            } else {
                ("├", "─")
            }
        } else if is_last {
            ("`", "-")
        } else if node.is_branch_point() {
            ("+", "+")
        } else {
            ("|", "-")
        };

        // Status indicator
        let status = if self.config.show_status {
            if node.is_applied() {
                if self.config.use_unicode {
                    "✓"
                } else {
                    "[x]"
                }
            } else if self.config.use_unicode {
                "○"
            } else {
                "[ ]"
            }
        } else {
            ""
        };

        // Branch indicator
        let branch_indicator = if !node.is_main_branch {
            if let Some(ref name) = node.branch_name {
                format!(" ({})", name)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Build line
        parts.push(format!(
            "{}{}{}",
            connector,
            branch_char.repeat(2),
            branch_char
        ));

        parts.push(format!(" {} ", status));
        parts.push(node.version.clone());

        // Description
        let desc = self.truncate(&node.name, self.config.max_description_len);
        parts.push(format!(" - {}", desc));

        // Branch indicator
        parts.push(branch_indicator);

        // Timestamp
        if self.config.show_timestamps {
            if let Some(applied_at) = node.applied_at {
                parts.push(format!(" ({})", applied_at.format("%Y-%m-%d")));
            } else {
                parts.push(" (pending)".to_string());
            }
        }

        parts.join("")
    }

    fn truncate(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }

    fn render_legend(&self, tree: &MigrationTree) -> String {
        let mut legend = Vec::new();

        if self.config.show_status {
            let (applied, pending) = if self.config.use_unicode {
                ("✓ = applied", "○ = pending")
            } else {
                ("[x] = applied", "[ ] = pending")
            };
            legend.push(format!("Legend: {} | {}", applied, pending));
        }

        if tree.has_branches() {
            let branches: Vec<String> = tree
                .branches
                .iter()
                .map(|(v, name)| format!("{}: {}", name, v))
                .collect();
            legend.push(format!("Branches: {}", branches.join(", ")));
        }

        legend.join("\n")
    }
}

impl Default for AsciiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Export Formats
// ============================================================================

/// Export format for migration history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Mermaid diagram
    Mermaid,
    /// JSON
    Json,
    /// Plain text
    Text,
    /// Markdown
    Markdown,
}

/// Migration history exporter.
pub struct HistoryExporter;

impl HistoryExporter {
    /// Export migration tree to specified format.
    pub fn export(tree: &MigrationTree, format: ExportFormat) -> String {
        match format {
            ExportFormat::Mermaid => Self::export_mermaid(tree),
            ExportFormat::Json => Self::export_json(tree),
            ExportFormat::Text => Self::export_text(tree),
            ExportFormat::Markdown => Self::export_markdown(tree),
        }
    }

    /// Export to Mermaid diagram format.
    fn export_mermaid(tree: &MigrationTree) -> String {
        let mut lines = Vec::new();
        lines.push("graph TD".to_string());

        let nodes = tree.nodes_in_order();

        for node in &nodes {
            // Node definition
            let status = if node.is_applied() { "✓" } else { "○" };
            let short_name = if node.name.len() > 20 {
                format!("{}...", &node.name[..17])
            } else {
                node.name.clone()
            };
            lines.push(format!(
                "    {}[\"{} {} - {}\"]",
                node.version.replace('_', ""), // Mermaid IDs can't have underscores
                status,
                node.version,
                short_name
            ));

            // Connections
            if let Some(ref parent) = node.parent {
                lines.push(format!(
                    "    {} --> {}",
                    parent.replace('_', ""),
                    node.version.replace('_', "")
                ));
            }
        }

        // Style applied vs pending
        lines.push(String::new());
        let applied: Vec<String> = nodes
            .iter()
            .filter(|n| n.is_applied())
            .map(|n| n.version.replace('_', ""))
            .collect();
        let pending: Vec<String> = nodes
            .iter()
            .filter(|n| !n.is_applied())
            .map(|n| n.version.replace('_', ""))
            .collect();

        if !applied.is_empty() {
            lines.push("    classDef applied fill:#90EE90".to_string());
            lines.push(format!("    class {} applied", applied.join(",")));
        }

        if !pending.is_empty() {
            lines.push("    classDef pending fill:#FFB6C1".to_string());
            lines.push(format!("    class {} pending", pending.join(",")));
        }

        lines.join("\n")
    }

    /// Export to JSON format.
    fn export_json(tree: &MigrationTree) -> String {
        let nodes: Vec<serde_json::Value> = tree
            .nodes_in_order()
            .iter()
            .map(|n| {
                serde_json::json!({
                    "version": n.version,
                    "name": n.name,
                    "applied": n.is_applied(),
                    "applied_at": n.applied_at.map(|dt| dt.to_rfc3339()),
                    "parent": n.parent,
                    "children": n.children,
                    "branch": n.branch_name,
                })
            })
            .collect();

        let output = serde_json::json!({
            "migrations": nodes,
            "branches": tree.branches,
            "total": tree.nodes.len(),
            "applied": tree.nodes.values().filter(|n| n.is_applied()).count(),
            "pending": tree.nodes.values().filter(|n| !n.is_applied()).count(),
        });

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export to plain text format.
    fn export_text(tree: &MigrationTree) -> String {
        let renderer = AsciiRenderer::with_config(AsciiConfig {
            use_unicode: false,
            ..Default::default()
        });
        renderer.render(tree)
    }

    /// Export to Markdown format.
    fn export_markdown(tree: &MigrationTree) -> String {
        let mut lines = Vec::new();

        lines.push("# Migration History".to_string());
        lines.push(String::new());

        let nodes = tree.nodes_in_order();
        let applied_count = nodes.iter().filter(|n| n.is_applied()).count();
        let pending_count = nodes.len() - applied_count;

        lines.push(format!(
            "**Total:** {} migrations ({} applied, {} pending)",
            nodes.len(),
            applied_count,
            pending_count
        ));
        lines.push(String::new());

        // Table header
        lines.push("| Status | Version | Description | Applied At |".to_string());
        lines.push("|--------|---------|-------------|------------|".to_string());

        for node in &nodes {
            let status = if node.is_applied() { "✅" } else { "⏳" };
            let applied_at = node
                .applied_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".to_string());

            lines.push(format!(
                "| {} | {} | {} | {} |",
                status, node.version, node.name, applied_at
            ));
        }

        if tree.has_branches() {
            lines.push(String::new());
            lines.push("## Branches".to_string());
            lines.push(String::new());

            for (version, name) in &tree.branches {
                lines.push(format!("- **{}**: starts at `{}`", name, version));
            }
        }

        lines.join("\n")
    }
}

// ============================================================================
// High-Level API
// ============================================================================

/// Visualize migration history.
pub struct HistoryVisualizer {
    conn: Connection,
    migrations_table: String,
}

impl HistoryVisualizer {
    /// Create a new visualizer.
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            migrations_table: "_migrations".to_string(),
        }
    }

    /// Set migrations table name.
    pub fn migrations_table(mut self, table: impl Into<String>) -> Self {
        self.migrations_table = table.into();
        self
    }

    /// Get ASCII visualization of migration history.
    pub async fn ascii(&self, migrations: &[Migration]) -> Result<String> {
        let runner = MigrationRunner::new(self.conn.clone(), Some(self.migrations_table.clone()));

        // Get applied migrations with actual timestamps from database
        let applied_migrations = runner.applied_migrations_with_details().await?;
        let applied_timestamps: HashMap<String, DateTime<Utc>> = applied_migrations
            .into_iter()
            .filter_map(|m| m.applied_at.map(|ts| (m.version, ts)))
            .collect();

        // Update migrations with applied status and actual timestamps
        let migrations_with_status: Vec<Migration> = migrations
            .iter()
            .map(|m| {
                let mut m = m.clone();
                if let Some(ts) = applied_timestamps.get(&m.version) {
                    m.applied_at = Some(*ts);
                }
                m
            })
            .collect();

        let tree = MigrationTree::build(&migrations_with_status);
        let renderer = AsciiRenderer::new();

        Ok(renderer.render(&tree))
    }

    /// Export migration history to specified format.
    pub async fn export(&self, migrations: &[Migration], format: ExportFormat) -> Result<String> {
        let runner = MigrationRunner::new(self.conn.clone(), Some(self.migrations_table.clone()));

        // Get applied migrations with actual timestamps from database
        let applied_migrations = runner.applied_migrations_with_details().await?;
        let applied_timestamps: HashMap<String, DateTime<Utc>> = applied_migrations
            .into_iter()
            .filter_map(|m| m.applied_at.map(|ts| (m.version, ts)))
            .collect();

        // Update migrations with applied status and actual timestamps
        let migrations_with_status: Vec<Migration> = migrations
            .iter()
            .map(|m| {
                let mut m = m.clone();
                if let Some(ts) = applied_timestamps.get(&m.version) {
                    m.applied_at = Some(*ts);
                }
                m
            })
            .collect();

        let tree = MigrationTree::build(&migrations_with_status);

        Ok(HistoryExporter::export(&tree, format))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_migrations() -> Vec<Migration> {
        vec![
            Migration::new(
                "20240101_000000".to_string(),
                "Initial schema".to_string(),
                "CREATE TABLE users".to_string(),
                "DROP TABLE users".to_string(),
            ),
            Migration::new(
                "20240102_000000".to_string(),
                "Add email column".to_string(),
                "ALTER TABLE users ADD email".to_string(),
                "ALTER TABLE users DROP email".to_string(),
            ),
            Migration::new(
                "20240103_000000".to_string(),
                "Add indexes".to_string(),
                "CREATE INDEX".to_string(),
                "DROP INDEX".to_string(),
            ),
        ]
    }

    #[test]
    fn test_migration_tree_build() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);

        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0], "20240101_000000");
    }

    #[test]
    fn test_migration_node_order() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);
        let nodes = tree.nodes_in_order();

        assert_eq!(nodes[0].version, "20240101_000000");
        assert_eq!(nodes[1].version, "20240102_000000");
        assert_eq!(nodes[2].version, "20240103_000000");
    }

    #[test]
    fn test_ascii_renderer() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);
        let renderer = AsciiRenderer::new();

        let output = renderer.render(&tree);

        assert!(output.contains("Migration History"));
        assert!(output.contains("20240101_000000"));
        assert!(output.contains("Initial schema"));
    }

    #[test]
    fn test_mermaid_export() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);

        let output = HistoryExporter::export(&tree, ExportFormat::Mermaid);

        assert!(output.contains("graph TD"));
        assert!(output.contains("20240101000000"));
    }

    #[test]
    fn test_json_export() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);

        let output = HistoryExporter::export(&tree, ExportFormat::Json);

        assert!(output.contains("\"migrations\""));
        assert!(output.contains("20240101_000000"));
    }

    #[test]
    fn test_markdown_export() {
        let migrations = sample_migrations();
        let tree = MigrationTree::build(&migrations);

        let output = HistoryExporter::export(&tree, ExportFormat::Markdown);

        assert!(output.contains("# Migration History"));
        assert!(output.contains("| Status |"));
        assert!(output.contains("20240101_000000"));
    }
}
