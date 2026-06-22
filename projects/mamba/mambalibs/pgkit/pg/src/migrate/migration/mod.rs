//! Database migration management.
//!
//! This module provides migration support for schema evolution,
//! similar to Alembic/SQLAlchemy migrations but in Rust.
//!
//! # Submodules
//!
//! | Module            | Purpose                                          |
//! |-------------------|--------------------------------------------------|
//! | [`runner`]        | Apply / rollback / init; bootstraps legacy history |
//! | [`model_diff`]    | Introspect models; generate SQL diff             |
//! | [`status_report`] | Query tracking table; format output table         |
//!
//! # Tracking table schema
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS _migrations (
//!     migration_id  TEXT         PRIMARY KEY,
//!     source        TEXT         NOT NULL CHECK (source IN ('legacy', 'native')),
//!     applied_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
//!     checksum      TEXT         NOT NULL
//! );
//! ```

pub mod model_diff;
pub mod runner;
pub mod status_report;

pub use model_diff::{ModelDiffResult, ModelDiffer};
pub use runner::MigrationRunner;
pub use status_report::MigrationStatusReport;

use crate::{DataBridgeError, Result};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

// -- Constants ----------------------------------------------------------------

/// Default name for the migrations tracking table.
pub(crate) const DEFAULT_TABLE: &str = "_migrations";
/// Name of the Alembic version table used for legacy bootstrap.
pub(crate) const ALEMBIC_TABLE: &str = "alembic_version";

// -- MigrationSource ----------------------------------------------------------

/// Whether a migration entry originated from a legacy system (e.g. Alembic)
/// or from the native `cclab pg migrate` runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationSource {
    /// Bootstrapped from a legacy migration system (e.g. Alembic) on first deploy.
    Legacy,
    /// Generated and applied by `cclab pg migrate`.
    Native,
}

impl MigrationSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            MigrationSource::Legacy => "legacy",
            MigrationSource::Native => "native",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "legacy" | "alembic" => MigrationSource::Legacy,
            _ => MigrationSource::Native,
        }
    }
}

// -- MigrationEntry -----------------------------------------------------------

/// A row in the migrations tracking table.
#[derive(Debug, Clone)]
pub struct MigrationEntry {
    /// Migration version identifier (e.g. `20260322_001_add_email`).
    pub migration_id: String,
    /// Whether this came from a legacy system or the native runner.
    pub source: MigrationSource,
    /// Timestamp when the migration was applied.
    pub applied_at: DateTime<Utc>,
    /// SHA-256 of the up SQL (empty string for legacy entries).
    pub checksum: String,
}

// -- Migration ----------------------------------------------------------------

/// Represents a single database migration.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version (timestamp or sequential number)
    pub version: String,
    /// Migration name/description
    pub name: String,
    /// SQL statements to apply migration (upgrade)
    pub up: String,
    /// SQL statements to revert migration (downgrade)
    pub down: String,
    /// When this migration was applied (None if not applied)
    pub applied_at: Option<DateTime<Utc>>,
    /// SHA256 checksum of migration content
    pub checksum: String,
}

impl Migration {
    /// Creates a new migration.
    ///
    /// # Arguments
    ///
    /// * `version` - Migration version identifier
    /// * `name` - Migration description
    /// * `up` - SQL for applying migration
    /// * `down` - SQL for reverting migration
    pub fn new(version: String, name: String, up: String, down: String) -> Self {
        let content = format!("{}\n{}\n{}", name, up, down);
        let checksum = Self::calculate_checksum(&content);

        Self {
            version,
            name,
            up,
            down,
            applied_at: None,
            checksum,
        }
    }

    /// Loads migration from a SQL file.
    ///
    /// Expected file format:
    /// ```sql
    /// -- Migration: 20250128_120000_create_users_table
    /// -- Description: Create users table with basic columns
    ///
    /// -- UP
    /// CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);
    ///
    /// -- DOWN
    /// DROP TABLE users;
    /// ```
    pub fn from_file(path: &Path) -> Result<Self> {
        // Read file contents
        let content = fs::read_to_string(path).map_err(|e| {
            DataBridgeError::Internal(format!("Failed to read migration file: {}", e))
        })?;

        // Extract version and description from filename
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DataBridgeError::Validation("Invalid migration filename".to_string()))?;

