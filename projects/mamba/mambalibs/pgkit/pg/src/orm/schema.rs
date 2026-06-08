//! Database schema introspection.
//!
//! This module provides utilities for inspecting the database schema,
//! useful for validation, documentation, and migration generation.

use crate::{Connection, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

/// PostgreSQL column data type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// SMALLINT
    SmallInt,
    /// INTEGER
    Integer,
    /// BIGINT
    BigInt,
    /// REAL
    Real,
    /// DOUBLE PRECISION
    DoublePrecision,
    /// NUMERIC(precision, scale)
    Numeric(Option<i32>, Option<i32>),
    /// VARCHAR(length)
    Varchar(Option<i32>),
    /// TEXT
    Text,
    /// BOOLEAN
    Boolean,
    /// BYTEA
    Bytea,
    /// UUID
    Uuid,
    /// DATE
    Date,
    /// TIME
    Time,
    /// TIMESTAMP
    Timestamp,
    /// TIMESTAMPTZ
    TimestampTz,
    /// JSON
    Json,
    /// JSONB
    Jsonb,
    /// ARRAY of type
    Array(Box<ColumnType>),
    /// Custom/Unknown type
    Custom(String),
}

impl ColumnType {
    /// Parse PostgreSQL type string into ColumnType enum (simple version without metadata)
    pub fn parse(type_str: &str) -> Self {
        match type_str.to_lowercase().as_str() {
            "smallint" => ColumnType::SmallInt,
            "integer" | "int" => ColumnType::Integer,
            "bigint" => ColumnType::BigInt,
            "real" => ColumnType::Real,
            "double precision" => ColumnType::DoublePrecision,
            "numeric" | "decimal" => ColumnType::Numeric(None, None),
            "character varying" | "varchar" => ColumnType::Varchar(None),
            "text" => ColumnType::Text,
            "boolean" | "bool" => ColumnType::Boolean,
            "bytea" => ColumnType::Bytea,
            "uuid" => ColumnType::Uuid,
            "date" => ColumnType::Date,
            "time" | "time without time zone" => ColumnType::Time,
            "timestamp" | "timestamp without time zone" => ColumnType::Timestamp,
            "timestamp with time zone" | "timestamptz" => ColumnType::TimestampTz,
            "json" => ColumnType::Json,
            "jsonb" => ColumnType::Jsonb,
            _ => ColumnType::Custom(type_str.to_string()),
        }
    }

    /// Convert to PostgreSQL type string
    pub fn to_sql(&self) -> String {
        match self {
            ColumnType::SmallInt => "SMALLINT".to_string(),
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Real => "REAL".to_string(),
            ColumnType::DoublePrecision => "DOUBLE PRECISION".to_string(),
            ColumnType::Numeric(precision, scale) => {
                match (precision, scale) {
                    (Some(p), Some(s)) => format!("NUMERIC({}, {})", p, s),
                    (Some(p), None) => format!("NUMERIC({})", p),
                    _ => "NUMERIC".to_string(),
                }
            }
            ColumnType::Varchar(len) => {
                match len {
                    Some(l) => format!("VARCHAR({})", l),
                    None => "VARCHAR".to_string(),
                }
            }
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Bytea => "BYTEA".to_string(),
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time => "TIME".to_string(),
            ColumnType::Timestamp => "TIMESTAMP".to_string(),
            ColumnType::TimestampTz => "TIMESTAMPTZ".to_string(),
            ColumnType::Json => "JSON".to_string(),
            ColumnType::Jsonb => "JSONB".to_string(),
            ColumnType::Array(inner) => format!("{}[]", inner.to_sql()),
            ColumnType::Custom(name) => name.clone(),
        }
    }
}

/// Represents a column in a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: ColumnType,
    /// Is nullable
    pub nullable: bool,
    /// Default value expression
    pub default: Option<String>,
    /// Is primary key
    pub is_primary_key: bool,
    /// Is unique
    pub is_unique: bool,
}

/// Represents a table index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Columns in the index
    pub columns: Vec<String>,
    /// Is unique index
    pub is_unique: bool,
    /// Index type (btree, hash, gin, gist, etc.)
    pub index_type: String,
}

/// Represents a foreign key constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    /// Constraint name
    pub name: String,
    /// Source columns
    pub columns: Vec<String>,
    /// Referenced table
    pub referenced_table: String,
    /// Referenced columns
    pub referenced_columns: Vec<String>,
    /// ON DELETE action
    pub on_delete: String,
    /// ON UPDATE action
    pub on_update: String,
}

/// Cascade rule for foreign key operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CascadeRule {
    /// Delete/update child rows automatically
    Cascade,
    /// Prevent operation if children exist
    Restrict,
    /// Set foreign key to NULL
    SetNull,
    /// Set foreign key to default value
    SetDefault,
    /// No action (check at end of transaction)
    NoAction,
}

impl CascadeRule {
    /// Parse from SQL string (e.g., "CASCADE", "RESTRICT")
    pub fn from_sql(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CASCADE" => Self::Cascade,
            "RESTRICT" => Self::Restrict,
            "SET NULL" => Self::SetNull,
            "SET DEFAULT" => Self::SetDefault,
            _ => Self::NoAction,
        }
    }

    /// Convert to SQL string
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
            Self::NoAction => "NO ACTION",
        }
    }
}

/// Back-reference representing an incoming foreign key from another table
#[derive(Debug, Clone)]
pub struct BackRef {
    /// Table that references this table (the "child" table)
    pub source_table: String,
    /// Column in source table that holds the FK
    pub source_column: String,
    /// This table being referenced (the "parent" table)
    pub target_table: String,
    /// Column in this table being referenced (usually "id")
    pub target_column: String,
    /// Constraint name
    pub constraint_name: String,
    /// What to do on delete of parent row
    pub on_delete: CascadeRule,
    /// What to do on update of parent row
    pub on_update: CascadeRule,
}

/// Configuration for a Many-to-Many relationship via join table
#[derive(Debug, Clone)]
pub struct ManyToManyConfig {
    /// Name of the join table (e.g., "user_tags")
    pub join_table: String,
    /// Column in join table referencing source table (e.g., "user_id")
    pub source_key: String,
    /// Column in join table referencing target table (e.g., "tag_id")
    pub target_key: String,
    /// Target table name (e.g., "tags")
    pub target_table: String,
    /// Column in source table being referenced (default: "id")
    pub source_reference: String,
    /// Column in target table being referenced (default: "id")
    pub target_reference: String,
    /// Cascade rule for deleting source (what happens to join table rows)
    pub on_delete: CascadeRule,
}

impl ManyToManyConfig {
    /// Creates a new ManyToManyConfig with sensible defaults.
    ///
    /// Default reference columns are "id" and cascade rule is CASCADE.
    pub fn new(
        join_table: impl Into<String>,
        source_key: impl Into<String>,
        target_key: impl Into<String>,
        target_table: impl Into<String>,
    ) -> Self {
        Self {
            join_table: join_table.into(),
            source_key: source_key.into(),
            target_key: target_key.into(),
            target_table: target_table.into(),
            source_reference: "id".to_string(),
            target_reference: "id".to_string(),
            on_delete: CascadeRule::Cascade,
        }
    }

