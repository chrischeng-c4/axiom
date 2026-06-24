//! Auto-migration detection for schema changes.
//!
//! Compares Python model definitions with actual database schema
//! and generates migration SQL automatically.

use crate::schema::{
    ColumnInfo, ColumnType, ForeignKeyInfo, IndexInfo, SchemaDiff, SchemaInspector, TableInfo,
};
use crate::{Connection, DataBridgeError, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

// ============================================================================
// Model Definition
// ============================================================================

/// Represents a field definition from a Python model.
#[derive(Debug, Clone)]
pub struct ModelField {
    /// Field name
    pub name: String,
    /// SQL data type
    pub data_type: ColumnType,
    /// Is nullable
    pub nullable: bool,
    /// Default value expression
    pub default: Option<String>,
    /// Is primary key
    pub is_primary_key: bool,
    /// Is unique
    pub is_unique: bool,
    /// Foreign key reference (table.column)
    pub foreign_key: Option<ForeignKeyRef>,
}

/// Foreign key reference.
#[derive(Debug, Clone)]
pub struct ForeignKeyRef {
    /// Referenced table
    pub table: String,
    /// Referenced column
    pub column: String,
    /// ON DELETE action
    pub on_delete: String,
    /// ON UPDATE action
    pub on_update: String,
}

/// Represents a Python model (Document) definition.
#[derive(Debug, Clone)]
pub struct ModelDefinition {
    /// Table name
    pub table_name: String,
    /// Schema name (default: "public")
    pub schema_name: String,
    /// Fields
    pub fields: Vec<ModelField>,
    /// Indexes
    pub indexes: Vec<ModelIndex>,
}

/// Index definition from model.
#[derive(Debug, Clone)]
pub struct ModelIndex {
    /// Index name
    pub name: String,
    /// Columns
    pub columns: Vec<String>,
    /// Is unique
    pub is_unique: bool,
}

impl ModelDefinition {
    /// Create a new model definition.
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            schema_name: "public".to_string(),
            fields: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /// Set schema name.
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema_name = schema.into();
        self
    }

    /// Add a field.
    pub fn field(mut self, field: ModelField) -> Self {
        self.fields.push(field);
        self
    }

    /// Add an index.
    pub fn index(mut self, index: ModelIndex) -> Self {
        self.indexes.push(index);
        self
    }

    /// Convert to TableInfo for schema comparison.
    pub fn to_table_info(&self) -> TableInfo {
        let columns: Vec<ColumnInfo> = self
            .fields
            .iter()
            .map(|f| ColumnInfo {
                name: f.name.clone(),
                data_type: f.data_type.clone(),
                nullable: f.nullable,
                default: f.default.clone(),
                is_primary_key: f.is_primary_key,
                is_unique: f.is_unique,
            })
            .collect();

        let indexes: Vec<IndexInfo> = self
            .indexes
            .iter()
            .map(|i| IndexInfo {
                name: i.name.clone(),
                columns: i.columns.clone(),
                is_unique: i.is_unique,
                index_type: "btree".to_string(),
            })
            .collect();

        let foreign_keys: Vec<ForeignKeyInfo> = self
            .fields
            .iter()
            .filter_map(|f| {
                f.foreign_key.as_ref().map(|fk| ForeignKeyInfo {
                    name: format!("fk_{}_{}", self.table_name, f.name),
                    columns: vec![f.name.clone()],
                    referenced_table: fk.table.clone(),
                    referenced_columns: vec![fk.column.clone()],
                    on_delete: fk.on_delete.clone(),
                    on_update: fk.on_update.clone(),
                })
            })
            .collect();

        TableInfo {
            name: self.table_name.clone(),
            schema: self.schema_name.clone(),
            columns,
            indexes,
            foreign_keys,
        }
    }
}

impl ModelField {
    /// Create a new field.
    pub fn new(name: impl Into<String>, data_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            default: None,
            is_primary_key: false,
            is_unique: false,
            foreign_key: None,
        }
    }

    /// Mark as primary key.
    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.nullable = false;
        self
    }

    /// Mark as not nullable.
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Mark as unique.
    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    /// Set default value.
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Set foreign key reference.
    pub fn references(mut self, table: impl Into<String>, column: impl Into<String>) -> Self {
        self.foreign_key = Some(ForeignKeyRef {
            table: table.into(),
            column: column.into(),
            on_delete: "NO ACTION".to_string(),
            on_update: "NO ACTION".to_string(),
        });
        self
    }
}