        let (version, description) = Self::parse_filename(filename)?;
        let (desc_from_content, up_sql, down_sql) = Self::parse_content(&content)?;

        // Use description from content if available, otherwise from filename
        let name = if !desc_from_content.is_empty() {
            desc_from_content
        } else {
            description
        };

        // Calculate checksum
        let checksum = Self::calculate_checksum(&content);

        Ok(Self {
            version,
            name,
            up: up_sql,
            down: down_sql,
            applied_at: None,
            checksum,
        })
    }

    /// Parses filename to extract version and description (expected format: YYYYMMDD_HHMMSS_description).
    fn parse_filename(filename: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = filename.splitn(3, '_').collect();

        if parts.len() < 2 {
            return Err(DataBridgeError::Validation(
                format!("Invalid migration filename format: {}. Expected format: YYYYMMDD_HHMMSS_description", filename)
            ));
        }

        let version = if parts.len() >= 2 {
            format!("{}_{}", parts[0], parts[1])
        } else {
            parts[0].to_string()
        };

        let description = if parts.len() >= 3 {
            parts[2].replace('_', " ")
        } else {
            String::new()
        };

        Ok((version, description))
    }

    /// Parses migration file content to extract description, up, and down SQL.
    fn parse_content(content: &str) -> Result<(String, String, String)> {
        let mut description = String::new();
        let mut up_sql = String::new();
        let mut down_sql = String::new();
        let mut current_section = Section::None;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section markers
            if trimmed.starts_with("-- Description:") {
                description = trimmed
                    .trim_start_matches("-- Description:")
                    .trim()
                    .to_string();
                continue;
            } else if trimmed.eq_ignore_ascii_case("-- UP")
                || trimmed.eq_ignore_ascii_case("-- migrate:up")
            {
                current_section = Section::Up;
                continue;
            } else if trimmed.eq_ignore_ascii_case("-- DOWN")
                || trimmed.eq_ignore_ascii_case("-- migrate:down")
            {
                current_section = Section::Down;
                continue;
            }

            // Skip other comment lines and empty lines when not in a section
            if current_section == Section::None {
                continue;
            }

            // Add non-empty lines to current section
            match current_section {
                Section::Up => {
                    up_sql.push_str(line);
                    up_sql.push('\n');
                }
                Section::Down => {
                    down_sql.push_str(line);
                    down_sql.push('\n');
                }
                Section::None => {}
            }
        }

        // Validate that we have both UP and DOWN sections
        if up_sql.trim().is_empty() {
            return Err(DataBridgeError::Validation(
                "Migration file missing UP section".to_string(),
            ));
        }

        if down_sql.trim().is_empty() {
            return Err(DataBridgeError::Validation(
                "Migration file missing DOWN section".to_string(),
            ));
        }

        Ok((
            description,
            up_sql.trim().to_string(),
            down_sql.trim().to_string(),
        ))
    }

    /// Calculates SHA256 checksum for migration content verification.
    fn calculate_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Section marker for parsing migration files.
#[derive(Debug, PartialEq)]
enum Section {
    None,
    Up,
    Down,
}

// -- SQL splitting ------------------------------------------------------------

