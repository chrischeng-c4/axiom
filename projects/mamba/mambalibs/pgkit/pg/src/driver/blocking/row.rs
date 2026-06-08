//! Blocking facade entry points for [`crate::driver::Row`].

// HANDWRITE-BEGIN reason: facade-mirror codegen gap; no section type
//   today emits `rt.block_on(async_fn(...))` wrappers from an async
//   signature. Closes when score grows a `facade-mirror` section
//   type or the regenerability invariant covers signature-driven
//   blocking-sibling emission.
//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-blocking-facade-row-bulk-executor.md#changes

use std::collections::HashMap;

use crate::driver::blocking::Connection;
use crate::driver::row::RelationConfig;
use crate::driver::{ExtractedValue, Row};
use crate::query::{Operator, OrderDirection};
use crate::schema::ManyToManyConfig;
use crate::QueryBuilder;
use crate::Result;

impl Row {
    pub fn insert_blocking(
        conn: &Connection,
        table: &str,
        values: &[(String, ExtractedValue)],
    ) -> Result<Self> {
        conn.runtime()
            .block_on(Row::insert(conn.as_async().pool(), table, values))
    }

    pub fn insert_many_blocking(
        conn: &Connection,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
    ) -> Result<Vec<Self>> {
        conn.runtime()
            .block_on(Row::insert_many(conn.as_async().pool(), table, rows))
    }

    pub fn upsert_blocking(
        conn: &Connection,
        table: &str,
        values: &[(String, ExtractedValue)],
        conflict_target: &[String],
        update_columns: Option<&[String]>,
    ) -> Result<Self> {
        conn.runtime().block_on(Row::upsert(
            conn.as_async().pool(),
            table,
            values,
            conflict_target,
            update_columns,
        ))
    }

    pub fn upsert_many_blocking(
        conn: &Connection,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
        conflict_target: &[String],
        update_columns: Option<&[String]>,
    ) -> Result<Vec<Self>> {
        conn.runtime().block_on(Row::upsert_many(
            conn.as_async().pool(),
            table,
            rows,
            conflict_target,
            update_columns,
        ))
    }

    pub fn find_by_id_blocking(conn: &Connection, table: &str, id: i64) -> Result<Option<Self>> {
        conn.runtime()
            .block_on(Row::find_by_id(conn.as_async().pool(), table, id))
    }

    pub fn find_many_blocking(
        conn: &Connection,
        table: &str,
        query: Option<&QueryBuilder>,
    ) -> Result<Vec<Self>> {
        conn.runtime()
            .block_on(Row::find_many(conn.as_async().pool(), table, query))
    }

    pub fn update_blocking(
        conn: &Connection,
        table: &str,
        id: i64,
        values: &[(String, ExtractedValue)],
    ) -> Result<bool> {
        conn.runtime()
            .block_on(Row::update(conn.as_async().pool(), table, id, values))
    }

    pub fn delete_blocking(conn: &Connection, table: &str, id: i64) -> Result<bool> {
        conn.runtime()
            .block_on(Row::delete(conn.as_async().pool(), table, id))
    }

    pub fn count_blocking(
        conn: &Connection,
        table: &str,
        query: Option<&QueryBuilder>,
    ) -> Result<i64> {
        conn.runtime()
            .block_on(Row::count(conn.as_async().pool(), table, query))
    }

    pub fn find_with_relations_blocking(
        conn: &Connection,
        table: &str,
        id: i64,
        relations: &[RelationConfig],
    ) -> Result<Option<Self>> {
        conn.runtime().block_on(Row::find_with_relations(
            conn.as_async().pool(),
            table,
            id,
            relations,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn find_many_with_relations_blocking(
        conn: &Connection,
        table: &str,
        relations: &[RelationConfig],
        where_clause: Option<(&str, Operator, ExtractedValue)>,
        order_by: Option<(&str, OrderDirection)>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>> {
        conn.runtime().block_on(Row::find_many_with_relations(
            conn.as_async().pool(),
            table,
            relations,
            where_clause,
            order_by,
            limit,
            offset,
        ))
    }

    pub fn find_one_eager_blocking(
        conn: &Connection,
        table: &str,
        id: i64,
        joins: &[(&str, &str, &str)],
    ) -> Result<Option<Self>> {
        conn.runtime().block_on(Row::find_one_eager(
            conn.as_async().pool(),
            table,
            id,
            joins,
        ))
    }

    pub fn delete_with_cascade_blocking(
        conn: &Connection,
        table: &str,
        id: i64,
        id_column: &str,
    ) -> Result<u64> {
        conn.runtime().block_on(Row::delete_with_cascade(
            conn.as_async().pool(),
            table,
            id,
            id_column,
        ))
    }

    pub fn delete_checked_blocking(
        conn: &Connection,
        table: &str,
        id: i64,
        id_column: &str,
    ) -> Result<u64> {
        conn.runtime().block_on(Row::delete_checked(
            conn.as_async().pool(),
            table,
            id,
            id_column,
        ))
    }

    pub fn create_join_table_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_table: &str,
    ) -> Result<()> {
        conn.runtime().block_on(Row::create_join_table(
            conn.as_async().pool(),
            config,
            source_table,
        ))
    }

    pub fn add_m2m_relation_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<()> {
        conn.runtime().block_on(Row::add_m2m_relation(
            conn.as_async().pool(),
            config,
            source_id,
            target_id,
        ))
    }

    pub fn remove_m2m_relation_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<u64> {
        conn.runtime().block_on(Row::remove_m2m_relation(
            conn.as_async().pool(),
            config,
            source_id,
            target_id,
        ))
    }

    pub fn clear_m2m_relations_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
    ) -> Result<u64> {
        conn.runtime().block_on(Row::clear_m2m_relations(
            conn.as_async().pool(),
            config,
            source_id,
        ))
    }

    pub fn fetch_m2m_related_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
        select_columns: Option<&[&str]>,
        order_by: Option<&[(&str, &str)]>,
        limit: Option<i64>,
    ) -> Result<Vec<HashMap<String, ExtractedValue>>> {
        conn.runtime().block_on(Row::fetch_m2m_related(
            conn.as_async().pool(),
            config,
            source_id,
            select_columns,
            order_by,
            limit,
        ))
    }

    pub fn count_m2m_related_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
    ) -> Result<i64> {
        conn.runtime().block_on(Row::count_m2m_related(
            conn.as_async().pool(),
            config,
            source_id,
        ))
    }

    pub fn has_m2m_relation_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
        target_id: i64,
    ) -> Result<bool> {
        conn.runtime().block_on(Row::has_m2m_relation(
            conn.as_async().pool(),
            config,
            source_id,
            target_id,
        ))
    }

    pub fn set_m2m_relations_blocking(
        conn: &Connection,
        config: &ManyToManyConfig,
        source_id: i64,
        target_ids: &[i64],
    ) -> Result<()> {
        conn.runtime().block_on(Row::set_m2m_relations(
            conn.as_async().pool(),
            config,
            source_id,
            target_ids,
        ))
    }
}

// HANDWRITE-END
