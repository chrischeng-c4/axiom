//! PostgreSQL row representation.
//!
//! This module provides a row abstraction for query results,
//! similar to cclab-nebula's document handling.

use serde_json::Value as JsonValue;
use sqlx::postgres::{PgArguments, PgPool};
use sqlx::Row as SqlxRow;
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use crate::query::{JoinCondition, JoinType, Operator, OrderDirection};
use crate::{row_to_extracted, DataBridgeError, ExtractedValue, QueryBuilder, Result};

/// Relation configuration for eager loading
#[derive(Debug, Clone)]
pub struct RelationConfig {
    /// Name of the relation (used as key in result)
    pub name: String,
    /// Table to join
    pub table: String,
    /// Column in main table that references the foreign table
    pub foreign_key: String,
    /// Column in foreign table being referenced (usually "id")
    pub reference_column: String,
    /// Type of join (usually Left for optional relations)
    pub join_type: JoinType,
    /// Columns to select from the related table (None = all)
    pub select_columns: Option<Vec<String>>,
}

/// Represents a single row from a PostgreSQL query result.
///
/// This is the primary data structure returned from queries.
/// It wraps column names and values in a type-safe manner.
#[derive(Debug, Clone)]
pub struct Row {
    /// Column name to value mapping
    pub(crate) columns: HashMap<String, ExtractedValue>,
}

impl Row {
    /// Creates a new row from a column map.
    pub fn new(columns: HashMap<String, ExtractedValue>) -> Self {
        Self { columns }
    }

    /// Gets a value by column name.
    pub fn get(&self, column: &str) -> Result<&ExtractedValue> {
        self.columns
            .get(column)
            .ok_or_else(|| DataBridgeError::Query("Column not found in result set".to_string()))
    }

    /// Gets all column names.
    pub fn columns(&self) -> Vec<&str> {
        self.columns.keys().map(|s| s.as_str()).collect()
    }

    /// Gets a reference to the column map.
    pub fn columns_map(&self) -> &HashMap<String, ExtractedValue> {
        &self.columns
    }

    /// Converts row to a JSON object.
    pub fn to_json(&self) -> Result<JsonValue> {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.columns {
            let json_value = extracted_value_to_json(value)?;
            map.insert(key.clone(), json_value);
        }
        Ok(JsonValue::Object(map))
    }

    /// Converts from SQLx row.
    pub fn from_sqlx(row: &sqlx::postgres::PgRow) -> Result<Self> {
        let columns = row_to_extracted(row)?;
        Ok(Self { columns })
    }