    /// Sets the source reference column (default: "id").
    pub fn with_source_reference(mut self, col: impl Into<String>) -> Self {
        self.source_reference = col.into();
        self
    }

    /// Sets the target reference column (default: "id").
    pub fn with_target_reference(mut self, col: impl Into<String>) -> Self {
        self.target_reference = col.into();
        self
    }

    /// Sets the on_delete cascade rule (default: CASCADE).
    pub fn with_on_delete(mut self, rule: CascadeRule) -> Self {
        self.on_delete = rule;
        self
    }
}

/// Represents a database table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// Table name
    pub name: String,
    /// Schema name (default: "public")
    pub schema: String,
    /// Columns
    pub columns: Vec<ColumnInfo>,
    /// Indexes
    pub indexes: Vec<IndexInfo>,
    /// Foreign keys
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

/// Represents a change to a column.
#[derive(Debug, Clone)]
pub enum ColumnChange {
    /// Column was added to the table.
    Added(ColumnInfo),
    /// Column was removed from the table.
    Removed(ColumnInfo),
    /// Column data type was changed.
    TypeChanged { old: ColumnInfo, new: ColumnInfo },
    /// Column nullability constraint was changed.
    NullabilityChanged { old: ColumnInfo, new: ColumnInfo },
    /// Column default value was changed.
    DefaultChanged { old: ColumnInfo, new: ColumnInfo },
}

/// Represents a change to an index.
#[derive(Debug, Clone)]
pub enum IndexChange {
    /// Index was created.
    Added(IndexInfo),
    /// Index was dropped.
    Removed(IndexInfo),
}

/// Represents a change to a foreign key.
#[derive(Debug, Clone)]
pub enum ForeignKeyChange {
    /// Foreign key constraint was added.
    Added(ForeignKeyInfo),
    /// Foreign key constraint was removed.
    Removed(ForeignKeyInfo),
}

/// Represents a change to a table.
#[derive(Debug, Clone)]
pub enum TableChange {
    /// Table was created.
    Created(TableInfo),
    /// Table was dropped (contains table name).
    Dropped(String),
    /// Table was altered with column, index, or foreign key changes.
    Altered {
        /// Name of the altered table.
        table_name: String,
        /// Column modifications.
        column_changes: Vec<ColumnChange>,
        /// Index modifications.
        index_changes: Vec<IndexChange>,
        /// Foreign key modifications.
        foreign_key_changes: Vec<ForeignKeyChange>,
    },
}

/// Represents the difference between two schemas.
#[derive(Debug, Clone, Default)]
pub struct SchemaDiff {
    /// List of table changes detected.
    pub changes: Vec<TableChange>,
}

impl SchemaDiff {
    /// Creates a new empty schema diff.
    pub fn new() -> Self {
        Self { changes: Vec::new() }
    }

    /// Compares two sets of table info and generates the diff.
    ///
    /// Detects created, dropped, and altered tables including column, index, and foreign key changes.
    pub fn compare(current: &[TableInfo], desired: &[TableInfo]) -> Self {
        let mut diff = SchemaDiff::new();

        let current_tables: std::collections::HashMap<&str, &TableInfo> =
            current.iter().map(|t| (t.name.as_str(), t)).collect();
        let desired_tables: std::collections::HashMap<&str, &TableInfo> =
            desired.iter().map(|t| (t.name.as_str(), t)).collect();

        // Find dropped tables
        for name in current_tables.keys() {
            if !desired_tables.contains_key(name) {
                diff.changes.push(TableChange::Dropped(name.to_string()));
            }
        }

        // Find created and altered tables
        for (name, desired_table) in &desired_tables {
            match current_tables.get(name) {
                None => {
                    // New table
                    diff.changes.push(TableChange::Created((*desired_table).clone()));
                }
                Some(current_table) => {
                    // Check for alterations
                    let column_changes = Self::compare_columns(&current_table.columns, &desired_table.columns);
                    let index_changes = Self::compare_indexes(&current_table.indexes, &desired_table.indexes);
                    let fk_changes = Self::compare_foreign_keys(&current_table.foreign_keys, &desired_table.foreign_keys);

                    if !column_changes.is_empty() || !index_changes.is_empty() || !fk_changes.is_empty() {
                        diff.changes.push(TableChange::Altered {
                            table_name: name.to_string(),
                            column_changes,
                            index_changes,
                            foreign_key_changes: fk_changes,
                        });
                    }
                }
            }
        }

        diff
    }

    fn compare_columns(current: &[ColumnInfo], desired: &[ColumnInfo]) -> Vec<ColumnChange> {
        let mut changes = Vec::new();

        let current_cols: std::collections::HashMap<&str, &ColumnInfo> =
            current.iter().map(|c| (c.name.as_str(), c)).collect();
        let desired_cols: std::collections::HashMap<&str, &ColumnInfo> =
            desired.iter().map(|c| (c.name.as_str(), c)).collect();

        // Removed columns
        for (name, col) in &current_cols {
            if !desired_cols.contains_key(name) {
                changes.push(ColumnChange::Removed((*col).clone()));
            }
        }

        // Added or modified columns
        for (name, desired_col) in &desired_cols {
            match current_cols.get(name) {
                None => changes.push(ColumnChange::Added((*desired_col).clone())),
                Some(current_col) => {
                    // Check for type change
                    if current_col.data_type != desired_col.data_type {
                        changes.push(ColumnChange::TypeChanged {
                            old: (*current_col).clone(),
                            new: (*desired_col).clone(),
                        });
                    } else if current_col.nullable != desired_col.nullable {
                        changes.push(ColumnChange::NullabilityChanged {
                            old: (*current_col).clone(),
                            new: (*desired_col).clone(),
                        });
                    } else if current_col.default != desired_col.default {
                        changes.push(ColumnChange::DefaultChanged {
                            old: (*current_col).clone(),
                            new: (*desired_col).clone(),
                        });
                    }
                }
            }
        }

        changes
    }

    fn compare_indexes(current: &[IndexInfo], desired: &[IndexInfo]) -> Vec<IndexChange> {
        let mut changes = Vec::new();

        let current_idx: std::collections::HashMap<&str, &IndexInfo> =
            current.iter().map(|i| (i.name.as_str(), i)).collect();
        let desired_idx: std::collections::HashMap<&str, &IndexInfo> =
            desired.iter().map(|i| (i.name.as_str(), i)).collect();

        for (name, idx) in &current_idx {
            if !desired_idx.contains_key(name) {
                changes.push(IndexChange::Removed((*idx).clone()));
            }
        }

        for (name, idx) in &desired_idx {
            if !current_idx.contains_key(name) {
                changes.push(IndexChange::Added((*idx).clone()));
            }
        }

        changes
    }

    fn compare_foreign_keys(current: &[ForeignKeyInfo], desired: &[ForeignKeyInfo]) -> Vec<ForeignKeyChange> {
        let mut changes = Vec::new();

        let current_fks: std::collections::HashMap<&str, &ForeignKeyInfo> =
            current.iter().map(|f| (f.name.as_str(), f)).collect();
        let desired_fks: std::collections::HashMap<&str, &ForeignKeyInfo> =
            desired.iter().map(|f| (f.name.as_str(), f)).collect();

        for (name, fk) in &current_fks {
            if !desired_fks.contains_key(name) {
                changes.push(ForeignKeyChange::Removed((*fk).clone()));
            }
        }

        for (name, fk) in &desired_fks {
            if !current_fks.contains_key(name) {
                changes.push(ForeignKeyChange::Added((*fk).clone()));
            }
        }

        changes
    }