// ============================================================================
// Auto-Detection
// ============================================================================

/// Configuration for auto-detection.
#[derive(Debug, Clone)]
pub struct AutoDetectConfig {
    /// Schema to inspect (default: "public")
    pub schema: String,
    /// Exclude tables matching patterns
    pub exclude_tables: Vec<String>,
    /// Include only tables matching patterns
    pub include_tables: Option<Vec<String>>,
    /// Generate verbose comments in SQL
    pub verbose_comments: bool,
}

impl Default for AutoDetectConfig {
    fn default() -> Self {
        Self {
            schema: "public".to_string(),
            exclude_tables: vec!["_migrations".to_string()],
            include_tables: None,
            verbose_comments: true,
        }
    }
}

/// Result of auto-detection.
#[derive(Debug, Clone)]
pub struct AutoDetectResult {
    /// Schema diff between current and desired
    pub diff: SchemaDiff,
    /// Generated UP SQL
    pub up_sql: String,
    /// Generated DOWN SQL
    pub down_sql: String,
    /// Summary of changes
    pub summary: Vec<String>,
}

impl AutoDetectResult {
    /// Check if there are any changes detected.
    pub fn has_changes(&self) -> bool {
        !self.diff.is_empty()
    }

    /// Get the number of changes.
    pub fn change_count(&self) -> usize {
        self.diff.changes.len()
    }
}

/// Auto-detect schema changes between models and database.
pub struct AutoDetector {
    conn: Connection,
    config: AutoDetectConfig,
}

impl AutoDetector {
    /// Create a new auto-detector.
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            config: AutoDetectConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(conn: Connection, config: AutoDetectConfig) -> Self {
        Self { conn, config }
    }

    /// Detect changes between model definitions and database schema.
    pub async fn detect(&self, models: &[ModelDefinition]) -> Result<AutoDetectResult> {
        let inspector = SchemaInspector::new(self.conn.clone());

        // Get current database schema
        let current_tables = self.get_current_schema(&inspector).await?;

        // Get desired schema from models
        let desired_tables: Vec<TableInfo> = models.iter().map(|m| m.to_table_info()).collect();

        // Compare schemas
        let diff = SchemaDiff::compare(&current_tables, &desired_tables);

        // Generate SQL
        let up_sql = diff.generate_up_sql();
        let down_sql = diff.generate_down_sql();

        // Generate summary
        let summary = self.generate_summary(&diff);

        Ok(AutoDetectResult {
            diff,
            up_sql,
            down_sql,
            summary,
        })
    }

    /// Get current database schema.
    async fn get_current_schema(&self, inspector: &SchemaInspector) -> Result<Vec<TableInfo>> {
        let tables = inspector.list_tables(Some(&self.config.schema)).await?;

        let mut table_infos = Vec::new();

        for table_name in tables {
            // Check exclusions
            if self.config.exclude_tables.contains(&table_name) {
                continue;
            }

            // Check inclusions
            if let Some(ref include) = self.config.include_tables {
                if !include.contains(&table_name) {
                    continue;
                }
            }

            match inspector
                .inspect_table(&table_name, Some(&self.config.schema))
                .await
            {
                Ok(info) => table_infos.push(info),
                Err(e) => {
                    tracing::warn!("Failed to inspect table {}: {}", table_name, e);
                }
            }
        }

        Ok(table_infos)
    }