    /// Insert row into database, return generated ID.
    #[instrument(skip(executor, values), fields(table = %table, value_count = values.len()))]
    pub async fn insert<'a, E>(
        executor: E,
        table: &str,
        values: &[(String, ExtractedValue)],
    ) -> Result<Self>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if values.is_empty() {
            return Err(DataBridgeError::Query(
                "Cannot insert with no values".to_string(),
            ));
        }

        info!("Inserting row");
        let query_builder = QueryBuilder::new(table)?;
        let (sql, params) = query_builder.build_insert(values)?;

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let row = sqlx::query_with(&sql, args)
            .fetch_one(executor)
            .await
            .map_err(|_| DataBridgeError::Query("Insert operation failed".to_string()))?;

        let result = Self::from_sqlx(&row)?;
        info!("Insert complete");
        Ok(result)
    }

    /// Insert multiple rows with a single batch INSERT statement.
    #[instrument(skip(executor, rows), fields(table = %table, row_count = rows.len()))]
    pub async fn insert_many<'a, E>(
        executor: E,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
    ) -> Result<Vec<Self>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if rows.is_empty() {
            return Ok(vec![]);
        }

        info!("Inserting rows");
        let first_row = &rows[0];
        let mut column_names: Vec<&String> = first_row.keys().collect();
        column_names.sort();

        QueryBuilder::validate_identifier(table)?;
        for col in &column_names {
            QueryBuilder::validate_identifier(col)?;
        }

        let mut col_list = Vec::new();
        for s in &column_names {
            col_list.push(QueryBuilder::quote_identifier(s));
        }

        let mut sql = format!(
            "INSERT INTO {} ({}) VALUES ",
            QueryBuilder::quote_identifier(table),
            col_list.join(", ")
        );

        let mut param_num = 1;
        let mut values_clauses = Vec::with_capacity(rows.len());

        for _ in 0..rows.len() {
            let mut placeholders = Vec::new();
            for _ in 0..column_names.len() {
                placeholders.push(format!("${}", param_num));
                param_num += 1;
            }
            values_clauses.push(format!("({})", placeholders.join(", ")));
        }

        sql.push_str(&values_clauses.join(", "));
        sql.push_str(" RETURNING *");

        let mut args = PgArguments::default();
        for row in rows {
            for col_name in &column_names {
                let value = row.get(*col_name).ok_or_else(|| {
                    DataBridgeError::Query("Required column not found in row data".to_string())
                })?;
                value.bind_to_arguments(&mut args)?;
            }
        }

        let pg_rows = sqlx::query_with(&sql, args)
            .fetch_all(executor)
            .await
            .map_err(|_| DataBridgeError::Query("Batch insert operation failed".to_string()))?;

        let results = pg_rows
            .iter()
            .map(Self::from_sqlx)
            .collect::<Result<Vec<_>>>()?;

        info!(affected = results.len(), "Insert complete");
        Ok(results)
    }

    /// Upsert a single row (INSERT ON CONFLICT UPDATE).
    pub async fn upsert<'a, E>(
        executor: E,
        table: &str,
        values: &[(String, ExtractedValue)],
        conflict_target: &[String],
        update_columns: Option<&[String]>,
    ) -> Result<Self>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if values.is_empty() {
            return Err(DataBridgeError::Query(
                "Cannot upsert with no values".to_string(),
            ));
        }

        let query_builder = QueryBuilder::new(table)?;
        let (sql, params) = query_builder.build_upsert(values, conflict_target, update_columns)?;

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let row = sqlx::query_with(&sql, args)
            .fetch_one(executor)
            .await
            .map_err(|_| DataBridgeError::Query("Upsert operation failed".to_string()))?;

        Self::from_sqlx(&row)
    }

    /// Upsert multiple rows with a single batch statement.
    pub async fn upsert_many<'a, E>(
        executor: E,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
        conflict_target: &[String],
        update_columns: Option<&[String]>,
    ) -> Result<Vec<Self>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if rows.is_empty() {
            return Ok(vec![]);
        }

        if conflict_target.is_empty() {
            return Err(DataBridgeError::Query(
                "Conflict target cannot be empty".to_string(),
            ));
        }

        let first_row = &rows[0];
        let mut column_names: Vec<&String> = first_row.keys().collect();
        column_names.sort();

        QueryBuilder::validate_identifier(table)?;
        for col in &column_names {
            QueryBuilder::validate_identifier(col)?;
        }

        let mut col_list = Vec::new();
        for s in &column_names {
            col_list.push(QueryBuilder::quote_identifier(s));
        }

        let mut sql = format!("INSERT INTO {} (", QueryBuilder::quote_identifier(table));
        sql.push_str(&col_list.join(", "));
        sql.push_str(") VALUES ");

        let num_cols = column_names.len();
        let value_clauses: Vec<String> = (0..rows.len())
            .map(|row_idx| {
                let start = row_idx * num_cols + 1;
                let mut placeholders = Vec::new();
                for i in start..(start + num_cols) {
                    placeholders.push(format!("${}", i));
                }
                format!("({})", placeholders.join(", "))
            })
            .collect();
        sql.push_str(&value_clauses.join(", "));

        sql.push_str(" ON CONFLICT (");
        let mut target_list = Vec::new();
        for s in conflict_target {
            target_list.push(QueryBuilder::quote_identifier(s));
        }
        sql.push_str(&target_list.join(", "));
        sql.push_str(") DO UPDATE SET ");

        let columns_to_update: Vec<&&String> = if let Some(update_cols) = update_columns {
            column_names
                .iter()
                .filter(|col| update_cols.contains(&col.to_string()))
                .collect()
        } else {
            column_names
                .iter()
                .filter(|col| !conflict_target.contains(&col.to_string()))
                .collect()
        };

        if columns_to_update.is_empty() {
            return Err(DataBridgeError::Query("No columns to update".to_string()));
        }

        let mut set_list = Vec::new();
        for col in columns_to_update {
            let quoted = QueryBuilder::quote_identifier(col);
            set_list.push(format!("{} = EXCLUDED.{}", quoted, quoted));
        }
        sql.push_str(&set_list.join(", "));
        sql.push_str(" RETURNING *");

        let mut args = PgArguments::default();
        for row in rows {
            for col_name in &column_names {
                let value = row.get(*col_name).ok_or_else(|| {
                    DataBridgeError::Query("Required column not found in row data".to_string())
                })?;
                value.bind_to_arguments(&mut args)?;
            }
        }

        let pg_rows = sqlx::query_with(&sql, args)
            .fetch_all(executor)
            .await
            .map_err(|_| DataBridgeError::Query("Batch upsert operation failed".to_string()))?;

        pg_rows.iter().map(Self::from_sqlx).collect()
    }

    /// Find single row by primary key.
    pub async fn find_by_id(pool: &PgPool, table: &str, id: i64) -> Result<Option<Self>> {
        let mut qb = QueryBuilder::new(table)?;
        qb = qb.where_clause("id", Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = qb.build_select();

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let result = sqlx::query_with(&sql, args)
            .fetch_optional(pool)
            .await
            .map_err(|_| DataBridgeError::Query("Find operation failed".to_string()))?;

        match result {
            Some(row) => Ok(Some(Self::from_sqlx(&row)?)),
            None => Ok(None),
        }
    }

    /// Find multiple rows with filters.
    pub async fn find_many(
        pool: &PgPool,
        table: &str,
        query: Option<&QueryBuilder>,
    ) -> Result<Vec<Self>> {
        let (sql, params) = if let Some(qb) = query {
            qb.build_select()
        } else {
            let qb = QueryBuilder::new(table)?;
            qb.build_select()
        };

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let rows = sqlx::query_with(&sql, args)
            .fetch_all(pool)
            .await
            .map_err(|_| DataBridgeError::Query("Find operation failed".to_string()))?;

        rows.iter().map(Self::from_sqlx).collect()
    }

    /// Update row in database.
    #[instrument(skip(pool, values), fields(table = %table, id = %id, value_count = values.len()))]
    pub async fn update(
        pool: &PgPool,
        table: &str,
        id: i64,
        values: &[(String, ExtractedValue)],
    ) -> Result<bool> {
        if values.is_empty() {
            return Err(DataBridgeError::Query(
                "Cannot update with no values".to_string(),
            ));
        }

        info!("Updating row");
        let mut qb = QueryBuilder::new(table)?;
        qb = qb.where_clause("id", Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = qb.build_update(values)?;

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let result = sqlx::query_with(&sql, args)
            .execute(pool)
            .await
            .map_err(|_| DataBridgeError::Query("Update operation failed".to_string()))?;

        let affected = result.rows_affected();
        info!(affected, "Update complete");
        Ok(affected > 0)
    }

    /// Delete row from database.
    #[instrument(skip(pool), fields(table = %table, id = %id))]
    pub async fn delete(pool: &PgPool, table: &str, id: i64) -> Result<bool> {
        info!("Deleting row");
        let mut qb = QueryBuilder::new(table)?;
        qb = qb.where_clause("id", Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = qb.build_delete();

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let result = sqlx::query_with(&sql, args)
            .execute(pool)
            .await
            .map_err(|_| DataBridgeError::Query("Delete operation failed".to_string()))?;

        let affected = result.rows_affected();
        info!(affected, "Delete complete");
        Ok(affected > 0)
    }

    /// Count rows matching query.
    pub async fn count(pool: &PgPool, table: &str, query: Option<&QueryBuilder>) -> Result<i64> {
        let mut sql = format!(
            "SELECT COUNT(*) FROM {}",
            QueryBuilder::quote_identifier(table)
        );
        let mut params = Vec::new();

        if let Some(qb) = query {
            let (select_sql, select_params) = qb.build_select();
            params = select_params;

            if let Some(where_pos) = select_sql.find(" WHERE ") {
                let where_clause = &select_sql[where_pos..];
                let end_pos = where_clause
                    .find(" ORDER BY ")
                    .or_else(|| where_clause.find(" LIMIT "))
                    .or_else(|| where_clause.find(" OFFSET "))
                    .unwrap_or(where_clause.len());
                sql.push_str(&where_clause[..end_pos]);
            }
        }

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let row = sqlx::query_with(&sql, args)
            .fetch_one(pool)
            .await
            .map_err(|_| DataBridgeError::Query("Count operation failed".to_string()))?;

        let count: i64 = row
            .try_get(0)
            .map_err(|_| DataBridgeError::Query("Failed to extract count result".to_string()))?;

        Ok(count)
    }

    /// Fetch a single row with related data using JOINs.
    pub async fn find_with_relations<'a, E>(
        executor: E,
        table: &str,
        id: i64,
        relations: &[RelationConfig],
    ) -> Result<Option<Self>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        // Validate RelationConfig fields before SQL generation
        for rel in relations {
            QueryBuilder::validate_identifier(&rel.name)?;
            QueryBuilder::validate_identifier(&rel.table)?;
            QueryBuilder::validate_identifier(&rel.foreign_key)?;
            QueryBuilder::validate_identifier(&rel.reference_column)?;
            if let Some(cols) = &rel.select_columns {
                for col in cols {
                    QueryBuilder::validate_identifier(col)?;
                }
            }
        }

        QueryBuilder::validate_identifier(table)?;

        // Get main table columns to properly alias them
        let query = "SELECT column_name FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position";

        let column_rows = sqlx::query(query)
            .bind("public")
            .bind(table)
            .fetch_all(executor)
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to fetch table columns: {}", e))
            })?;

        let mut main_columns = Vec::new();
        for row in column_rows {
            let col_name: String = row
                .try_get("column_name")
                .map_err(|e| DataBridgeError::Database(e.to_string()))?;
            main_columns.push(col_name);
        }

        if main_columns.is_empty() {
            return Err(DataBridgeError::Query(format!(
                "Table '{}' not found or has no columns",
                table
            )));
        }

        let quoted_main_table = QueryBuilder::quote_identifier(table);

        // Alias main table columns with _main_ prefix to avoid collisions
        let mut select_cols: Vec<String> = main_columns
            .iter()
            .map(|col| {
                let quoted_col = QueryBuilder::quote_identifier(col);
                format!("{}.{} AS \"_main_{}\"", quoted_main_table, quoted_col, col)
            })
            .collect();

        for (idx, rel) in relations.iter().enumerate() {
            let alias = format!("_rel{}", idx);
            match &rel.select_columns {
                Some(cols) => {
                    for col in cols {
                        let quoted_col = QueryBuilder::quote_identifier(col);
                        // quoted_col already contains quotes (e.g., "column_name"), so use it directly
                        select_cols.push(format!(
                            "\"{}\".{} AS \"{}__{}\"",
                            alias, quoted_col, rel.name, col
                        ));
                    }
                }
                None => {
                    select_cols.push(format!(
                        "\"{}\".\"{}\" AS \"{}__exists\"",
                        alias, rel.reference_column, rel.name
                    ));
                    select_cols.push(format!(
                        "row_to_json(\"{}\") AS \"{}__data\"",
                        alias, rel.name
                    ));
                }
            }
        }

        let mut qb = QueryBuilder::new(table)?.select_raw(select_cols);

        for (idx, rel) in relations.iter().enumerate() {
            let alias = format!("_rel{}", idx);
            let join_condition =
                JoinCondition::new(&rel.foreign_key, &alias, &rel.reference_column)?;

            qb = qb.join(
                rel.join_type.clone(),
                &rel.table,
                Some(&alias),
                join_condition,
            )?;
        }

        let qualified_id_col = format!("{}.id", table);
        qb = qb.where_clause(&qualified_id_col, Operator::Eq, ExtractedValue::BigInt(id))?;

        let (sql, params) = qb.build_select();

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let row = sqlx::query_with(&sql, args)
            .fetch_optional(executor)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        match row {
            Some(pg_row) => {
                let mut result = Self::from_sqlx(&pg_row)?;

                // First, strip _main_ prefix from main table columns
                let main_keys: Vec<String> = result
                    .columns
                    .keys()
                    .filter(|k| k.starts_with("_main_"))
                    .cloned()
                    .collect();

                for key in main_keys {
                    if let Some(value) = result.columns.remove(&key) {
                        let final_key = key
                            .strip_prefix("_main_")
                            .ok_or_else(|| {
                                DataBridgeError::Query(
                                    "Failed to process main table column prefix".to_string(),
                                )
                            })?
                            .to_string();
                        result.columns.insert(final_key, value);
                    }
                }

                // Then process relations
                for rel in relations {
                    let real_prefix = format!("{}__", rel.name);

                    let exists_key = format!("{}exists", real_prefix);
                    let row_exists = !matches!(
                        result.columns.remove(&exists_key),
                        Some(ExtractedValue::Null) | None
                    );

                    if !row_exists {
                        result
                            .columns
                            .insert(rel.name.clone(), ExtractedValue::Null);
                        let mut to_remove = Vec::new();
                        for k in result.columns.keys() {
                            if k.starts_with(&real_prefix) {
                                to_remove.push(k.clone());
                            }
                        }
                        for k in to_remove {
                            result.columns.remove(&k);
                        }
                        continue;
                    }

                    let mut rel_data = serde_json::Map::new();
                    let keys_to_process: Vec<String> = result
                        .columns
                        .keys()
                        .filter(|k| k.starts_with(&real_prefix))
                        .cloned()
                        .collect();

                    for key in keys_to_process {
                        if let Some(value) = result.columns.remove(&key) {
                            let rel_key = key
                                .strip_prefix(&real_prefix)
                                .ok_or_else(|| {
                                    DataBridgeError::Query(
                                        "Failed to process nested data structure".to_string(),
                                    )
                                })?
                                .to_string();

                            if rel_key == "data" {
                                if let ExtractedValue::Json(JsonValue::Object(data_map)) = value {
                                    for (k, v) in data_map {
                                        rel_data.insert(k, v);
                                    }
                                }
                            } else {
                                let json_value = extracted_value_to_json(&value)?;
                                rel_data.insert(rel_key, json_value);
                            }
                        }
                    }

                    if !rel_data.is_empty() {
                        result.columns.insert(
                            rel.name.clone(),
                            ExtractedValue::Json(JsonValue::Object(rel_data)),
                        );
                    } else {
                        result
                            .columns
                            .insert(rel.name.clone(), ExtractedValue::Null);
                    }
                }

                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Fetch multiple rows with related data using JOINs.
    pub async fn find_many_with_relations<'a, E>(
        executor: E,
        table: &str,
        relations: &[RelationConfig],
        where_clause: Option<(&str, Operator, ExtractedValue)>,
        order_by: Option<(&str, OrderDirection)>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        // Validate RelationConfig fields before SQL generation
        for rel in relations {
            QueryBuilder::validate_identifier(&rel.name)?;
            QueryBuilder::validate_identifier(&rel.table)?;
            QueryBuilder::validate_identifier(&rel.foreign_key)?;
            QueryBuilder::validate_identifier(&rel.reference_column)?;
            if let Some(cols) = &rel.select_columns {
                for col in cols {
                    QueryBuilder::validate_identifier(col)?;
                }
            }
        }

        QueryBuilder::validate_identifier(table)?;

        // Get main table columns to properly alias them
        let query = "SELECT column_name FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position";

        let column_rows = sqlx::query(query)
            .bind("public")
            .bind(table)
            .fetch_all(executor)
            .await
            .map_err(|e| {
                DataBridgeError::Database(format!("Failed to fetch table columns: {}", e))
            })?;

        let mut main_columns = Vec::new();
        for row in column_rows {
            let col_name: String = row
                .try_get("column_name")
                .map_err(|e| DataBridgeError::Database(e.to_string()))?;
            main_columns.push(col_name);
        }

        if main_columns.is_empty() {
            return Err(DataBridgeError::Query(format!(
                "Table '{}' not found or has no columns",
                table
            )));
        }

        let quoted_main_table = QueryBuilder::quote_identifier(table);

        // Alias main table columns with _main_ prefix to avoid collisions
        let mut select_cols: Vec<String> = main_columns
            .iter()
            .map(|col| {
                let quoted_col = QueryBuilder::quote_identifier(col);
                format!("{}.{} AS \"_main_{}\"", quoted_main_table, quoted_col, col)
            })
            .collect();
        for (idx, rel) in relations.iter().enumerate() {
            let alias = format!("_rel{}", idx);
            match &rel.select_columns {
                Some(cols) => {
                    for col in cols {
                        let quoted_col = QueryBuilder::quote_identifier(col);
                        // quoted_col already contains quotes (e.g., "column_name"), so use it directly
                        select_cols.push(format!(
                            "\"{}\".{} AS \"{}__{}\"",
                            alias, quoted_col, rel.name, col
                        ));
                    }
                }
                None => {
                    select_cols.push(format!(
                        "\"{}\".\"{}\" AS \"{}__exists\"",
                        alias, rel.reference_column, rel.name
                    ));
                    select_cols.push(format!(
                        "row_to_json(\"{}\") AS \"{}__data\"",
                        alias, rel.name
                    ));
                }
            }
        }

        let mut qb = QueryBuilder::new(table)?.select_raw(select_cols);

        for (idx, rel) in relations.iter().enumerate() {
            let alias = format!("_rel{}", idx);
            let join_condition =
                JoinCondition::new(&rel.foreign_key, &alias, &rel.reference_column)?;

            qb = qb.join(
                rel.join_type.clone(),
                &rel.table,
                Some(&alias),
                join_condition,
            )?;
        }

        if let Some((col, op, val)) = where_clause {
            let qualified_col = if col.contains('.') {
                col.to_string()
            } else {
                format!("{}.{}", table, col)
            };
            qb = qb.where_clause(&qualified_col, op, val)?;
        }

        if let Some((col, dir)) = order_by {
            let qualified_col = if col.contains('.') {
                col.to_string()
            } else {
                format!("{}.{}", table, col)
            };
            qb = qb.order_by(&qualified_col, dir)?;
        }

        if let Some(l) = limit {
            qb = qb.limit(l);
        }
        if let Some(o) = offset {
            qb = qb.offset(o);
        }

        let (sql, params) = qb.build_select();

        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        let rows = sqlx::query_with(&sql, args)
            .fetch_all(executor)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        let mut results = Vec::with_capacity(rows.len());
        for pg_row in rows {
            let mut result = Self::from_sqlx(&pg_row)?;

            // First, strip _main_ prefix from main table columns
            let main_keys: Vec<String> = result
                .columns
                .keys()
                .filter(|k| k.starts_with("_main_"))
                .cloned()
                .collect();

            for key in main_keys {
                if let Some(value) = result.columns.remove(&key) {
                    let final_key = key
                        .strip_prefix("_main_")
                        .ok_or_else(|| {
                            DataBridgeError::Query(
                                "Failed to process main table column prefix".to_string(),
                            )
                        })?
                        .to_string();
                    result.columns.insert(final_key, value);
                }
            }

            // Then process relations
            for rel in relations {
                let real_prefix = format!("{}__", rel.name);

                let exists_key = format!("{}exists", real_prefix);
                let row_exists = !matches!(
                    result.columns.remove(&exists_key),
                    Some(ExtractedValue::Null) | None
                );

                if !row_exists {
                    result
                        .columns
                        .insert(rel.name.clone(), ExtractedValue::Null);
                    let mut to_remove = Vec::new();
                    for k in result.columns.keys() {
                        if k.starts_with(&real_prefix) {
                            to_remove.push(k.clone());
                        }
                    }
                    for k in to_remove {
                        result.columns.remove(&k);
                    }
                    continue;
                }

                let mut rel_data = serde_json::Map::new();
                let keys_to_process: Vec<String> = result
                    .columns
                    .keys()
                    .filter(|k| k.starts_with(&real_prefix))
                    .cloned()
                    .collect();

                for key in keys_to_process {
                    if let Some(value) = result.columns.remove(&key) {
                        let rel_key = key
                            .strip_prefix(&real_prefix)
                            .ok_or_else(|| {
                                DataBridgeError::Query(
                                    "Failed to process nested data structure".to_string(),
                                )
                            })?
                            .to_string();
                        if rel_key == "data" {
                            if let ExtractedValue::Json(JsonValue::Object(data_map)) = value {
                                for (k, v) in data_map {
                                    rel_data.insert(k, v);
                                }
                            }
                        } else {
                            let json_value = extracted_value_to_json(&value)?;
                            rel_data.insert(rel_key, json_value);
                        }
                    }
                }

                if !rel_data.is_empty() {
                    result.columns.insert(
                        rel.name.clone(),
                        ExtractedValue::Json(JsonValue::Object(rel_data)),
                    );
                } else {
                    result
                        .columns
                        .insert(rel.name.clone(), ExtractedValue::Null);
                }
            }
            results.push(result);
        }

        Ok(results)
    }

    /// Simple eager loading helper - fetches with LEFT JOINs.
    pub async fn find_one_eager<'a, E>(
        executor: E,
        table: &str,
        id: i64,
        joins: &[(&str, &str, &str)],
    ) -> Result<Option<Self>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let relations: Vec<RelationConfig> = joins
            .iter()
            .map(|(name, fk, ref_table)| RelationConfig {
                name: name.to_string(),
                table: ref_table.to_string(),
                foreign_key: fk.to_string(),
                reference_column: "id".to_string(),
                join_type: JoinType::Left,
                select_columns: None,
            })
            .collect();

        Self::find_with_relations(executor, table, id, &relations).await
    }

    /// Delete a row with cascade handling based on foreign key rules.
    #[instrument(skip(pool), fields(table = %table, id = %id, id_column = %id_column))]
    pub async fn delete_with_cascade(
        pool: &PgPool,
        table: &str,
        id: i64,
        id_column: &str,
    ) -> Result<u64> {
        use crate::schema::CascadeRule;

        info!("Starting cascade delete");
        QueryBuilder::validate_identifier(table)?;
        QueryBuilder::validate_identifier(id_column)?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;
        let backrefs = Self::get_backreferences_internal(&mut *tx, table).await?;

        // Validate all backref identifiers before use in SQL
        for backref in &backrefs {
            QueryBuilder::validate_identifier(&backref.source_table)?;
            QueryBuilder::validate_identifier(&backref.source_column)?;
        }

        let mut total_deleted = 0u64;
        let mut cascaded_tables = Vec::new();

        for backref in &backrefs {
            match backref.on_delete {
                CascadeRule::Restrict | CascadeRule::NoAction => {
                    let check_query = format!(
                        "SELECT EXISTS(SELECT 1 FROM \"{}\" WHERE \"{}\" = $1) as has_children",
                        backref.source_table, backref.source_column
                    );
                    let row: (bool,) = sqlx::query_as(&check_query)
                        .bind(id)
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?;

                    if row.0 {
                        warn!(source_table = %backref.source_table, "Cascade delete blocked by RESTRICT constraint");
                        tx.rollback()
                            .await
                            .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                        return Err(DataBridgeError::Validation(
                            "Cannot delete record: foreign key constraint violation. Use cascade delete or remove referencing records first.".to_string()
                        ));
                    }
                }
                CascadeRule::Cascade => {
                    let delete_children = format!(
                        "DELETE FROM \"{}\" WHERE \"{}\" = $1",
                        backref.source_table, backref.source_column
                    );
                    let result = sqlx::query(&delete_children)
                        .bind(id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                    let deleted = result.rows_affected();
                    if deleted > 0 {
                        debug!(target_table = %backref.source_table, deleted, "Cascaded delete to related table");
                        cascaded_tables.push(backref.source_table.clone());
                    }
                    total_deleted += deleted;
                }
                CascadeRule::SetNull => {
                    let update_query = format!(
                        "UPDATE \"{}\" SET \"{}\" = NULL WHERE \"{}\" = $1",
                        backref.source_table, backref.source_column, backref.source_column
                    );
                    sqlx::query(&update_query)
                        .bind(id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                    debug!(target_table = %backref.source_table, "Set foreign key to NULL");
                }
                CascadeRule::SetDefault => {
                    let update_query = format!(
                        "UPDATE \"{}\" SET \"{}\" = DEFAULT WHERE \"{}\" = $1",
                        backref.source_table, backref.source_column, backref.source_column
                    );
                    sqlx::query(&update_query)
                        .bind(id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?;
                    debug!(target_table = %backref.source_table, "Set foreign key to DEFAULT");
                }
            }
        }

        let delete_query = format!(
            "DELETE FROM {} WHERE \"{}\" = $1",
            QueryBuilder::quote_identifier(table),
            id_column
        );
        let result = sqlx::query(&delete_query)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;
        total_deleted += result.rows_affected();

        tx.commit()
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        info!(
            total_deleted,
            cascaded_to = ?cascaded_tables,
            "Cascade delete complete"
        );
        Ok(total_deleted)
    }

    /// Delete a row, checking for RESTRICT constraints.
    pub async fn delete_checked(
        pool: &PgPool,
        table: &str,
        id: i64,
        id_column: &str,
    ) -> Result<u64> {
        use crate::schema::CascadeRule;

        QueryBuilder::validate_identifier(table)?;
        QueryBuilder::validate_identifier(id_column)?;

        let backrefs = Self::get_backreferences_internal(pool, table).await?;

        // Validate all backref identifiers before use in SQL
        for backref in &backrefs {
            QueryBuilder::validate_identifier(&backref.source_table)?;
            QueryBuilder::validate_identifier(&backref.source_column)?;
        }

        for backref in &backrefs {
            if matches!(
                backref.on_delete,
                CascadeRule::Restrict | CascadeRule::NoAction
            ) {
                let check_query = format!(
                    "SELECT EXISTS(SELECT 1 FROM \"{}\" WHERE \"{}\" = $1) as has_children",
                    backref.source_table, backref.source_column
                );
                let row: (bool,) = sqlx::query_as(&check_query)
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?;

                if row.0 {
                    return Err(DataBridgeError::Validation(
                        "Cannot delete record: foreign key constraint violation. Use cascade delete or remove referencing records first.".to_string()
                    ));
                }
            }
        }

        let query = format!(
            "DELETE FROM {} WHERE \"{}\" = $1",
            QueryBuilder::quote_identifier(table),
            id_column
        );

        let result = sqlx::query(&query)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Internal helper to get back-references without requiring a SchemaInspector instance.
    async fn get_backreferences_internal<'a, E>(
        executor: E,
        table: &str,
    ) -> Result<Vec<crate::schema::BackRef>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use crate::schema::{BackRef, CascadeRule};

        let query = "SELECT tc.table_name as source_table, kcu.column_name as source_column, ccu.table_name as target_table, ccu.column_name as target_column, tc.constraint_name, rc.delete_rule, rc.update_rule FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema JOIN information_schema.constraint_column_usage ccu ON ccu.constraint_name = tc.constraint_name AND ccu.table_schema = tc.table_schema JOIN information_schema.referential_constraints rc ON rc.constraint_name = tc.constraint_name AND rc.constraint_schema = tc.table_schema WHERE tc.constraint_type = 'FOREIGN KEY' AND ccu.table_name = $1 AND tc.table_schema = $2";

        let rows = sqlx::query(query)
            .bind(table)
            .bind("public")
            .fetch_all(executor)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        let mut backrefs = Vec::new();
        for row in rows {
            backrefs.push(BackRef {
                source_table: row
                    .try_get("source_table")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                source_column: row
                    .try_get("source_column")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                target_table: row
                    .try_get("target_table")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                target_column: row
                    .try_get("target_column")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                constraint_name: row
                    .try_get("constraint_name")
                    .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                on_delete: CascadeRule::from_sql(
                    &row.try_get::<String, _>("delete_rule")
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                ),
                on_update: CascadeRule::from_sql(
                    &row.try_get::<String, _>("update_rule")
                        .map_err(|e| DataBridgeError::Database(e.to_string()))?,
                ),
            });
        }

        Ok(backrefs)
    }

    // ============================================================================
    // Many-to-Many Operations
    // ============================================================================

    /// Create a join table for many-to-many relationship if it doesn't exist
    pub async fn create_join_table(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_table: &str,
    ) -> Result<()> {
        // Validate identifiers
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;
        QueryBuilder::validate_identifier(&config.target_key)?;
        QueryBuilder::validate_identifier(&config.target_table)?;
        QueryBuilder::validate_identifier(source_table)?;

        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS "{}" (
                "{}" INTEGER NOT NULL REFERENCES "{}"("{}") ON DELETE CASCADE,
                "{}" INTEGER NOT NULL REFERENCES "{}"("{}") ON DELETE CASCADE,
                PRIMARY KEY ("{}", "{}")
            )
            "#,
            config.join_table,
            config.source_key,
            source_table,
            config.source_reference,
            config.target_key,
            config.target_table,
            config.target_reference,
            config.source_key,
            config.target_key
        );

        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;
        Ok(())
    }

    /// Add a relation between source and target in the join table
    pub async fn add_m2m_relation(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<()> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;
        QueryBuilder::validate_identifier(&config.target_key)?;

        let sql = format!(
            r#"INSERT INTO "{}" ("{}", "{}") VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
            config.join_table, config.source_key, config.target_key
        );

        sqlx::query(&sql)
            .bind(source_id)
            .bind(target_id)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove a relation between source and target from the join table
    pub async fn remove_m2m_relation(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<u64> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;
        QueryBuilder::validate_identifier(&config.target_key)?;

        let sql = format!(
            r#"DELETE FROM "{}" WHERE "{}" = $1 AND "{}" = $2"#,
            config.join_table, config.source_key, config.target_key
        );

        let result = sqlx::query(&sql)
            .bind(source_id)
            .bind(target_id)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Remove all relations for a source from the join table
    pub async fn clear_m2m_relations(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
    ) -> Result<u64> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;

        let sql = format!(
            r#"DELETE FROM "{}" WHERE "{}" = $1"#,
            config.join_table, config.source_key
        );

        let result = sqlx::query(&sql)
            .bind(source_id)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Fetch all related target records for a source through the join table
    pub async fn fetch_m2m_related(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
        select_columns: Option<&[&str]>,
        order_by: Option<&[(&str, &str)]>,
        limit: Option<i64>,
    ) -> Result<Vec<HashMap<String, ExtractedValue>>> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;
        QueryBuilder::validate_identifier(&config.target_key)?;
        QueryBuilder::validate_identifier(&config.target_table)?;
        QueryBuilder::validate_identifier(&config.target_reference)?;

        // Validate select columns if provided
        let columns = match select_columns {
            Some(cols) => {
                for col in cols {
                    QueryBuilder::validate_identifier(col)?;
                }
                cols.iter()
                    .map(|c| format!(r#"t."{}""#, c))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            None => "t.*".to_string(),
        };

        // Build ORDER BY clause
        let order_clause = match order_by {
            Some(orders) => {
                let mut parts = Vec::new();
                for (col, dir) in orders {
                    QueryBuilder::validate_identifier(col)?;
                    let direction = if dir.to_lowercase() == "desc" {
                        "DESC"
                    } else {
                        "ASC"
                    };
                    parts.push(format!(r#"t."{}" {}"#, col, direction));
                }
                format!("ORDER BY {}", parts.join(", "))
            }
            None => String::new(),
        };

        // Build LIMIT clause
        let limit_clause = match limit {
            Some(n) => format!("LIMIT {}", n),
            None => String::new(),
        };

        let sql = format!(
            r#"
            SELECT {columns}
            FROM "{target_table}" t
            INNER JOIN "{join_table}" j ON t."{target_ref}" = j."{target_key}"
            WHERE j."{source_key}" = $1
            {order_clause}
            {limit_clause}
            "#,
            columns = columns,
            target_table = config.target_table,
            join_table = config.join_table,
            target_ref = config.target_reference,
            target_key = config.target_key,
            source_key = config.source_key,
            order_clause = order_clause,
            limit_clause = limit_clause,
        );

        let rows = sqlx::query(&sql)
            .bind(source_id)
            .fetch_all(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(crate::row_to_extracted(&row)?);
        }

        Ok(results)
    }

    /// Count the number of related target records for a source
    pub async fn count_m2m_related(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
    ) -> Result<i64> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;

        let sql = format!(
            r#"SELECT COUNT(*) as count FROM "{}" WHERE "{}" = $1"#,
            config.join_table, config.source_key
        );

        let row = sqlx::query(&sql)
            .bind(source_id)
            .fetch_one(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Check if a relation exists between source and target
    pub async fn has_m2m_relation(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<bool> {
        QueryBuilder::validate_identifier(&config.join_table)?;
        QueryBuilder::validate_identifier(&config.source_key)?;
        QueryBuilder::validate_identifier(&config.target_key)?;

        let sql = format!(
            r#"SELECT 1 FROM "{}" WHERE "{}" = $1 AND "{}" = $2 LIMIT 1"#,
            config.join_table, config.source_key, config.target_key
        );

        let result = sqlx::query(&sql)
            .bind(source_id)
            .bind(target_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| DataBridgeError::Database(e.to_string()))?;

        Ok(result.is_some())
    }

    /// Set the exact list of related targets (remove old, add new)
    pub async fn set_m2m_relations(
        pool: &PgPool,
        config: &crate::schema::ManyToManyConfig,
        source_id: i64,
        target_ids: &[i64],
    ) -> Result<()> {
        // Clear existing relations
        Self::clear_m2m_relations(pool, config, source_id).await?;

        // Add new relations
        for &target_id in target_ids {
            Self::add_m2m_relation(pool, config, source_id, target_id).await?;
        }

        Ok(())
    }
}

/// Helper function to convert ExtractedValue to JSON.
fn extracted_value_to_json(value: &ExtractedValue) -> Result<JsonValue> {
    Ok(match value {
        ExtractedValue::Null => JsonValue::Null,
        ExtractedValue::Bool(v) => JsonValue::Bool(*v),
        ExtractedValue::SmallInt(v) => JsonValue::Number((*v).into()),
        ExtractedValue::Int(v) => JsonValue::Number((*v).into()),
        ExtractedValue::BigInt(v) => JsonValue::Number((*v).into()),
        ExtractedValue::Float(v) => serde_json::Number::from_f64(*v as f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        ExtractedValue::Double(v) => serde_json::Number::from_f64(*v)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        ExtractedValue::String(v) => JsonValue::String(v.clone()),
        ExtractedValue::Bytes(v) => {
            let hex_string = v.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            JsonValue::String(hex_string)
        }
        ExtractedValue::Uuid(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Date(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Time(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Timestamp(v) => JsonValue::String(v.to_string()),
        ExtractedValue::TimestampTz(v) => JsonValue::String(v.to_rfc3339()),
        ExtractedValue::Json(v) => v.clone(),
        ExtractedValue::Array(values) => {
            let json_values: Vec<JsonValue> = values
                .iter()
                .map(extracted_value_to_json)
                .collect::<Result<Vec<_>>>()?;
            JsonValue::Array(json_values)
        }
        ExtractedValue::Decimal(v) => JsonValue::String(v.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_row_data_creation() {
        let mut columns = HashMap::new();
        columns.insert("id".to_string(), ExtractedValue::BigInt(42));
        columns.insert(
            "name".to_string(),
            ExtractedValue::String("Alice".to_string()),
        );
        let row = Row::new(columns.clone());
        assert!(matches!(row.get("id"), Ok(ExtractedValue::BigInt(42))));
        assert_eq!(row.columns_map(), &columns);
    }
}