    /// Checks if there are any changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Generates the UP migration SQL to apply changes.
    pub fn generate_up_sql(&self) -> String {
        let mut statements = Vec::new();

        for change in &self.changes {
            match change {
                TableChange::Created(table) => {
                    statements.push(Self::generate_create_table(table));
                    // Add indexes after table creation
                    for index in &table.indexes {
                        if !index.is_unique || !table.columns.iter().any(|c| c.is_primary_key && index.columns.contains(&c.name)) {
                            statements.push(Self::generate_create_index(&table.name, index));
                        }
                    }
                    // Add foreign keys
                    for fk in &table.foreign_keys {
                        statements.push(Self::generate_add_foreign_key(&table.name, fk));
                    }
                }
                TableChange::Dropped(name) => {
                    statements.push(format!("DROP TABLE IF EXISTS \"{}\" CASCADE;", name));
                }
                TableChange::Altered { table_name, column_changes, index_changes, foreign_key_changes } => {
                    for change in column_changes {
                        statements.push(Self::generate_column_change_sql(table_name, change));
                    }
                    for change in index_changes {
                        statements.push(Self::generate_index_change_sql(table_name, change));
                    }
                    for change in foreign_key_changes {
                        statements.push(Self::generate_fk_change_sql(table_name, change));
                    }
                }
            }
        }

        statements.join("\n\n")
    }

    /// Generates the DOWN migration SQL to revert changes.
    ///
    /// Note: Dropped tables cannot be fully reconstructed and will include placeholder comments.
    pub fn generate_down_sql(&self) -> String {
        let mut statements = Vec::new();

        // Process in reverse order
        for change in self.changes.iter().rev() {
            match change {
                TableChange::Created(table) => {
                    // Reverse: drop the table
                    statements.push(format!("DROP TABLE IF EXISTS \"{}\" CASCADE;", table.name));
                }
                TableChange::Dropped(name) => {
                    // Reverse: would need to recreate, but we don't have the info
                    // Add a comment as placeholder
                    statements.push(format!("-- Cannot auto-generate: recreate table \"{}\"", name));
                }
                TableChange::Altered { table_name, column_changes, index_changes, foreign_key_changes } => {
                    // Reverse foreign key changes
                    for change in foreign_key_changes.iter().rev() {
                        statements.push(Self::generate_fk_change_sql_reverse(table_name, change));
                    }
                    // Reverse index changes
                    for change in index_changes.iter().rev() {
                        statements.push(Self::generate_index_change_sql_reverse(table_name, change));
                    }
                    // Reverse column changes
                    for change in column_changes.iter().rev() {
                        statements.push(Self::generate_column_change_sql_reverse(table_name, change));
                    }
                }
            }
        }

        statements.join("\n\n")
    }

    fn generate_create_table(table: &TableInfo) -> String {
        let mut parts = Vec::new();

        for col in &table.columns {
            let mut col_def = format!("    \"{}\" {}", col.name, col.data_type.to_sql());

            if col.is_primary_key {
                col_def.push_str(" PRIMARY KEY");
            }
            if !col.nullable && !col.is_primary_key {
                col_def.push_str(" NOT NULL");
            }
            if col.is_unique && !col.is_primary_key {
                col_def.push_str(" UNIQUE");
            }
            if let Some(ref default) = col.default {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }

            parts.push(col_def);
        }

        format!(
            "CREATE TABLE \"{}\" (\n{}\n);",
            table.name,
            parts.join(",\n")
        )
    }

    fn generate_create_index(table_name: &str, index: &IndexInfo) -> String {
        let unique = if index.is_unique { "UNIQUE " } else { "" };
        let columns: Vec<String> = index.columns.iter().map(|c| format!("\"{}\"", c)).collect();
        format!(
            "CREATE {}INDEX \"{}\" ON \"{}\" ({});",
            unique,
            index.name,
            table_name,
            columns.join(", ")
        )
    }

    fn generate_add_foreign_key(table_name: &str, fk: &ForeignKeyInfo) -> String {
        let columns: Vec<String> = fk.columns.iter().map(|c| format!("\"{}\"", c)).collect();
        let ref_columns: Vec<String> = fk.referenced_columns.iter().map(|c| format!("\"{}\"", c)).collect();
        format!(
            "ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" FOREIGN KEY ({}) REFERENCES \"{}\" ({}) ON DELETE {} ON UPDATE {};",
            table_name,
            fk.name,
            columns.join(", "),
            fk.referenced_table,
            ref_columns.join(", "),
            fk.on_delete,
            fk.on_update
        )
    }

    fn generate_column_change_sql(table_name: &str, change: &ColumnChange) -> String {
        match change {
            ColumnChange::Added(col) => {
                let mut sql = format!(
                    "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}",
                    table_name, col.name, col.data_type.to_sql()
                );
                if !col.nullable {
                    sql.push_str(" NOT NULL");
                }
                if let Some(ref default) = col.default {
                    sql.push_str(&format!(" DEFAULT {}", default));
                }
                sql.push(';');
                sql
            }
            ColumnChange::Removed(col) => {
                format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\";", table_name, col.name)
            }
            ColumnChange::TypeChanged { new, .. } => {
                format!(
                    "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" TYPE {} USING \"{}\"::{};",
                    table_name, new.name, new.data_type.to_sql(), new.name, new.data_type.to_sql()
                )
            }
            ColumnChange::NullabilityChanged { new, .. } => {
                if new.nullable {
                    format!("ALTER TABLE \"{}\" ALTER COLUMN \"{}\" DROP NOT NULL;", table_name, new.name)
                } else {
                    format!("ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET NOT NULL;", table_name, new.name)
                }
            }
            ColumnChange::DefaultChanged { new, .. } => {
                match &new.default {
                    Some(default) => format!(
                        "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET DEFAULT {};",
                        table_name, new.name, default
                    ),
                    None => format!(
                        "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" DROP DEFAULT;",
                        table_name, new.name
                    ),
                }
            }
        }
    }

    fn generate_column_change_sql_reverse(table_name: &str, change: &ColumnChange) -> String {
        match change {
            ColumnChange::Added(col) => {
                format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\";", table_name, col.name)
            }
            ColumnChange::Removed(col) => {
                let mut sql = format!(
                    "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}",
                    table_name, col.name, col.data_type.to_sql()
                );
                if !col.nullable {
                    sql.push_str(" NOT NULL");
                }
                if let Some(ref default) = col.default {
                    sql.push_str(&format!(" DEFAULT {}", default));
                }
                sql.push(';');
                sql
            }
            ColumnChange::TypeChanged { old, .. } => {
                format!(
                    "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" TYPE {} USING \"{}\"::{};",
                    table_name, old.name, old.data_type.to_sql(), old.name, old.data_type.to_sql()
                )
            }
            ColumnChange::NullabilityChanged { old, .. } => {
                if old.nullable {
                    format!("ALTER TABLE \"{}\" ALTER COLUMN \"{}\" DROP NOT NULL;", table_name, old.name)
                } else {
                    format!("ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET NOT NULL;", table_name, old.name)
                }
            }
            ColumnChange::DefaultChanged { old, .. } => {
                match &old.default {
                    Some(default) => format!(
                        "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" SET DEFAULT {};",
                        table_name, old.name, default
                    ),
                    None => format!(
                        "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" DROP DEFAULT;",
                        table_name, old.name
                    ),
                }
            }
        }
    }

    fn generate_index_change_sql(table_name: &str, change: &IndexChange) -> String {
        match change {
            IndexChange::Added(idx) => Self::generate_create_index(table_name, idx),
            IndexChange::Removed(idx) => format!("DROP INDEX IF EXISTS \"{}\";", idx.name),
        }
    }

    fn generate_index_change_sql_reverse(table_name: &str, change: &IndexChange) -> String {
        match change {
            IndexChange::Added(idx) => format!("DROP INDEX IF EXISTS \"{}\";", idx.name),
            IndexChange::Removed(idx) => Self::generate_create_index(table_name, idx),
        }
    }

    fn generate_fk_change_sql(table_name: &str, change: &ForeignKeyChange) -> String {
        match change {
            ForeignKeyChange::Added(fk) => Self::generate_add_foreign_key(table_name, fk),
            ForeignKeyChange::Removed(fk) => {
                format!("ALTER TABLE \"{}\" DROP CONSTRAINT \"{}\";", table_name, fk.name)
            }
        }
    }

    fn generate_fk_change_sql_reverse(table_name: &str, change: &ForeignKeyChange) -> String {
        match change {
            ForeignKeyChange::Added(fk) => {
                format!("ALTER TABLE \"{}\" DROP CONSTRAINT \"{}\";", table_name, fk.name)
            }
            ForeignKeyChange::Removed(fk) => Self::generate_add_foreign_key(table_name, fk),
        }
    }
}

