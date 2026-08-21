//! Migration runner for applying, reverting, and tracking database migrations.
//!
//! Tracks migrations in a configurable table (default: `_migrations`) with
//! source tracking (`legacy` for bootstrapped entries, `native` for new ones).

use crate::{Connection, DataBridgeError, PoolConfig, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::fs;
use std::path::Path;

use super::{
    sha256_hex, split_sql_statements, split_statements_simple, Migration, MigrationEntry,
    MigrationSource, MigrationStatus, ALEMBIC_TABLE, DEFAULT_TABLE,
};

/// Migration runner for applying, reverting, and tracking database migrations.
///
/// Tracks migrations in a configurable table (default: `_migrations`) with
/// source tracking (`legacy` for bootstrapped entries, `native` for new ones).
#[derive(Debug)]
pub struct MigrationRunner {
    conn: Connection,
    migrations_table: String,
}

impl MigrationRunner {
    /// Creates a new migration runner with connection and optional custom tracking table.
    pub fn new(conn: Connection, migrations_table: Option<String>) -> Self {
        Self {
            conn,
            migrations_table: migrations_table.unwrap_or_else(|| DEFAULT_TABLE.to_string()),
        }
    }

    /// Connect using `database_url` and return a new runner with default table.
    pub async fn connect(database_url: &str) -> Result<Self> {
        let conn = Connection::new(database_url, PoolConfig::default()).await?;
        Ok(Self {
            conn,
            migrations_table: DEFAULT_TABLE.to_string(),
        })
    }

    /// Initializes the migrations tracking table.
    ///
    /// Creates the table if it does not exist. If `alembic_version` exists,
    /// seeds those entries as `source = 'legacy'` so they are not re-applied.
    pub async fn init(&self) -> Result<()> {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table} (
                migration_id  TEXT         PRIMARY KEY,
                source        TEXT         NOT NULL
                              CHECK (source IN ('legacy', 'native')),
                applied_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
                checksum      TEXT         NOT NULL
            )
            "#,
            table = self.migrations_table
        );

        sqlx::query(&sql)
            .execute(self.conn.pool())
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to create migrations table: {}", e))
            })?;

        self.bootstrap_alembic().await?;

        tracing::info!("Migrations table '{}' initialized", self.migrations_table);
        Ok(())
    }

    /// If `alembic_version` exists and has not yet been bootstrapped, seed
    /// all its version IDs into the tracking table with `source = 'legacy'`.
    async fn bootstrap_alembic(&self) -> Result<()> {
        // Check whether alembic_version table exists
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name = $1",
        )
        .bind(ALEMBIC_TABLE)
        .fetch_one(self.conn.pool())
        .await
        .map_err(|e| DataBridgeError::Database(format!("check alembic table: {e}")))?;

        if exists == 0 {
            return Ok(());
        }

        // Skip if already bootstrapped
        let already: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} WHERE source = 'legacy'",
            self.migrations_table
        ))
        .fetch_one(self.conn.pool())
        .await
        .map_err(|e| DataBridgeError::Database(format!("check legacy rows: {e}")))?;

        if already > 0 {
            return Ok(());
        }

        // Read Alembic version IDs
        let ids: Vec<String> = sqlx::query_scalar("SELECT version_num FROM alembic_version")
            .fetch_all(self.conn.pool())
            .await
            .map_err(|e| DataBridgeError::Database(format!("fetch alembic versions: {e}")))?;

        // Seed each ID
        for id in &ids {
            let sql = format!(
                "INSERT INTO {} (migration_id, source, checksum) \
                 VALUES ($1, 'legacy', '') ON CONFLICT DO NOTHING",
                self.migrations_table
            );
            sqlx::query(&sql)
                .bind(id)
                .execute(self.conn.pool())
                .await
                .map_err(|e| DataBridgeError::Database(format!("seed legacy id {id}: {e}")))?;
        }

        tracing::info!("Bootstrapped {} legacy migration IDs", ids.len());
        Ok(())
    }

    /// Returns version strings of all applied migrations in sorted order.
    pub async fn applied_migrations(&self) -> Result<Vec<String>> {
        let sql = format!(
            "SELECT migration_id FROM {} ORDER BY migration_id",
            self.migrations_table
        );

        let rows = sqlx::query_scalar::<_, String>(&sql)
            .fetch_all(self.conn.pool())
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to fetch applied migrations: {}", e))
            })?;

        Ok(rows)
    }

    /// Native-only migration IDs sorted -- used by `down` to find the last one.
    pub async fn applied_native_ids(&self) -> Result<Vec<String>> {
        sqlx::query_scalar::<_, String>(&format!(
            "SELECT migration_id FROM {} \
             WHERE source = 'native' ORDER BY migration_id",
            self.migrations_table
        ))
        .fetch_all(self.conn.pool())
        .await
        .map_err(|e| DataBridgeError::Database(format!("fetch native ids: {e}")))
    }

    /// All entries in the tracking table, sorted by `migration_id`.
    pub async fn all_entries(&self) -> Result<Vec<MigrationEntry>> {
        let rows = sqlx::query(&format!(
            "SELECT migration_id, source, applied_at, checksum \
             FROM {} ORDER BY migration_id",
            self.migrations_table
        ))
        .fetch_all(self.conn.pool())
        .await
        .map_err(|e| DataBridgeError::Database(format!("fetch entries: {e}")))?;

        rows.into_iter()
            .map(|row| {
                let migration_id: String = row
                    .try_get("migration_id")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                let source_str: String = row
                    .try_get("source")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                let applied_at: DateTime<Utc> = row
                    .try_get("applied_at")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                let checksum: String = row
                    .try_get("checksum")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                Ok(MigrationEntry {
                    migration_id,
                    source: MigrationSource::from_str(&source_str),
                    applied_at,
                    checksum,
                })
            })
            .collect()
    }

    /// Returns full migration details for applied migrations.
    pub async fn applied_migrations_with_details(&self) -> Result<Vec<Migration>> {
        let sql = format!(
            "SELECT migration_id, source, applied_at, checksum FROM {} ORDER BY migration_id",
            self.migrations_table
        );

        let rows = sqlx::query(&sql)
            .fetch_all(self.conn.pool())
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to fetch applied migrations: {}", e))
            })?;

        let mut migrations = Vec::new();
        for row in rows {
            let version: String = row.try_get("migration_id").map_err(|e| {
                DataBridgeError::Database(format!("Failed to get migration_id: {}", e))
            })?;
            let applied_at: DateTime<Utc> = row.try_get("applied_at").map_err(|e| {
                DataBridgeError::Database(format!("Failed to get applied_at: {}", e))
            })?;
            let checksum: String = row
                .try_get("checksum")
                .map_err(|e| DataBridgeError::Database(format!("Failed to get checksum: {}", e)))?;

            migrations.push(Migration {
                version,
                name: String::new(),
                up: String::new(),
                down: String::new(),
                applied_at: Some(applied_at),
                checksum,
            });
        }

        Ok(migrations)
    }

    /// Returns migrations not yet applied by comparing with provided list.
    pub async fn pending_migrations(&self, all_migrations: &[Migration]) -> Result<Vec<Migration>> {
        let applied = self.applied_migrations().await?;
        let applied_set: std::collections::HashSet<_> = applied.into_iter().collect();

        let pending: Vec<Migration> = all_migrations
            .iter()
            .filter(|m| !applied_set.contains(&m.version))
            .cloned()
            .collect();

        Ok(pending)
    }

    /// Verifies migration checksum matches stored value to detect modifications.
    async fn verify_checksum(&self, migration: &Migration) -> Result<bool> {
        let sql = format!(
            "SELECT checksum FROM {} WHERE migration_id = $1",
            self.migrations_table
        );

        let checksum: Option<String> = sqlx::query_scalar(&sql)
            .bind(&migration.version)
            .fetch_optional(self.conn.pool())
            .await
            .map_err(|e| DataBridgeError::Database(format!("Failed to verify checksum: {}", e)))?;

        match checksum {
            Some(stored_checksum) => Ok(stored_checksum == migration.checksum),
            None => Ok(true), // Migration not applied yet
        }
    }

    /// Applies a migration by executing its up SQL in a transaction.
    pub async fn apply(&self, migration: &Migration) -> Result<()> {
        // Verify checksum if migration was already applied
        if !self.verify_checksum(migration).await? {
            return Err(DataBridgeError::Validation(
                format!("Checksum mismatch for migration {}. The migration file has been modified after being applied.", migration.version)
            ));
        }

        // Begin transaction
        let mut tx = self.conn.pool().begin().await.map_err(|e| {
            DataBridgeError::Database(format!("Failed to begin transaction: {}", e))
        })?;

        // Split SQL into individual statements and execute each
        let statements = split_sql_statements(&migration.up);
        for (idx, statement) in statements.iter().enumerate() {
            if !statement.trim().is_empty() {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        DataBridgeError::Database(format!(
                            "Failed to apply migration {} (statement {}): {}",
                            migration.version,
                            idx + 1,
                            e
                        ))
                    })?;
            }
        }

        // Record migration with source tracking
        let insert_sql = format!(
            "INSERT INTO {} (migration_id, source, checksum) VALUES ($1, 'native', $2)",
            self.migrations_table
        );

        sqlx::query(&insert_sql)
            .bind(&migration.version)
            .bind(&migration.checksum)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataBridgeError::Database(format!("Failed to record migration: {}", e)))?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| DataBridgeError::Database(format!("Failed to commit migration: {}", e)))?;

        tracing::info!(
            "Applied migration: {} - {}",
            migration.version,
            migration.name
        );
        Ok(())
    }

    /// Reverts a migration by executing its down SQL in a transaction.
    pub async fn revert(&self, migration: &Migration) -> Result<()> {
        // Begin transaction
        let mut tx = self.conn.pool().begin().await.map_err(|e| {
            DataBridgeError::Database(format!("Failed to begin transaction: {}", e))
        })?;

        // Split SQL into individual statements and execute each
        let statements = split_sql_statements(&migration.down);
        for (idx, statement) in statements.iter().enumerate() {
            if !statement.trim().is_empty() {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        DataBridgeError::Database(format!(
                            "Failed to revert migration {} (statement {}): {}",
                            migration.version,
                            idx + 1,
                            e
                        ))
                    })?;
            }
        }

        // Remove migration record
        let delete_sql = format!(
            "DELETE FROM {} WHERE migration_id = $1",
            self.migrations_table
        );

        sqlx::query(&delete_sql)
            .bind(&migration.version)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to remove migration record: {}", e))
            })?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| DataBridgeError::Database(format!("Failed to commit rollback: {}", e)))?;

        tracing::info!(
            "Reverted migration: {} - {}",
            migration.version,
            migration.name
        );
        Ok(())
    }

    /// Applies all pending migrations sequentially, returning applied versions.
    pub async fn migrate(&self, migrations: &[Migration]) -> Result<Vec<String>> {
        let pending = self.pending_migrations(migrations).await?;

        if pending.is_empty() {
            tracing::info!("No pending migrations to apply");
            return Ok(Vec::new());
        }

        let mut applied = Vec::new();

        for migration in &pending {
            self.apply(migration).await?;
            applied.push(migration.version.clone());
        }

        tracing::info!("Applied {} migrations", applied.len());
        Ok(applied)
    }

    /// Apply all pending migrations from `migrations_dir` in version order.
    ///
    /// Returns the list of newly applied migration IDs.
    pub async fn up(&self, migrations_dir: &Path) -> Result<Vec<String>> {
        self.init().await?;

        let all = Self::load_from_directory(migrations_dir)?;
        let applied = self.applied_migrations().await?;
        let applied_set: std::collections::HashSet<_> = applied.into_iter().collect();

        let pending: Vec<_> = all
            .iter()
            .filter(|m| !applied_set.contains(&m.version))
            .collect();

        if pending.is_empty() {
            tracing::info!("No pending migrations");
            return Ok(Vec::new());
        }

        let mut applied_ids = Vec::new();
        for migration in &pending {
            self.apply_one(&migration.version, &migration.up).await?;
            applied_ids.push(migration.version.clone());
            tracing::info!("Applied {}", migration.version);
        }

        Ok(applied_ids)
    }

    /// Apply a single migration by version and up SQL (simple split, no dollar quotes).
    async fn apply_one(&self, version: &str, up_sql: &str) -> Result<()> {
        let mut tx = self
            .conn
            .pool()
            .begin()
            .await
            .map_err(|e| DataBridgeError::Database(format!("begin tx: {e}")))?;

        for stmt in split_statements_simple(up_sql) {
            sqlx::query(&stmt)
                .execute(&mut *tx)
                .await
                .map_err(|e| DataBridgeError::Database(format!("apply {version} stmt: {e}")))?;
        }

        let checksum = sha256_hex(up_sql);
        sqlx::query(&format!(
            "INSERT INTO {} (migration_id, source, checksum) \
             VALUES ($1, 'native', $2)",
            self.migrations_table
        ))
        .bind(version)
        .bind(&checksum)
        .execute(&mut *tx)
        .await
        .map_err(|e| DataBridgeError::Database(format!("record {version}: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| DataBridgeError::Database(format!("commit: {e}")))?;

        Ok(())
    }

    /// Revert the last applied native migration.
    ///
    /// Returns the reverted migration ID, or `None` if nothing to revert.
    pub async fn down(&self, migrations_dir: &Path) -> Result<Option<String>> {
        self.init().await?;

        let applied = self.applied_native_ids().await?;
        let last = match applied.last() {
            Some(id) => id.clone(),
            None => {
                tracing::info!("No native migrations to revert");
                return Ok(None);
            }
        };

        let all = Self::load_from_directory(migrations_dir)?;
        let migration = all.iter().find(|m| m.version == last).ok_or_else(|| {
            DataBridgeError::Validation(format!(
                "Migration file for {last} not found in {}",
                migrations_dir.display()
            ))
        })?;

        self.revert_one(&migration.version, &migration.down).await?;
        Ok(Some(last))
    }

    /// Revert a single migration by version and down SQL (simple split).
    async fn revert_one(&self, version: &str, down_sql: &str) -> Result<()> {
        let mut tx = self
            .conn
            .pool()
            .begin()
            .await
            .map_err(|e| DataBridgeError::Database(format!("begin tx: {e}")))?;

        for stmt in split_statements_simple(down_sql) {
            sqlx::query(&stmt)
                .execute(&mut *tx)
                .await
                .map_err(|e| DataBridgeError::Database(format!("revert {version} stmt: {e}")))?;
        }

        sqlx::query(&format!(
            "DELETE FROM {} WHERE migration_id = $1",
            self.migrations_table
        ))
        .bind(version)
        .execute(&mut *tx)
        .await
        .map_err(|e| DataBridgeError::Database(format!("delete {version}: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| DataBridgeError::Database(format!("commit: {e}")))?;

        tracing::info!("Reverted {version}");
        Ok(())
    }

    /// Reverts the last N applied migrations in reverse order.
    pub async fn rollback(&self, migrations: &[Migration], count: usize) -> Result<Vec<String>> {
        let applied = self.applied_migrations().await?;

        if applied.is_empty() {
            tracing::info!("No migrations to rollback");
            return Ok(Vec::new());
        }

        // Get the last N migrations to revert
        let to_revert_count = count.min(applied.len());
        let to_revert_versions: Vec<String> = applied
            .iter()
            .rev()
            .take(to_revert_count)
            .cloned()
            .collect();

        // Find the corresponding Migration objects
        let mut migrations_to_revert = Vec::new();
        for version in &to_revert_versions {
            if let Some(migration) = migrations.iter().find(|m| &m.version == version) {
                migrations_to_revert.push(migration.clone());
            } else {
                return Err(DataBridgeError::Validation(format!(
                    "Migration {} is applied but not found in migration files",
                    version
                )));
            }
        }

        let mut reverted = Vec::new();

        for migration in &migrations_to_revert {
            self.revert(migration).await?;
            reverted.push(migration.version.clone());
        }

        tracing::info!("Reverted {} migrations", reverted.len());
        Ok(reverted)
    }

    /// Loads all .sql migration files from directory, sorted by version.
    pub fn load_from_directory(path: &Path) -> Result<Vec<Migration>> {
        if !path.exists() {
            return Err(DataBridgeError::Internal(format!(
                "Migration directory does not exist: {}",
                path.display()
            )));
        }

        if !path.is_dir() {
            return Err(DataBridgeError::Internal(format!(
                "Path is not a directory: {}",
                path.display()
            )));
        }

        let mut migrations = Vec::new();

        let entries = fs::read_dir(path)
            .map_err(|e| DataBridgeError::Internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                DataBridgeError::Internal(format!("Failed to read directory entry: {}", e))
            })?;
            let file_path = entry.path();

            // Skip non-SQL files
            if file_path
                .extension()
                .and_then(|s: &std::ffi::OsStr| s.to_str())
                != Some("sql")
            {
                continue;
            }

            // Skip hidden files
            if file_path
                .file_name()
                .and_then(|s: &std::ffi::OsStr| s.to_str())
                .map(|s: &str| s.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            match Migration::from_file(&file_path) {
                Ok(migration) => {
                    migrations.push(migration);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load migration from {}: {}",
                        file_path.display(),
                        e
                    );
                }
            }
        }

        // Sort migrations by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        tracing::info!(
            "Loaded {} migrations from {}",
            migrations.len(),
            path.display()
        );
        Ok(migrations)
    }

    /// Returns current migration status with applied and pending lists.
    pub async fn status(&self, migrations: &[Migration]) -> Result<MigrationStatus> {
        let applied = self.applied_migrations().await?;
        let pending = self.pending_migrations(migrations).await?;

        Ok(MigrationStatus {
            applied,
            pending: pending.iter().map(|m| m.version.clone()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires PostgreSQL database (set POSTGRES_URL)"]
    async fn migrate_up_applies_in_order() {
        let url = std::env::var("POSTGRES_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string());
        let runner = MigrationRunner::connect(&url).await.unwrap();

        let dir = tempfile::TempDir::new().unwrap();
        let files = [
            (
                "20260101_000001_test_a.sql",
                "SELECT 1;",
                "SELECT 0;",
                "test_a",
            ),
            (
                "20260101_000002_test_b.sql",
                "SELECT 1;",
                "SELECT 0;",
                "test_b",
            ),
            (
                "20260101_000003_test_c.sql",
                "SELECT 1;",
                "SELECT 0;",
                "test_c",
            ),
        ];

        fn migration_sql(up: &str, down: &str, desc: &str) -> String {
            format!("-- Description: {desc}\n-- UP\n{up}\n-- DOWN\n{down}\n")
        }

        for (name, up, down, desc) in &files {
            let content = migration_sql(up, down, desc);
            std::fs::write(dir.path().join(name), content).unwrap();
        }

        let applied = runner.up(dir.path()).await.unwrap();
        assert_eq!(applied.len(), 3);
        let mut sorted = applied.clone();
        sorted.sort();
        assert_eq!(applied, sorted);

        let all = runner.applied_native_ids().await.unwrap();
        for (name, _, _, _) in &files {
            let version = name
                .trim_end_matches(".sql")
                .splitn(3, '_')
                .take(2)
                .collect::<Vec<_>>()
                .join("_");
            assert!(all.contains(&version), "missing version {version}");
        }

        // Cleanup
        let conn = Connection::new(&url, PoolConfig::default()).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS _migrations")
            .execute(conn.pool())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL database (set POSTGRES_URL)"]
    async fn migrate_up_idempotent() {
        let url = std::env::var("POSTGRES_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string());
        let runner = MigrationRunner::connect(&url).await.unwrap();

        let dir = tempfile::TempDir::new().unwrap();

        fn migration_sql(up: &str, down: &str, desc: &str) -> String {
            format!("-- Description: {desc}\n-- UP\n{up}\n-- DOWN\n{down}\n")
        }

        let content = migration_sql("SELECT 1;", "SELECT 0;", "idempotent_test");
        std::fs::write(dir.path().join("20260101_000001_idempotent.sql"), content).unwrap();

        let first = runner.up(dir.path()).await.unwrap();
        assert_eq!(first.len(), 1);

        let second = runner.up(dir.path()).await.unwrap();
        assert_eq!(second.len(), 0);

        // Cleanup
        let conn = Connection::new(&url, PoolConfig::default()).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS _migrations")
            .execute(conn.pool())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL database (set POSTGRES_URL)"]
    async fn migrate_down_reverts_last() {
        let url = std::env::var("POSTGRES_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string());
        let runner = MigrationRunner::connect(&url).await.unwrap();

        let dir = tempfile::TempDir::new().unwrap();

        fn migration_sql(up: &str, down: &str, desc: &str) -> String {
            format!("-- Description: {desc}\n-- UP\n{up}\n-- DOWN\n{down}\n")
        }

        let content = migration_sql("SELECT 1;", "SELECT 0;", "down_test");
        std::fs::write(dir.path().join("20260101_000001_down_test.sql"), content).unwrap();

        runner.up(dir.path()).await.unwrap();
        let reverted = runner.down(dir.path()).await.unwrap();
        assert!(reverted.is_some());

        let remaining = runner.applied_native_ids().await.unwrap();
        assert!(remaining.is_empty());

        // Cleanup
        let conn = Connection::new(&url, PoolConfig::default()).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS _migrations")
            .execute(conn.pool())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL database (set POSTGRES_URL)"]
    async fn bootstrap_alembic_versions() {
        let url = std::env::var("POSTGRES_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string());
        let conn = Connection::new(&url, PoolConfig::default()).await.unwrap();

        // Create a fake alembic_version table with 8 rows
        sqlx::query("DROP TABLE IF EXISTS alembic_version")
            .execute(conn.pool())
            .await
            .unwrap();
        sqlx::query("CREATE TABLE alembic_version (version_num TEXT PRIMARY KEY)")
            .execute(conn.pool())
            .await
            .unwrap();
        for i in 0..8u32 {
            sqlx::query("INSERT INTO alembic_version (version_num) VALUES ($1)")
                .bind(format!("abc{:03}", i))
                .execute(conn.pool())
                .await
                .unwrap();
        }

        let runner = MigrationRunner::new(conn.clone(), None);
        runner.init().await.unwrap();

        let entries = runner.all_entries().await.unwrap();
        let legacy_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.source == MigrationSource::Legacy)
            .collect();
        assert_eq!(legacy_entries.len(), 8);

        // Cleanup
        sqlx::query("DROP TABLE IF EXISTS alembic_version")
            .execute(conn.pool())
            .await
            .unwrap();
        sqlx::query("DROP TABLE IF EXISTS _migrations")
            .execute(conn.pool())
            .await
            .unwrap();
    }
}