    /// Generate human-readable summary of changes.
    fn generate_summary(&self, diff: &SchemaDiff) -> Vec<String> {
        use crate::schema::TableChange;
        let mut summary = Vec::new();

        for change in &diff.changes {
            match change {
                TableChange::Created(table) => {
                    summary.push(format!(
                        "CREATE TABLE {} ({} columns)",
                        table.name,
                        table.columns.len()
                    ));
                }
                TableChange::Dropped(name) => {
                    summary.push(format!("DROP TABLE {}", name));
                }
                TableChange::Altered {
                    table_name,
                    column_changes,
                    index_changes,
                    foreign_key_changes,
                } => {
                    for change in column_changes {
                        use crate::schema::ColumnChange;
                        match change {
                            ColumnChange::Added(col) => {
                                summary.push(format!(
                                    "ADD COLUMN {}.{} ({})",
                                    table_name,
                                    col.name,
                                    col.data_type.to_sql()
                                ));
                            }
                            ColumnChange::Removed(col) => {
                                summary.push(format!("DROP COLUMN {}.{}", table_name, col.name));
                            }
                            ColumnChange::TypeChanged { old, new } => {
                                summary.push(format!(
                                    "ALTER COLUMN {}.{} TYPE {} -> {}",
                                    table_name,
                                    new.name,
                                    old.data_type.to_sql(),
                                    new.data_type.to_sql()
                                ));
                            }
                            ColumnChange::NullabilityChanged { new, .. } => {
                                let constraint = if new.nullable {
                                    "DROP NOT NULL"
                                } else {
                                    "SET NOT NULL"
                                };
                                summary.push(format!(
                                    "ALTER COLUMN {}.{} {}",
                                    table_name, new.name, constraint
                                ));
                            }
                            ColumnChange::DefaultChanged { new, .. } => match &new.default {
                                Some(d) => summary.push(format!(
                                    "ALTER COLUMN {}.{} SET DEFAULT {}",
                                    table_name, new.name, d
                                )),
                                None => summary.push(format!(
                                    "ALTER COLUMN {}.{} DROP DEFAULT",
                                    table_name, new.name
                                )),
                            },
                        }
                    }

                    for change in index_changes {
                        use crate::schema::IndexChange;
                        match change {
                            IndexChange::Added(idx) => {
                                summary.push(format!("CREATE INDEX {}", idx.name));
                            }
                            IndexChange::Removed(idx) => {
                                summary.push(format!("DROP INDEX {}", idx.name));
                            }
                        }
                    }

                    for change in foreign_key_changes {
                        use crate::schema::ForeignKeyChange;
                        match change {
                            ForeignKeyChange::Added(fk) => {
                                summary.push(format!(
                                    "ADD FOREIGN KEY {} -> {}",
                                    fk.name, fk.referenced_table
                                ));
                            }
                            ForeignKeyChange::Removed(fk) => {
                                summary.push(format!("DROP FOREIGN KEY {}", fk.name));
                            }
                        }
                    }
                }
            }
        }

        summary
    }

    /// Generate a migration file from detected changes.
    pub fn generate_migration_file(
        &self,
        result: &AutoDetectResult,
        name: &str,
        output_dir: &Path,
    ) -> Result<String> {
        if !result.has_changes() {
            return Err(DataBridgeError::Validation(
                "No changes detected".to_string(),
            ));
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.sql", timestamp, name.replace(' ', "_"));
        let filepath = output_dir.join(&filename);

        let mut content = String::new();

        // Header
        content.push_str(&format!("-- Migration: {}_{}\n", timestamp, name));
        content.push_str(&format!("-- Description: {}\n", name));
        content.push_str("-- Auto-generated by cclab-titan\n\n");

        // Summary
        content.push_str("-- Changes:\n");
        for summary in &result.summary {
            content.push_str(&format!("--   {}\n", summary));
        }
        content.push('\n');

        // UP section
        content.push_str("-- UP\n");
        content.push_str(&result.up_sql);
        content.push_str("\n\n");

        // DOWN section
        content.push_str("-- DOWN\n");
        content.push_str(&result.down_sql);
        content.push('\n');

        // Write file
        fs::write(&filepath, &content)
            .map_err(|e| DataBridgeError::Internal(format!("Failed to write migration: {}", e)))?;

        Ok(filename)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_definition() {
        let model = ModelDefinition::new("users")
            .field(
                ModelField::new("id", ColumnType::Integer)
                    .primary_key()
                    .default("nextval('users_id_seq')"),
            )
            .field(ModelField::new("name", ColumnType::Text).not_null())
            .field(ModelField::new("email", ColumnType::Varchar(Some(255))).unique());

        let table_info = model.to_table_info();

        assert_eq!(table_info.name, "users");
        assert_eq!(table_info.columns.len(), 3);
        assert!(table_info.columns[0].is_primary_key);
        assert!(!table_info.columns[1].nullable);
        assert!(table_info.columns[2].is_unique);
    }

    #[test]
    fn test_model_field_builder() {
        let field = ModelField::new("user_id", ColumnType::Integer)
            .not_null()
            .references("users", "id");

        assert!(!field.nullable);
        assert!(field.foreign_key.is_some());
        let fk = field.foreign_key.unwrap();
        assert_eq!(fk.table, "users");
        assert_eq!(fk.column, "id");
    }

    #[test]
    fn test_auto_detect_config_default() {
        let config = AutoDetectConfig::default();
        assert_eq!(config.schema, "public");
        assert!(config.exclude_tables.contains(&"_migrations".to_string()));
    }
}