/// Schema introspection utilities for querying database metadata.
#[derive(Debug)]
pub struct SchemaInspector {
    conn: Connection,
}

impl SchemaInspector {
    /// Creates a new schema inspector from a connection.
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Lists all tables in the specified schema (defaults to "public").
    pub async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<String>> {
        let schema_name = schema.unwrap_or("public");

        let rows = sqlx::query(
            "SELECT tablename FROM pg_tables
             WHERE schemaname = $1
             ORDER BY tablename"
        )
        .bind(schema_name)
        .fetch_all(self.conn.pool())
        .await?;

        let tables = rows
            .iter()
            .filter_map(|row| row.try_get::<String, _>("tablename").ok())
            .collect();

        Ok(tables)
    }

    /// Gets detailed information about a table including columns, indexes, and foreign keys.
    pub async fn inspect_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let schema_name = schema.unwrap_or("public");

        // Check if table exists
        if !self.table_exists(table, Some(schema_name)).await? {
            return Err(crate::DataBridgeError::Query(
                "Table does not exist".to_string()
            ));
        }

        // Get columns, indexes, and foreign keys
        let columns = self.get_columns(table, Some(schema_name)).await?;
        let indexes = self.get_indexes(table, Some(schema_name)).await?;
        let foreign_keys = self.get_foreign_keys(table, Some(schema_name)).await?;

