//! CLI migration tool for database schema management.
//!
//! Provides command-line interface for applying, reverting, and managing
//! database migrations.

use crate::migration::MigrationRunner;
use crate::{Connection, DataBridgeError, PoolConfig, Result};
use std::path::PathBuf;

// ============================================================================
// CLI Configuration
// ============================================================================

/// CLI configuration for migration operations.
#[derive(Debug, Clone)]
pub struct MigrationCliConfig {
    /// Database connection string
    pub database_url: String,
    /// Migrations directory
    pub migrations_dir: PathBuf,
    /// Migrations table name
    pub migrations_table: String,
    /// Dry run mode (show SQL without executing)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
}

impl MigrationCliConfig {
    /// Create from environment variables.
    ///
    /// Reads DATABASE_URL and optionally MIGRATIONS_DIR.
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            DataBridgeError::Internal("DATABASE_URL environment variable not set".to_string())
        })?;

        let migrations_dir = std::env::var("MIGRATIONS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./migrations"));

        let migrations_table = std::env::var("MIGRATIONS_TABLE")
            .unwrap_or_else(|_| "_migrations".to_string());

        Ok(Self {
            database_url,
            migrations_dir,
            migrations_table,
            dry_run: false,
            verbose: false,
        })
    }

    /// Create with explicit values.
    pub fn new(database_url: impl Into<String>, migrations_dir: impl Into<PathBuf>) -> Self {
        Self {
            database_url: database_url.into(),
            migrations_dir: migrations_dir.into(),
            migrations_table: "_migrations".to_string(),
            dry_run: false,
            verbose: false,
        }
    }

    /// Set dry run mode.
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set verbose mode.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set migrations table name.
    pub fn migrations_table(mut self, table: impl Into<String>) -> Self {
        self.migrations_table = table.into();
        self
    }
}

// ============================================================================
// CLI Commands
// ============================================================================

/// Result of a CLI operation.
#[derive(Debug, Clone)]
pub struct CliResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Output messages
    pub messages: Vec<String>,
    /// Affected migration versions
    pub affected: Vec<String>,
}

impl CliResult {
    /// Create a success result.
    pub fn success(messages: Vec<String>, affected: Vec<String>) -> Self {
        Self {
            success: true,
            messages,
            affected,
        }
    }

    /// Create a failure result.
    pub fn failure(messages: Vec<String>) -> Self {
        Self {
            success: false,
            messages,
            affected: Vec::new(),
        }
    }
}

/// Migration CLI handler.
pub struct MigrationCli {
    config: MigrationCliConfig,
}

impl MigrationCli {
    /// Create a new CLI handler.
    pub fn new(config: MigrationCliConfig) -> Self {
        Self { config }
    }

    /// Execute the "up" command - apply pending migrations.
    ///
    /// # Arguments
    /// * `steps` - Number of migrations to apply (None = all pending)
    pub async fn up(&self, steps: Option<usize>) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        // Initialize migrations table
        runner.init().await?;

        // Load migrations
        let migrations = MigrationRunner::load_from_directory(&self.config.migrations_dir)?;
        let pending = runner.pending_migrations(&migrations).await?;

        if pending.is_empty() {
            return Ok(CliResult::success(
                vec!["No pending migrations to apply".to_string()],
                Vec::new(),
            ));
        }

        // Determine how many to apply
        let to_apply: Vec<_> = match steps {
            Some(n) => pending.into_iter().take(n).collect(),
            None => pending,
        };

        let mut messages = Vec::new();
        let mut affected = Vec::new();

        if self.config.dry_run {
            messages.push("DRY RUN - No changes will be made".to_string());
            messages.push(String::new());
        }

        for migration in &to_apply {
            if self.config.dry_run {
                messages.push(format!(
                    "Would apply: {} - {}",
                    migration.version, migration.name
                ));
                if self.config.verbose {
                    messages.push("SQL:".to_string());
                    messages.push(migration.up.clone());
                    messages.push(String::new());
                }
            } else {
                messages.push(format!(
                    "Applying: {} - {}",
                    migration.version, migration.name
                ));
                runner.apply(migration).await?;
                messages.push(format!("Applied: {}", migration.version));
            }
            affected.push(migration.version.clone());
        }

        if !self.config.dry_run {
            messages.push(format!("Applied {} migration(s)", affected.len()));
        }

