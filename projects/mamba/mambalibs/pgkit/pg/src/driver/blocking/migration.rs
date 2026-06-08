//! Blocking façade over [`crate::MigrationRunner`].

use std::path::Path;
use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::migration::{
    Migration, MigrationEntry, MigrationRunner as AsyncMigrationRunner, MigrationStatus,
};
use crate::Result;

use super::Connection;

/// Blocking façade over [`AsyncMigrationRunner`].
pub struct MigrationRunner {
    inner: AsyncMigrationRunner,
    rt: Arc<Runtime>,
}

impl std::fmt::Debug for MigrationRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("blocking::MigrationRunner")
            .finish_non_exhaustive()
    }
}

impl MigrationRunner {
    /// Creates a new runner sharing the given blocking connection's runtime.
    pub fn new(conn: Connection, migrations_table: Option<String>) -> Self {
        let rt = conn.runtime();
        let inner = AsyncMigrationRunner::new(conn.as_async().clone(), migrations_table);
        Self { inner, rt }
    }

    /// Connects to the database and returns a runner with default tracking table.
    pub fn connect(database_url: &str) -> Result<Self> {
        let conn = Connection::new(database_url, crate::PoolConfig::default())?;
        Ok(Self::new(conn, None))
    }

    /// Initializes the migrations tracking table.
    pub fn init(&self) -> Result<()> {
        self.rt.block_on(self.inner.init())
    }

    /// Returns version strings of all applied migrations.
    pub fn applied_migrations(&self) -> Result<Vec<String>> {
        self.rt.block_on(self.inner.applied_migrations())
    }

    /// Returns IDs of native (non-legacy) migrations only.
    pub fn applied_native_ids(&self) -> Result<Vec<String>> {
        self.rt.block_on(self.inner.applied_native_ids())
    }

    /// Returns every tracked migration entry (native + legacy).
    pub fn all_entries(&self) -> Result<Vec<MigrationEntry>> {
        self.rt.block_on(self.inner.all_entries())
    }

    /// Returns applied migrations rehydrated as full [`Migration`] objects.
    pub fn applied_migrations_with_details(&self) -> Result<Vec<Migration>> {
        self.rt
            .block_on(self.inner.applied_migrations_with_details())
    }

    /// Returns migrations from `all_migrations` that have not yet been applied.
    pub fn pending_migrations(&self, all_migrations: &[Migration]) -> Result<Vec<Migration>> {
        self.rt
            .block_on(self.inner.pending_migrations(all_migrations))
    }

    /// Applies a single migration.
    pub fn apply(&self, migration: &Migration) -> Result<()> {
        self.rt.block_on(self.inner.apply(migration))
    }

    /// Reverts a single migration.
    pub fn revert(&self, migration: &Migration) -> Result<()> {
        self.rt.block_on(self.inner.revert(migration))
    }

    /// Applies every migration in order, skipping ones already applied.
    pub fn migrate(&self, migrations: &[Migration]) -> Result<Vec<String>> {
        self.rt.block_on(self.inner.migrate(migrations))
    }

    /// Applies all pending migrations under `migrations_dir`.
    pub fn up(&self, migrations_dir: &Path) -> Result<Vec<String>> {
        self.rt.block_on(self.inner.up(migrations_dir))
    }

    /// Reverts the most recent migration found in `migrations_dir`.
    pub fn down(&self, migrations_dir: &Path) -> Result<Option<String>> {
        self.rt.block_on(self.inner.down(migrations_dir))
    }

    /// Rolls back the most recent `count` migrations.
    pub fn rollback(&self, migrations: &[Migration], count: usize) -> Result<Vec<String>> {
        self.rt.block_on(self.inner.rollback(migrations, count))
    }

    /// Loads `.sql` migration files from a directory (delegate — pure sync).
    pub fn load_from_directory(path: &Path) -> Result<Vec<Migration>> {
        AsyncMigrationRunner::load_from_directory(path)
    }

    /// Returns a structured status summary.
    pub fn status(&self, migrations: &[Migration]) -> Result<MigrationStatus> {
        self.rt.block_on(self.inner.status(migrations))
    }
}