        Ok(TableInfo {
            name: table.to_string(),
            schema: schema_name.to_string(),
            columns,
            indexes,
            foreign_keys,
        })
    }

    /// Checks if a table exists in the specified schema.
    pub async fn table_exists(&self, table: &str, schema: Option<&str>) -> Result<bool> {
        let schema_name = schema.unwrap_or("public");

        let row = sqlx::query(
            "SELECT EXISTS (
                SELECT 1 FROM pg_tables
                WHERE tablename = $1 AND schemaname = $2
             ) as exists"
        )
        .bind(table)
        .bind(schema_name)
        .fetch_one(self.conn.pool())
        .await?;

        let exists: bool = row.try_get("exists")?;
        Ok(exists)
    }

    /// Gets column information for a table including types, constraints, and defaults.
    pub async fn get_columns(&self, table: &str, schema: Option<&str>) -> Result<Vec<ColumnInfo>> {
        let schema_name = schema.unwrap_or("public");

        // Query column information from information_schema
        let rows = sqlx::query(
            "SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                c.character_maximum_length,
                c.numeric_precision,
                c.numeric_scale,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
                CASE WHEN u.column_name IS NOT NULL THEN true ELSE false END as is_unique
             FROM information_schema.columns c
             LEFT JOIN (
                 SELECT ku.column_name
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage ku
                     ON tc.constraint_name = ku.constraint_name
                     AND tc.table_schema = ku.table_schema
                 WHERE tc.constraint_type = 'PRIMARY KEY'
                     AND tc.table_name = $1
                     AND tc.table_schema = $2
             ) pk ON c.column_name = pk.column_name
             LEFT JOIN (
                 SELECT ku.column_name
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage ku
                     ON tc.constraint_name = ku.constraint_name
                     AND tc.table_schema = ku.table_schema
                 WHERE tc.constraint_type = 'UNIQUE'
                     AND tc.table_name = $1
                     AND tc.table_schema = $2
             ) u ON c.column_name = u.column_name
             WHERE c.table_name = $1 AND c.table_schema = $2
             ORDER BY c.ordinal_position"
        )
        .bind(table)
        .bind(schema_name)
        .fetch_all(self.conn.pool())
        .await?;

        let mut columns = Vec::new();
        for row in rows {
            let name: String = row.try_get("column_name")?;
            let data_type_str: String = row.try_get("data_type")?;
            let nullable_str: String = row.try_get("is_nullable")?;
            let default: Option<String> = row.try_get("column_default").ok();
            let is_primary_key: bool = row.try_get("is_primary_key")?;
            let is_unique: bool = row.try_get("is_unique")?;

            // Parse data type
            let data_type = Self::parse_column_type(
                &data_type_str,
                row.try_get("character_maximum_length").ok(),
                row.try_get("numeric_precision").ok(),
                row.try_get("numeric_scale").ok(),
            );

            columns.push(ColumnInfo {
                name,
                data_type,
                nullable: nullable_str == "YES",
                default,
                is_primary_key,
                is_unique,
            });
        }

        Ok(columns)
    }

    /// Parses PostgreSQL data type string into ColumnType enum.
    fn parse_column_type(
        type_str: &str,
        max_length: Option<i32>,
        precision: Option<i32>,
        scale: Option<i32>,
    ) -> ColumnType {
        match type_str {
            "smallint" => ColumnType::SmallInt,
            "integer" => ColumnType::Integer,
            "bigint" => ColumnType::BigInt,
            "real" => ColumnType::Real,
            "double precision" => ColumnType::DoublePrecision,
            "numeric" | "decimal" => ColumnType::Numeric(precision, scale),
            "character varying" | "varchar" => ColumnType::Varchar(max_length),
            "text" => ColumnType::Text,
            "boolean" => ColumnType::Boolean,
            "bytea" => ColumnType::Bytea,
            "uuid" => ColumnType::Uuid,
            "date" => ColumnType::Date,
            "time" | "time without time zone" => ColumnType::Time,
            "timestamp" | "timestamp without time zone" => ColumnType::Timestamp,
            "timestamp with time zone" | "timestamptz" => ColumnType::TimestampTz,
            "json" => ColumnType::Json,
            "jsonb" => ColumnType::Jsonb,
            "ARRAY" => {
                // For arrays, we'd need more parsing, but for now use Custom
                ColumnType::Custom(type_str.to_string())
            }
            _ => ColumnType::Custom(type_str.to_string()),
        }
    }

    /// Gets index information for a table including columns and uniqueness constraints.
    pub async fn get_indexes(&self, table: &str, schema: Option<&str>) -> Result<Vec<IndexInfo>> {
        let schema_name = schema.unwrap_or("public");

        // Query index information from pg_indexes and pg_class/pg_index
        let rows = sqlx::query(
            "SELECT
                i.indexname as name,
                i.indexdef as definition,
                ix.indisunique as is_unique,
                am.amname as index_type,
                ARRAY(
                    SELECT a.attname
                    FROM pg_attribute a
                    WHERE a.attrelid = ix.indexrelid
                    AND a.attnum > 0
                    ORDER BY a.attnum
                ) as columns
             FROM pg_indexes i
             JOIN pg_class c ON c.relname = i.tablename
             JOIN pg_index ix ON ix.indexrelid = (i.schemaname || '.' || i.indexname)::regclass
             JOIN pg_class ic ON ic.oid = ix.indexrelid
             JOIN pg_am am ON am.oid = ic.relam
             WHERE i.tablename = $1 AND i.schemaname = $2
             ORDER BY i.indexname"
        )
        .bind(table)
        .bind(schema_name)
        .fetch_all(self.conn.pool())
        .await?;

        let mut indexes = Vec::new();
        for row in rows {
            let name: String = row.try_get("name")?;
            let is_unique: bool = row.try_get("is_unique")?;
            let index_type: String = row.try_get("index_type")?;
            let columns: Vec<String> = row.try_get("columns")?;

            indexes.push(IndexInfo {
                name,
                columns,
                is_unique,
                index_type,
            });
        }

        Ok(indexes)
    }

    /// Gets foreign key information for a table.
    ///
    /// Queries PostgreSQL's information_schema to retrieve foreign key constraints
    /// and their associated metadata.
    pub async fn get_foreign_keys(&self, table: &str, schema: Option<&str>) -> Result<Vec<ForeignKeyInfo>> {
        let schema_name = schema.unwrap_or("public");

        let rows = sqlx::query(
            "SELECT
                tc.constraint_name,
                ARRAY_AGG(DISTINCT kcu.column_name::TEXT ORDER BY kcu.column_name::TEXT)::TEXT[] as columns,
                ccu.table_name AS referenced_table,
                ARRAY_AGG(DISTINCT ccu.column_name::TEXT ORDER BY ccu.column_name::TEXT)::TEXT[] as referenced_columns,
                rc.update_rule,
                rc.delete_rule
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            JOIN information_schema.referential_constraints AS rc
                ON rc.constraint_name = tc.constraint_name
                AND rc.constraint_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_name = $1
                AND tc.table_schema = $2
            GROUP BY tc.constraint_name, ccu.table_name, rc.update_rule, rc.delete_rule
            ORDER BY tc.constraint_name"
        )
        .bind(table)
        .bind(schema_name)
        .fetch_all(self.conn.pool())
        .await?;

        let mut foreign_keys = Vec::new();
        for row in rows {
            let name: String = row.try_get("constraint_name")?;
            let columns: Vec<String> = row.try_get("columns")?;
            let referenced_table: String = row.try_get("referenced_table")?;
            let referenced_columns: Vec<String> = row.try_get("referenced_columns")?;
            let update_rule: String = row.try_get("update_rule")?;
            let delete_rule: String = row.try_get("delete_rule")?;

            foreign_keys.push(ForeignKeyInfo {
                name,
                columns,
                referenced_table,
                referenced_columns,
                on_delete: delete_rule,
                on_update: update_rule,
            });
        }

        Ok(foreign_keys)
    }

    /// Gets all tables that reference a given table (back-references).
    ///
    /// Returns a list of back-references showing which tables have foreign keys
    /// pointing to the specified table.
    pub async fn get_backreferences(&self, table: &str, schema: Option<&str>) -> Result<Vec<BackRef>> {
        let schema_name = schema.unwrap_or("public");

        let query = r#"
            SELECT
                tc.table_name as source_table,
                kcu.column_name as source_column,
                ccu.table_name as target_table,
                ccu.column_name as target_column,
                tc.constraint_name,
                rc.delete_rule,
                rc.update_rule
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            JOIN information_schema.referential_constraints rc
                ON rc.constraint_name = tc.constraint_name
                AND rc.constraint_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND ccu.table_name = $1
                AND tc.table_schema = $2
        "#;

        let rows = sqlx::query(query)
            .bind(table)
            .bind(schema_name)
            .fetch_all(self.conn.pool())
            .await?;

        let mut backrefs = Vec::new();
        for row in rows {
            backrefs.push(BackRef {
                source_table: row.try_get("source_table")?,
                source_column: row.try_get("source_column")?,
                target_table: row.try_get("target_table")?,
                target_column: row.try_get("target_column")?,
                constraint_name: row.try_get("constraint_name")?,
                on_delete: CascadeRule::from_sql(&row.try_get::<String, _>("delete_rule")?),
                on_update: CascadeRule::from_sql(&row.try_get::<String, _>("update_rule")?),
            });
        }

        Ok(backrefs)
    }

    /// Detects potential many-to-many relationships by finding join tables.
    ///
    /// A join table is detected when:
    /// - Table has exactly 2 foreign key columns
    /// - Both FK columns together form the primary key (composite PK)
    /// - Table may have additional metadata columns
    pub async fn detect_many_to_many(
        &self,
        table: &str,
        schema: Option<&str>,
    ) -> Result<Vec<ManyToManyConfig>> {
        let schema_name = schema.unwrap_or("public");

        // Query to find tables that reference this table via FK
        // and are themselves join tables (2 FK columns as composite PK)
        let query = r#"
            WITH join_tables AS (
                SELECT DISTINCT
                    kcu.table_name as join_table,
                    array_agg(kcu.column_name ORDER BY kcu.ordinal_position) as fk_columns,
                    array_agg(ccu.table_name ORDER BY kcu.ordinal_position) as ref_tables,
                    array_agg(ccu.column_name ORDER BY kcu.ordinal_position) as ref_columns
                FROM information_schema.key_column_usage kcu
                JOIN information_schema.table_constraints tc
                    ON kcu.constraint_name = tc.constraint_name
                    AND kcu.table_schema = tc.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON tc.constraint_name = ccu.constraint_name
                    AND tc.table_schema = ccu.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY'
                    AND kcu.table_schema = $2
                    AND ccu.table_name = $1
                GROUP BY kcu.table_name
                HAVING count(*) >= 2
            )
            SELECT
                jt.join_table,
                jt.fk_columns[1] as source_key,
                jt.fk_columns[2] as target_key,
                jt.ref_tables[1] as source_table,
                jt.ref_tables[2] as target_table,
                jt.ref_columns[1] as source_ref,
                jt.ref_columns[2] as target_ref
            FROM join_tables jt
            WHERE jt.ref_tables[1] = $1
        "#;

        let rows = sqlx::query(query)
            .bind(table)
            .bind(schema_name)
            .fetch_all(self.conn.pool())
            .await?;

        let mut configs = Vec::new();
        for row in rows {
            let join_table: String = row.try_get("join_table")?;
            let source_key: String = row.try_get("source_key")?;
            let target_key: String = row.try_get("target_key")?;
            let target_table: String = row.try_get("target_table")?;
            let source_ref: String = row.try_get("source_ref")?;
            let target_ref: String = row.try_get("target_ref")?;

            configs.push(ManyToManyConfig {
                join_table,
                source_key,
                target_key,
                target_table,
                source_reference: source_ref,
                target_reference: target_ref,
                on_delete: CascadeRule::Cascade,
            });
        }

        Ok(configs)
    }
}