        Ok(CliResult::success(messages, affected))
    }

    /// Execute the "down" command - revert migrations.
    ///
    /// # Arguments
    /// * `steps` - Number of migrations to revert (default: 1)
    pub async fn down(&self, steps: Option<usize>) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        // Load migrations
        let migrations = MigrationRunner::load_from_directory(&self.config.migrations_dir)?;

        let steps = steps.unwrap_or(1);

        if self.config.dry_run {
            let applied = runner.applied_migrations().await?;
            let to_revert: Vec<_> = applied.iter().rev().take(steps).cloned().collect();

            if to_revert.is_empty() {
                return Ok(CliResult::success(
                    vec!["No migrations to revert".to_string()],
                    Vec::new(),
                ));
            }

            let mut messages = vec![
                "DRY RUN - No changes will be made".to_string(),
                String::new(),
            ];

            for version in &to_revert {
                if let Some(migration) = migrations.iter().find(|m| &m.version == version) {
                    messages.push(format!(
                        "Would revert: {} - {}",
                        migration.version, migration.name
                    ));
                    if self.config.verbose {
                        messages.push("SQL:".to_string());
                        messages.push(migration.down.clone());
                        messages.push(String::new());
                    }
                }
            }

            return Ok(CliResult::success(messages, to_revert));
        }

        let reverted = runner.rollback(&migrations, steps).await?;

        if reverted.is_empty() {
            return Ok(CliResult::success(
                vec!["No migrations to revert".to_string()],
                Vec::new(),
            ));
        }

        let messages = vec![format!("Reverted {} migration(s)", reverted.len())];
        Ok(CliResult::success(messages, reverted))
    }

    /// Execute the "status" command - show current migration status.
    pub async fn status(&self) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        // Initialize migrations table (creates if not exists)
        runner.init().await?;

        // Load migrations
        let migrations = MigrationRunner::load_from_directory(&self.config.migrations_dir)?;
        let status = runner.status(&migrations).await?;

        let mut messages = Vec::new();

        messages.push("Migration Status".to_string());
        messages.push("================".to_string());
        messages.push(String::new());

        messages.push(format!("Applied: {} migrations", status.applied.len()));
        messages.push(format!("Pending: {} migrations", status.pending.len()));
        messages.push(String::new());

        if !status.applied.is_empty() {
            messages.push("Applied Migrations:".to_string());
            for version in &status.applied {
                if let Some(migration) = migrations.iter().find(|m| &m.version == version) {
                    messages.push(format!("  [✓] {} - {}", version, migration.name));
                } else {
                    messages.push(format!("  [✓] {} - (file not found)", version));
                }
            }
            messages.push(String::new());
        }

        if !status.pending.is_empty() {
            messages.push("Pending Migrations:".to_string());
            for version in &status.pending {
                if let Some(migration) = migrations.iter().find(|m| &m.version == version) {
                    messages.push(format!("  [ ] {} - {}", version, migration.name));
                }
            }
        }

        Ok(CliResult::success(messages, Vec::new()))
    }

    /// Execute the "history" command - show migration history with details.
    pub async fn history(&self) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        let applied = runner.applied_migrations_with_details().await?;

        let mut messages = Vec::new();

        messages.push("Migration History".to_string());
        messages.push("=================".to_string());
        messages.push(String::new());

        if applied.is_empty() {
            messages.push("No migrations have been applied yet.".to_string());
        } else {
            messages.push(format!(
                "{:<20} {:<30} {}",
                "VERSION", "DESCRIPTION", "APPLIED AT"
            ));
            messages.push(format!("{:-<20} {:-<30} {:-<25}", "", "", ""));

            for migration in &applied {
                let applied_at = migration
                    .applied_at
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                let name = if migration.name.len() > 28 {
                    format!("{}...", &migration.name[..25])
                } else {
                    migration.name.clone()
                };

                messages.push(format!("{:<20} {:<30} {}", migration.version, name, applied_at));
            }
        }

        Ok(CliResult::success(messages, Vec::new()))
    }

    /// Execute the "current" command - show current migration version.
    pub async fn current(&self) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        let applied = runner.applied_migrations().await?;

        let messages = if let Some(version) = applied.last() {
            vec![format!("Current version: {}", version)]
        } else {
            vec!["No migrations applied yet".to_string()]
        };

        Ok(CliResult::success(messages, Vec::new()))
    }

    /// Execute the "validate" command - verify migration checksums.
    pub async fn validate(&self) -> Result<CliResult> {
        let conn = self.connect().await?;
        let runner = MigrationRunner::new(conn, Some(self.config.migrations_table.clone()));

        // Load migrations from files
        let file_migrations = MigrationRunner::load_from_directory(&self.config.migrations_dir)?;

        // Get applied migrations with checksums
        let applied_migrations = runner.applied_migrations_with_details().await?;

        let mut messages = Vec::new();
        let mut errors = Vec::new();

        messages.push("Validating migration checksums...".to_string());
        messages.push(String::new());

        for applied in &applied_migrations {
            if let Some(file_migration) =
                file_migrations.iter().find(|m| m.version == applied.version)
            {
                if file_migration.checksum == applied.checksum {
                    messages.push(format!("  [✓] {} - checksum valid", applied.version));
                } else {
                    messages.push(format!("  [✗] {} - CHECKSUM MISMATCH", applied.version));
                    errors.push(format!(
                        "Migration {} has been modified after being applied",
                        applied.version
                    ));
                }
            } else {
                messages.push(format!("  [?] {} - file not found", applied.version));
                errors.push(format!(
                    "Migration file for {} not found in {}",
                    applied.version,
                    self.config.migrations_dir.display()
                ));
            }
        }

        messages.push(String::new());

        if errors.is_empty() {
            messages.push("All migrations valid!".to_string());
            Ok(CliResult::success(messages, Vec::new()))
        } else {
            messages.push("Validation errors:".to_string());
            for error in errors {
                messages.push(format!("  - {}", error));
            }
            Ok(CliResult::failure(messages))
        }
    }

    /// Create database connection.
    async fn connect(&self) -> Result<Connection> {
        Connection::new(&self.config.database_url, PoolConfig::default()).await
    }
}