/// Splits SQL into individual statements, handling PostgreSQL syntax (dollar quotes, comments).
pub(crate) fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag: Option<String> = None;
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        current_statement.push(ch);

        if in_dollar_quote {
            // Check if we're ending a dollar-quoted string
            if ch == '$' {
                // Collect the potential closing tag
                let mut potential_tag = String::from("$");
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        potential_tag.push(next_ch);
                        current_statement.push(next_ch);
                        chars.next();
                    } else if next_ch == '$' {
                        potential_tag.push(next_ch);
                        current_statement.push(next_ch);
                        chars.next();
                        break;
                    } else {
                        break;
                    }
                }

                // Check if this matches our opening tag
                if let Some(ref tag) = dollar_quote_tag {
                    if &potential_tag == tag {
                        in_dollar_quote = false;
                        dollar_quote_tag = None;
                    }
                }
            }
        } else {
            // Check if we're starting a dollar-quoted string
            if ch == '$' {
                // Collect the tag
                let mut tag = String::from("$");
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        tag.push(next_ch);
                        current_statement.push(next_ch);
                        chars.next();
                    } else if next_ch == '$' {
                        tag.push(next_ch);
                        current_statement.push(next_ch);
                        chars.next();
                        in_dollar_quote = true;
                        dollar_quote_tag = Some(tag);
                        break;
                    } else {
                        break;
                    }
                }
            } else if ch == ';' {
                // Found statement terminator (semicolon) outside of dollar-quotes
                let stmt = current_statement.trim().trim_end_matches(';').trim();

                // Check if this statement has any SQL (not just comments)
                let has_sql = stmt
                    .lines()
                    .map(|line| line.trim())
                    .any(|line| !line.is_empty() && !line.starts_with("--"));

                if has_sql {
                    statements.push(stmt.to_string());
                }

                current_statement.clear();
            }
        }
    }

    // Don't forget the last statement if it doesn't end with a semicolon
    let final_stmt = current_statement.trim().trim_end_matches(';').trim();
    let has_sql = final_stmt
        .lines()
        .map(|line| line.trim())
        .any(|line| !line.is_empty() && !line.starts_with("--"));

    if has_sql {
        statements.push(final_stmt.to_string());
    }

    statements
}

/// Simple semicolon split for generated SQL that does not contain PL/pgSQL
/// dollar-quote blocks.
pub(crate) fn split_statements_simple(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
        .collect()
}

/// Compute the SHA-256 hex digest of `text`.
pub(crate) fn sha256_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())
}

// -- MigrationStatus ----------------------------------------------------------