/// Detects potential many-to-many relationships by finding join tables (standalone function).
///
/// A join table is detected when:
/// - Table has exactly 2 foreign key columns
/// - Both FK columns together form the primary key (composite PK)
/// - Table may have additional metadata columns
pub async fn detect_many_to_many(
    pool: &PgPool,
    table: &str,
    schema: &str,
) -> Result<Vec<ManyToManyConfig>> {
    // Query to find tables that reference this table via FK
    // and are themselves join tables (2 FK columns as composite PK)
    let query = r#"
        WITH join_tables AS (
            SELECT DISTINCT
                kcu.table_name as join_table,
                array_agg(kcu.column_name ORDER BY kcu.ordinal_position) as fk_columns,
                array_agg(ccu.table_name ORDER BY kcu.ordinal_position) as ref_tables,
                array_agg(ccu.column_name ORDER BY kcu.ordinal_position) as ref_columns
            FROM information_schema.key_column_usage kcu
            JOIN information_schema.table_constraints tc
                ON kcu.constraint_name = tc.constraint_name
                AND kcu.table_schema = tc.table_schema
            JOIN information_schema.constraint_column_usage ccu
                ON tc.constraint_name = ccu.constraint_name
                AND tc.table_schema = ccu.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND kcu.table_schema = $2
                AND ccu.table_name = $1
            GROUP BY kcu.table_name
            HAVING count(*) >= 2
        )
        SELECT
            jt.join_table,
            jt.fk_columns[1] as source_key,
            jt.fk_columns[2] as target_key,
            jt.ref_tables[1] as source_table,
            jt.ref_tables[2] as target_table,
            jt.ref_columns[1] as source_ref,
            jt.ref_columns[2] as target_ref
        FROM join_tables jt
        WHERE jt.ref_tables[1] = $1
    "#;

    let rows = sqlx::query(query)
        .bind(table)
        .bind(schema)
        .fetch_all(pool)
        .await?;

    let mut configs = Vec::new();
    for row in rows {
        let join_table: String = row.try_get("join_table")?;
        let source_key: String = row.try_get("source_key")?;
        let target_key: String = row.try_get("target_key")?;
        let target_table: String = row.try_get("target_table")?;
        let source_ref: String = row.try_get("source_ref")?;
        let target_ref: String = row.try_get("target_ref")?;

        configs.push(ManyToManyConfig {
            join_table,
            source_key,
            target_key,
            target_table,
            source_reference: source_ref,
            target_reference: target_ref,
            on_delete: CascadeRule::Cascade,
        });
    }

    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_column(name: &str, data_type: ColumnType, nullable: bool) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type,
            nullable,
            default: None,
            is_primary_key: false,
            is_unique: false,
        }
    }

    fn create_test_index(name: &str, columns: Vec<&str>, is_unique: bool) -> IndexInfo {
        IndexInfo {
            name: name.to_string(),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            is_unique,
            index_type: "btree".to_string(),
        }
    }

    fn create_test_foreign_key(name: &str, columns: Vec<&str>, referenced_table: &str) -> ForeignKeyInfo {
        ForeignKeyInfo {
            name: name.to_string(),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            referenced_table: referenced_table.to_string(),
            referenced_columns: vec!["id".to_string()],
            on_delete: "NO ACTION".to_string(),
            on_update: "NO ACTION".to_string(),
        }
    }

    #[test]
    fn test_schema_diff_no_changes() {
        let table1 = TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                create_test_column("id", ColumnType::Integer, false),
                create_test_column("name", ColumnType::Text, false),
            ],
            indexes: vec![],
            foreign_keys: vec![],
        };

        let current = vec![table1.clone()];
        let desired = vec![table1];

        let diff = SchemaDiff::compare(&current, &desired);
        assert!(diff.is_empty(), "Expected no changes");
    }

    #[test]
    fn test_schema_diff_new_table() {
        let current: Vec<TableInfo> = vec![];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("id", ColumnType::Integer, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Created(table) => {
                assert_eq!(table.name, "users");
            }
            _ => panic!("Expected TableChange::Created"),
        }
    }

    #[test]
    fn test_schema_diff_dropped_table() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired: Vec<TableInfo> = vec![];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Dropped(name) => {
                assert_eq!(name, "users");
            }
            _ => panic!("Expected TableChange::Dropped"),
        }
    }

    #[test]
    fn test_schema_diff_added_column() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("id", ColumnType::Integer, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                create_test_column("id", ColumnType::Integer, false),
                create_test_column("email", ColumnType::Text, false),
            ],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { table_name, column_changes, .. } => {
                assert_eq!(table_name, "users");
                assert_eq!(column_changes.len(), 1);
                match &column_changes[0] {
                    ColumnChange::Added(col) => {
                        assert_eq!(col.name, "email");
                    }
                    _ => panic!("Expected ColumnChange::Added"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_removed_column() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                create_test_column("id", ColumnType::Integer, false),
                create_test_column("email", ColumnType::Text, false),
            ],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("id", ColumnType::Integer, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { table_name, column_changes, .. } => {
                assert_eq!(table_name, "users");
                assert_eq!(column_changes.len(), 1);
                match &column_changes[0] {
                    ColumnChange::Removed(col) => {
                        assert_eq!(col.name, "email");
                    }
                    _ => panic!("Expected ColumnChange::Removed"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_type_changed() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("age", ColumnType::SmallInt, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("age", ColumnType::Integer, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { column_changes, .. } => {
                assert_eq!(column_changes.len(), 1);
                match &column_changes[0] {
                    ColumnChange::TypeChanged { old, new } => {
                        assert_eq!(old.data_type, ColumnType::SmallInt);
                        assert_eq!(new.data_type, ColumnType::Integer);
                    }
                    _ => panic!("Expected ColumnChange::TypeChanged"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_nullability_changed() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("email", ColumnType::Text, true)],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![create_test_column("email", ColumnType::Text, false)],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { column_changes, .. } => {
                assert_eq!(column_changes.len(), 1);
                match &column_changes[0] {
                    ColumnChange::NullabilityChanged { old, new } => {
                        assert!(old.nullable);
                        assert!(!new.nullable);
                    }
                    _ => panic!("Expected ColumnChange::NullabilityChanged"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_added_index() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![create_test_index("idx_email", vec!["email"], true)],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { index_changes, .. } => {
                assert_eq!(index_changes.len(), 1);
                match &index_changes[0] {
                    IndexChange::Added(idx) => {
                        assert_eq!(idx.name, "idx_email");
                        assert!(idx.is_unique);
                    }
                    _ => panic!("Expected IndexChange::Added"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_removed_index() {
        let current = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![create_test_index("idx_email", vec!["email"], true)],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { index_changes, .. } => {
                assert_eq!(index_changes.len(), 1);
                match &index_changes[0] {
                    IndexChange::Removed(idx) => {
                        assert_eq!(idx.name, "idx_email");
                    }
                    _ => panic!("Expected IndexChange::Removed"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_added_foreign_key() {
        let current = vec![TableInfo {
            name: "posts".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
        }];
        let desired = vec![TableInfo {
            name: "posts".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![create_test_foreign_key("fk_author", vec!["author_id"], "users")],
        }];

        let diff = SchemaDiff::compare(&current, &desired);
        assert_eq!(diff.changes.len(), 1);
        match &diff.changes[0] {
            TableChange::Altered { foreign_key_changes, .. } => {
                assert_eq!(foreign_key_changes.len(), 1);
                match &foreign_key_changes[0] {
                    ForeignKeyChange::Added(fk) => {
                        assert_eq!(fk.name, "fk_author");
                        assert_eq!(fk.referenced_table, "users");
                    }
                    _ => panic!("Expected ForeignKeyChange::Added"),
                }
            }
            _ => panic!("Expected TableChange::Altered"),
        }
    }

    #[test]
    fn test_schema_diff_complex_changes() {
        let current = vec![
            TableInfo {
                name: "users".to_string(),
                schema: "public".to_string(),
                columns: vec![
                    create_test_column("id", ColumnType::Integer, false),
                    create_test_column("old_field", ColumnType::Text, true),
                ],
                indexes: vec![create_test_index("old_idx", vec!["old_field"], false)],
                foreign_keys: vec![],
            },
            TableInfo {
                name: "deprecated_table".to_string(),
                schema: "public".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
            },
        ];

        let desired = vec![
            TableInfo {
                name: "users".to_string(),
                schema: "public".to_string(),
                columns: vec![
                    create_test_column("id", ColumnType::Integer, false),
                    create_test_column("email", ColumnType::Text, false),
                ],
                indexes: vec![create_test_index("idx_email", vec!["email"], true)],
                foreign_keys: vec![],
            },
            TableInfo {
                name: "posts".to_string(),
                schema: "public".to_string(),
                columns: vec![create_test_column("id", ColumnType::Integer, false)],
                indexes: vec![],
                foreign_keys: vec![],
            },
        ];

        let diff = SchemaDiff::compare(&current, &desired);

        // Should have 3 changes: dropped table, altered users, created posts
        assert_eq!(diff.changes.len(), 3);

        let mut has_drop = false;
        let mut has_alter = false;
        let mut has_create = false;

        for change in &diff.changes {
            match change {
                TableChange::Dropped(name) => {
                    assert_eq!(name, "deprecated_table");
                    has_drop = true;
                }
                TableChange::Altered { table_name, column_changes, index_changes, .. } => {
                    assert_eq!(table_name, "users");
                    assert_eq!(column_changes.len(), 2); // removed old_field, added email
                    assert_eq!(index_changes.len(), 2); // removed old_idx, added idx_email
                    has_alter = true;
                }
                TableChange::Created(table) => {
                    assert_eq!(table.name, "posts");
                    has_create = true;
                }
            }
        }

        assert!(has_drop, "Should have drop change");
        assert!(has_alter, "Should have alter change");
        assert!(has_create, "Should have create change");
    }

    #[test]
    fn test_column_type_to_sql() {
        assert_eq!(ColumnType::SmallInt.to_sql(), "SMALLINT");
        assert_eq!(ColumnType::Integer.to_sql(), "INTEGER");
        assert_eq!(ColumnType::BigInt.to_sql(), "BIGINT");
        assert_eq!(ColumnType::Text.to_sql(), "TEXT");
        assert_eq!(ColumnType::Boolean.to_sql(), "BOOLEAN");
        assert_eq!(ColumnType::Uuid.to_sql(), "UUID");
        assert_eq!(ColumnType::Varchar(Some(255)).to_sql(), "VARCHAR(255)");
        assert_eq!(ColumnType::Varchar(None).to_sql(), "VARCHAR");
        assert_eq!(ColumnType::Numeric(Some(10), Some(2)).to_sql(), "NUMERIC(10, 2)");
        assert_eq!(ColumnType::Numeric(Some(10), None).to_sql(), "NUMERIC(10)");
        assert_eq!(ColumnType::Numeric(None, None).to_sql(), "NUMERIC");
        assert_eq!(ColumnType::Array(Box::new(ColumnType::Integer)).to_sql(), "INTEGER[]");
        assert_eq!(ColumnType::Custom("CITEXT".to_string()).to_sql(), "CITEXT");
    }

    #[test]
    fn test_generate_up_sql_create_table() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: ColumnType::Integer,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                },
                ColumnInfo {
                    name: "email".to_string(),
                    data_type: ColumnType::Text,
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_unique: true,
                },
                ColumnInfo {
                    name: "created_at".to_string(),
                    data_type: ColumnType::Timestamp,
                    nullable: false,
                    default: Some("NOW()".to_string()),
                    is_primary_key: false,
                    is_unique: false,
                },
            ],
            indexes: vec![],
            foreign_keys: vec![],
        };

        let diff = SchemaDiff {
            changes: vec![TableChange::Created(table)],
        };

        let sql = diff.generate_up_sql();
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("\"id\" INTEGER PRIMARY KEY"));
        assert!(sql.contains("\"email\" TEXT NOT NULL UNIQUE"));
        assert!(sql.contains("\"created_at\" TIMESTAMP NOT NULL DEFAULT NOW()"));
    }

    #[test]
    fn test_generate_up_sql_drop_table() {
        let diff = SchemaDiff {
            changes: vec![TableChange::Dropped("old_table".to_string())],
        };

        let sql = diff.generate_up_sql();
        assert_eq!(sql, "DROP TABLE IF EXISTS \"old_table\" CASCADE;");
    }

    #[test]
    fn test_generate_up_sql_add_column() {
        let diff = SchemaDiff {
            changes: vec![TableChange::Altered {
                table_name: "users".to_string(),
                column_changes: vec![ColumnChange::Added(ColumnInfo {
                    name: "age".to_string(),
                    data_type: ColumnType::Integer,
                    nullable: true,
                    default: None,
                    is_primary_key: false,
                    is_unique: false,
                })],
                index_changes: vec![],
                foreign_key_changes: vec![],
            }],
        };

        let sql = diff.generate_up_sql();
        assert_eq!(sql, "ALTER TABLE \"users\" ADD COLUMN \"age\" INTEGER;");
    }

    #[test]
    fn test_generate_up_sql_remove_column() {
        let diff = SchemaDiff {
            changes: vec![TableChange::Altered {
                table_name: "users".to_string(),
                column_changes: vec![ColumnChange::Removed(ColumnInfo {
                    name: "old_field".to_string(),
                    data_type: ColumnType::Text,
                    nullable: true,
                    default: None,
                    is_primary_key: false,
                    is_unique: false,
                })],
                index_changes: vec![],
                foreign_key_changes: vec![],
            }],
        };

        let sql = diff.generate_up_sql();
        assert_eq!(sql, "ALTER TABLE \"users\" DROP COLUMN \"old_field\";");
    }

    #[test]
    fn test_generate_up_sql_type_change() {
        let old_col = ColumnInfo {
            name: "age".to_string(),
            data_type: ColumnType::SmallInt,
            nullable: false,
            default: None,
            is_primary_key: false,
            is_unique: false,
        };
        let new_col = ColumnInfo {
            name: "age".to_string(),
            data_type: ColumnType::Integer,
            nullable: false,
            default: None,
            is_primary_key: false,
            is_unique: false,
        };

        let diff = SchemaDiff {
            changes: vec![TableChange::Altered {
                table_name: "users".to_string(),
                column_changes: vec![ColumnChange::TypeChanged { old: old_col, new: new_col }],
                index_changes: vec![],
                foreign_key_changes: vec![],
            }],
        };

        let sql = diff.generate_up_sql();
        assert_eq!(sql, "ALTER TABLE \"users\" ALTER COLUMN \"age\" TYPE INTEGER USING \"age\"::INTEGER;");
    }

    #[test]
    fn test_generate_down_sql_create_table() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
        };

        let diff = SchemaDiff {
            changes: vec![TableChange::Created(table)],
        };

        let sql = diff.generate_down_sql();
        assert_eq!(sql, "DROP TABLE IF EXISTS \"users\" CASCADE;");
    }

    #[test]
    fn test_generate_down_sql_drop_table() {
        let diff = SchemaDiff {
            changes: vec![TableChange::Dropped("old_table".to_string())],
        };

        let sql = diff.generate_down_sql();
        assert_eq!(sql, "-- Cannot auto-generate: recreate table \"old_table\"");
    }

    #[test]
    fn test_generate_sql_with_indexes() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: ColumnType::Integer,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                },
                ColumnInfo {
                    name: "email".to_string(),
                    data_type: ColumnType::Text,
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_unique: false,
                },
            ],
            indexes: vec![
                IndexInfo {
                    name: "idx_email".to_string(),
                    columns: vec!["email".to_string()],
                    is_unique: true,
                    index_type: "btree".to_string(),
                },
            ],
            foreign_keys: vec![],
        };

        let diff = SchemaDiff {
            changes: vec![TableChange::Created(table)],
        };

        let sql = diff.generate_up_sql();
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("CREATE UNIQUE INDEX \"idx_email\" ON \"users\" (\"email\");"));
    }

    #[test]
    fn test_generate_sql_with_foreign_keys() {
        let table = TableInfo {
            name: "posts".to_string(),
            schema: "public".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: ColumnType::Integer,
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    is_unique: false,
                },
                ColumnInfo {
                    name: "author_id".to_string(),
                    data_type: ColumnType::Integer,
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_unique: false,
                },
            ],
            indexes: vec![],
            foreign_keys: vec![
                ForeignKeyInfo {
                    name: "fk_author".to_string(),
                    columns: vec!["author_id".to_string()],
                    referenced_table: "users".to_string(),
                    referenced_columns: vec!["id".to_string()],
                    on_delete: "CASCADE".to_string(),
                    on_update: "CASCADE".to_string(),
                },
            ],
        };

        let diff = SchemaDiff {
            changes: vec![TableChange::Created(table)],
        };

        let sql = diff.generate_up_sql();
        assert!(sql.contains("CREATE TABLE \"posts\""));
        assert!(sql.contains("ALTER TABLE \"posts\" ADD CONSTRAINT \"fk_author\" FOREIGN KEY (\"author_id\") REFERENCES \"users\" (\"id\") ON DELETE CASCADE ON UPDATE CASCADE;"));
    }

    #[test]
    fn test_cascade_rule_conversions() {
        // Test from_sql
        assert_eq!(CascadeRule::from_sql("CASCADE"), CascadeRule::Cascade);
        assert_eq!(CascadeRule::from_sql("RESTRICT"), CascadeRule::Restrict);
        assert_eq!(CascadeRule::from_sql("SET NULL"), CascadeRule::SetNull);
        assert_eq!(CascadeRule::from_sql("SET DEFAULT"), CascadeRule::SetDefault);
        assert_eq!(CascadeRule::from_sql("NO ACTION"), CascadeRule::NoAction);
        assert_eq!(CascadeRule::from_sql("unknown"), CascadeRule::NoAction);

        // Test to_sql
        assert_eq!(CascadeRule::Cascade.to_sql(), "CASCADE");
        assert_eq!(CascadeRule::Restrict.to_sql(), "RESTRICT");
        assert_eq!(CascadeRule::SetNull.to_sql(), "SET NULL");
        assert_eq!(CascadeRule::SetDefault.to_sql(), "SET DEFAULT");
        assert_eq!(CascadeRule::NoAction.to_sql(), "NO ACTION");
    }

    #[test]
    fn test_backref_struct() {
        let backref = BackRef {
            source_table: "posts".to_string(),
            source_column: "author_id".to_string(),
            target_table: "users".to_string(),
            target_column: "id".to_string(),
            constraint_name: "fk_author".to_string(),
            on_delete: CascadeRule::Cascade,
            on_update: CascadeRule::NoAction,
        };

        assert_eq!(backref.source_table, "posts");
        assert_eq!(backref.on_delete, CascadeRule::Cascade);
    }

    #[test]
    fn test_many_to_many_config_builder() {
        // Test basic creation with defaults
        let config = ManyToManyConfig::new(
            "user_tags",
            "user_id",
            "tag_id",
            "tags",
        );

        assert_eq!(config.join_table, "user_tags");
        assert_eq!(config.source_key, "user_id");
        assert_eq!(config.target_key, "tag_id");
        assert_eq!(config.target_table, "tags");
        assert_eq!(config.source_reference, "id");
        assert_eq!(config.target_reference, "id");
        assert_eq!(config.on_delete, CascadeRule::Cascade);
    }

    #[test]
    fn test_many_to_many_config_builder_with_options() {
        // Test builder pattern with custom values
        let config = ManyToManyConfig::new(
            "user_roles",
            "user_uuid",
            "role_uuid",
            "roles",
        )
        .with_source_reference("uuid")
        .with_target_reference("uuid")
        .with_on_delete(CascadeRule::SetNull);

        assert_eq!(config.join_table, "user_roles");
        assert_eq!(config.source_key, "user_uuid");
        assert_eq!(config.target_key, "role_uuid");
        assert_eq!(config.target_table, "roles");
        assert_eq!(config.source_reference, "uuid");
        assert_eq!(config.target_reference, "uuid");
        assert_eq!(config.on_delete, CascadeRule::SetNull);
    }
}