// ============================================================================
// Command Parsing
// ============================================================================

/// Parsed CLI command.
#[derive(Debug, Clone)]
pub enum MigrationCommand {
    /// Apply pending migrations
    Up { steps: Option<usize> },
    /// Revert migrations
    Down { steps: Option<usize> },
    /// Show migration status
    Status,
    /// Show migration history
    History,
    /// Show current version
    Current,
    /// Validate checksums
    Validate,
}

impl MigrationCommand {
    /// Parse from command line arguments.
    pub fn parse(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(DataBridgeError::Validation(
                "No command specified. Use: up, down, status, history, current, validate"
                    .to_string(),
            ));
        }

        let command = args[0].to_lowercase();
        let rest = &args[1..];

        match command.as_str() {
            "up" => {
                let steps = Self::parse_steps_arg(rest)?;
                Ok(MigrationCommand::Up { steps })
            }
            "down" => {
                let steps = Self::parse_steps_arg(rest);
                Ok(MigrationCommand::Down {
                    steps: steps.ok().flatten().or(Some(1)),
                })
            }
            "status" => Ok(MigrationCommand::Status),
            "history" => Ok(MigrationCommand::History),
            "current" => Ok(MigrationCommand::Current),
            "validate" => Ok(MigrationCommand::Validate),
            _ => Err(DataBridgeError::Validation(format!(
                "Unknown command: {}. Use: up, down, status, history, current, validate",
                command
            ))),
        }
    }

    fn parse_steps_arg(args: &[String]) -> Result<Option<usize>> {
        for (i, arg) in args.iter().enumerate() {
            if arg == "--steps" || arg == "-n" {
                if let Some(value) = args.get(i + 1) {
                    let steps: usize = value.parse().map_err(|_| {
                        DataBridgeError::Validation(format!("Invalid steps value: {}", value))
                    })?;
                    return Ok(Some(steps));
                }
            }
        }
        Ok(None)
    }

    /// Execute the command.
    pub async fn execute(&self, cli: &MigrationCli) -> Result<CliResult> {
        match self {
            MigrationCommand::Up { steps } => cli.up(*steps).await,
            MigrationCommand::Down { steps } => cli.down(*steps).await,
            MigrationCommand::Status => cli.status().await,
            MigrationCommand::History => cli.history().await,
            MigrationCommand::Current => cli.current().await,
            MigrationCommand::Validate => cli.validate().await,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = MigrationCliConfig::new("postgres://localhost/test", "./migrations")
            .dry_run(true)
            .verbose(true)
            .migrations_table("schema_migrations");

        assert!(config.dry_run);
        assert!(config.verbose);
        assert_eq!(config.migrations_table, "schema_migrations");
    }

    #[test]
    fn test_parse_up_command() {
        let args = vec!["up".to_string()];
        let cmd = MigrationCommand::parse(&args).unwrap();
        assert!(matches!(cmd, MigrationCommand::Up { steps: None }));
    }

    #[test]
    fn test_parse_up_with_steps() {
        let args = vec!["up".to_string(), "--steps".to_string(), "2".to_string()];
        let cmd = MigrationCommand::parse(&args).unwrap();
        assert!(matches!(cmd, MigrationCommand::Up { steps: Some(2) }));
    }

    #[test]
    fn test_parse_down_default_steps() {
        let args = vec!["down".to_string()];
        let cmd = MigrationCommand::parse(&args).unwrap();
        assert!(matches!(cmd, MigrationCommand::Down { steps: Some(1) }));
    }

    #[test]
    fn test_parse_status() {
        let args = vec!["status".to_string()];
        let cmd = MigrationCommand::parse(&args).unwrap();
        assert!(matches!(cmd, MigrationCommand::Status));
    }

    #[test]
    fn test_parse_unknown_command() {
        let args = vec!["unknown".to_string()];
        let result = MigrationCommand::parse(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_result() {
        let result = CliResult::success(
            vec!["Applied migration".to_string()],
            vec!["20240101_000000".to_string()],
        );

        assert!(result.success);
        assert_eq!(result.affected.len(), 1);
    }
}