/// Contains lists of applied and pending migration versions.
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// Applied migration versions
    pub applied: Vec<String>,
    /// Pending migration versions
    pub pending: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sql_statements_single_statement() {
        let sql = "CREATE TABLE users (id SERIAL PRIMARY KEY)";
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0], "CREATE TABLE users (id SERIAL PRIMARY KEY)");
    }

    #[test]
    fn test_split_sql_statements_multiple_statements() {
        let sql = r#"
            CREATE TABLE users (id SERIAL PRIMARY KEY);
            CREATE INDEX idx_users ON users(id);
            INSERT INTO users VALUES (1);
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 3);
        assert!(statements[0].contains("CREATE TABLE users"));
        assert!(statements[1].contains("CREATE INDEX idx_users"));
        assert!(statements[2].contains("INSERT INTO users"));
    }

    #[test]
    fn test_split_sql_statements_with_inline_comments() {
        let sql = r#"
            CREATE TABLE users ( -- inline comment
                id SERIAL
            );
            CREATE INDEX idx_users ON users(id);
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE TABLE users"));
        assert!(statements[0].contains("-- inline comment"));
        assert!(statements[1].contains("CREATE INDEX idx_users"));
    }

    #[test]
    fn test_split_sql_statements_only_comment_statements_filtered() {
        let sql = r#"
            CREATE TABLE users (id SERIAL);
            -- This is just a comment, no SQL
            CREATE INDEX idx_users ON users(id);
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE TABLE users"));
        assert!(statements[1].contains("CREATE INDEX idx_users"));
    }

    #[test]
    fn test_split_sql_statements_leading_comments_included() {
        let sql = r#"
            -- This comment becomes part of the statement
            CREATE TABLE users (id SERIAL);
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].contains("-- This comment"));
        assert!(statements[0].contains("CREATE TABLE users"));
    }

    #[test]
    fn test_split_sql_statements_with_empty_lines() {
        let sql = r#"
            CREATE TABLE users (id SERIAL);

            CREATE INDEX idx_users ON users(id);

        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn test_split_sql_statements_empty_input() {
        let sql = "";
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 0);
    }

    #[test]
    fn test_split_sql_statements_only_whitespace() {
        let sql = "   \n\n\t  \n  ";
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 0);
    }

    #[test]
    fn test_split_sql_statements_semicolon_only() {
        let sql = ";;;";
        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 0);
    }

    #[test]
    fn test_split_sql_statements_complex_migration() {
        let sql = r#"
            CREATE TABLE products (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                price DECIMAL(10, 2),
                created_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE INDEX idx_products_name ON products(name);

            CREATE TABLE orders (
                id SERIAL PRIMARY KEY,
                product_id INTEGER REFERENCES products(id),
                quantity INTEGER NOT NULL
            );
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 3);
        assert!(statements[0].contains("CREATE TABLE products"));
        assert!(statements[1].contains("CREATE INDEX idx_products_name"));
        assert!(statements[2].contains("CREATE TABLE orders"));
    }

    #[test]
    fn test_split_sql_statements_with_dollar_quotes() {
        let sql = r#"
            CREATE FUNCTION test_func()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            CREATE TRIGGER test_trigger BEFORE UPDATE ON users
                FOR EACH ROW EXECUTE FUNCTION test_func();
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE FUNCTION"));
        assert!(statements[0].contains("$$"));
        assert!(statements[0].contains("NEW.updated_at = NOW()"));
        assert!(statements[1].contains("CREATE TRIGGER"));
    }

    #[test]
    fn test_split_sql_statements_with_tagged_dollar_quotes() {
        let sql = r#"
            CREATE FUNCTION complex_func()
            RETURNS TEXT AS $function$
            DECLARE
                result TEXT;
            BEGIN
                result := 'test; with semicolon';
                RETURN result;
            END;
            $function$ LANGUAGE plpgsql;
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 1);
        assert!(statements[0].contains("CREATE FUNCTION"));
        assert!(statements[0].contains("$function$"));
        assert!(statements[0].contains("test; with semicolon"));
    }

    #[test]
    fn test_split_sql_statements_real_migration() {
        let sql = r#"
            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) UNIQUE NOT NULL,
                name VARCHAR(255),
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE INDEX idx_users_email ON users(email);
            CREATE INDEX idx_users_created_at ON users(created_at);

            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = NOW();
                RETURN NEW;
            END;
            $$ language 'plpgsql';

            CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
                FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
        "#;

        let statements = split_sql_statements(sql);
        assert_eq!(statements.len(), 5);
        assert!(statements[0].contains("CREATE TABLE users"));
        assert!(statements[1].contains("CREATE INDEX idx_users_email"));
        assert!(statements[2].contains("CREATE INDEX idx_users_created_at"));
        assert!(statements[3].contains("CREATE OR REPLACE FUNCTION"));
        assert!(statements[3].contains("$$"));
        assert!(statements[4].contains("CREATE TRIGGER"));
    }

    // -- Tests for new types --------------------------------------------------

    #[test]
    fn migration_source_as_str() {
        assert_eq!(MigrationSource::Legacy.as_str(), "legacy");
        assert_eq!(MigrationSource::Native.as_str(), "native");
    }

    #[test]
    fn migration_source_from_str() {
        assert_eq!(MigrationSource::from_str("legacy"), MigrationSource::Legacy);
        assert_eq!(
            MigrationSource::from_str("alembic"),
            MigrationSource::Legacy
        );
        assert_eq!(MigrationSource::from_str("native"), MigrationSource::Native);
        assert_eq!(
            MigrationSource::from_str("unknown"),
            MigrationSource::Native
        );
    }

    #[test]
    fn split_statements_simple_basic() {
        let sql = "CREATE TABLE a (id INT); ALTER TABLE a ADD COLUMN b TEXT;";
        let stmts = split_statements_simple(sql);
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("CREATE TABLE a"));
        assert!(stmts[1].contains("ADD COLUMN b"));
    }

    #[test]
    fn split_statements_simple_skips_comments() {
        let sql = "-- just a comment; CREATE TABLE foo (id INT);";
        let stmts = split_statements_simple(sql);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("CREATE TABLE foo"));
    }

    #[test]
    fn split_statements_simple_empty_input() {
        assert_eq!(split_statements_simple("").len(), 0);
        assert_eq!(split_statements_simple("   \n\n\t  ").len(), 0);
    }

    #[test]
    fn sha256_hex_deterministic() {
        let a = sha256_hex("hello world");
        let b = sha256_hex("hello world");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn sha256_hex_different_inputs_differ() {
        let a = sha256_hex("input_one");
        let b = sha256_hex("input_two");
        assert_ne!(a, b);
    }
}
