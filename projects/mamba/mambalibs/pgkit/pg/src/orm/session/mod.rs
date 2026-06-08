//! ORM Session: identity map + unit-of-work staging.
//!
//! Async-first surface; the blocking sibling lives at
//! [`crate::orm::session::blocking`].
//!
//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes

// HANDWRITE-BEGIN reason: orm-session machinery; no section type
//   today emits identity-map + UoW state from a spec. Closes when
//   score grows an `orm-session` section type that emits the
//   Session struct + trait + flush state machine from a structured
//   definition.

use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use sqlx::postgres::PgArguments;
use sqlx::{Postgres, Transaction as SqlxTx};

use crate::driver::{Connection, ExtractedValue, Row};
use crate::orm::query::{Operator, QueryBuilder};
use crate::{DataBridgeError, Result};

pub mod blocking;

const DEFAULT_PK_COLUMN: &str = "id";

/// Sealing sub-module for [`SessionModel`].
///
/// Soft-seal: the trait is public but `#[doc(hidden)]`. v1 callers
/// (including the integration test fixture) impl `Sealed` directly;
/// #2089's `#[derive(Model)]` macro will subsume it.
pub mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Trait implemented by model types managed by a [`Session`].
///
/// v1 requires hand-implementation (alongside [`sealed::Sealed`]). The
/// future declarative-model macro (issue #2089) auto-derives both.
pub trait SessionModel: sealed::Sealed + Any + Send + Sync + 'static {
    fn table() -> &'static str;
    fn pk_column() -> &'static str {
        DEFAULT_PK_COLUMN
    }
    fn pk(&self) -> i64;
    fn to_values(&self) -> Vec<(String, ExtractedValue)>;
    fn from_row(row: &Row) -> Result<Self>
    where
        Self: Sized;
}

/// Unit-of-work staging entries. Drained in INSERT → UPDATE → DELETE
/// order by [`Session::flush`].
enum UowEntry {
    Insert {
        table: String,
        values: Vec<(String, ExtractedValue)>,
        slot_pk: Option<Arc<Mutex<i64>>>,
    },
    Update {
        table: String,
        pk: i64,
        values: Vec<(String, ExtractedValue)>,
    },
    Delete {
        table: String,
        pk: i64,
    },
}

/// ORM session bound to a [`Connection`].
///
/// Holds a per-session identity map and unit-of-work staging vector.
/// `add` / `delete` / `touch` are sync — they only mutate the staging
/// vector. SQL is emitted on `flush` / `commit`.
pub struct Session<'a> {
    conn: &'a Connection,
    tx: Option<SqlxTx<'static, Postgres>>,
    identity_map: HashMap<(String, i64), Arc<dyn Any + Send + Sync>>,
    dyn_identity_map: HashMap<(String, i64), HashMap<String, ExtractedValue>>,
    staging: Vec<UowEntry>,
}

impl<'a> Session<'a> {
    /// New session bound to `conn`. No IO.
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            tx: None,
            identity_map: HashMap::new(),
            dyn_identity_map: HashMap::new(),
            staging: Vec::new(),
        }
    }

    /// The underlying connection.
    pub fn connection(&self) -> &'a Connection {
        self.conn
    }

    /// Open a long-running transaction. Subsequent staged operations
    /// flushed before `commit` / `rollback` run inside this tx.
    pub async fn begin(&mut self) -> Result<()> {
        if self.tx.is_some() {
            return Err(DataBridgeError::Internal(
                "Session::begin called while a transaction is already open".into(),
            ));
        }
        let tx = self
            .conn
            .pool()
            .begin()
            .await
            .map_err(|e| DataBridgeError::Connection(format!("begin: {e}")))?;
        self.tx = Some(tx);
        Ok(())
    }

    /// Drain staging and commit. If no explicit `begin`, opens a
    /// short-lived tx inside `flush` and commits it.
    pub async fn commit(&mut self) -> Result<()> {
        self.flush().await?;
        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .map_err(|e| DataBridgeError::Database(format!("commit: {e}")))?;
        }
        self.staging.clear();
        Ok(())
    }

    /// Roll back the outer transaction (if any) and clear both the
    /// staging vector and the identity map.
    pub async fn rollback(&mut self) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .map_err(|e| DataBridgeError::Database(format!("rollback: {e}")))?;
        }
        self.staging.clear();
        self.identity_map.clear();
        self.dyn_identity_map.clear();
        Ok(())
    }

    /// Drain the staging vector in canonical INSERT → UPDATE → DELETE
    /// order, intra-pass order preserved. If `begin` has been called,
    /// runs inside that transaction; otherwise opens + commits a
    /// short-lived one.
    pub async fn flush(&mut self) -> Result<()> {
        if self.staging.is_empty() {
            return Ok(());
        }
        let staging = std::mem::take(&mut self.staging);
        let mut inserts: Vec<UowEntry> = Vec::new();
        let mut updates: Vec<UowEntry> = Vec::new();
        let mut deletes: Vec<UowEntry> = Vec::new();
        for entry in staging {
            match entry {
                UowEntry::Insert { .. } => inserts.push(entry),
                UowEntry::Update { .. } => updates.push(entry),
                UowEntry::Delete { .. } => deletes.push(entry),
            }
        }

        let mut auto_commit_tx: Option<SqlxTx<'static, Postgres>> = None;
        if self.tx.is_none() {
            let t = self
                .conn
                .pool()
                .begin()
                .await
                .map_err(|e| DataBridgeError::Connection(format!("flush.begin: {e}")))?;
            auto_commit_tx = Some(t);
        }
        let tx_ref: &mut SqlxTx<'static, Postgres> = if let Some(t) = auto_commit_tx.as_mut() {
            t
        } else {
            self.tx.as_mut().expect("tx checked above")
        };

        let drain_result = drain_all(tx_ref, inserts, updates, deletes).await;

        match (drain_result, auto_commit_tx) {
            (Ok(()), Some(t)) => {
                t.commit()
                    .await
                    .map_err(|e| DataBridgeError::Database(format!("flush.commit: {e}")))?;
                Ok(())
            }
            (Ok(()), None) => Ok(()),
            (Err(e), Some(t)) => {
                let _ = t.rollback().await;
                Err(e)
            }
            (Err(e), None) => Err(e),
        }
    }

    /// Stage an INSERT of `model`. Sync — no IO until `flush`.
    pub fn add<M: SessionModel>(&mut self, model: Arc<M>) {
        let table = M::table().to_string();
        let values = model.to_values();
        self.staging.push(UowEntry::Insert {
            table,
            values,
            slot_pk: None,
        });
        let _ = model;
    }

    /// Stage a DELETE by primary key.
    pub fn delete<M: SessionModel>(&mut self, pk: i64) {
        self.staging.push(UowEntry::Delete {
            table: M::table().to_string(),
            pk,
        });
        self.identity_map.remove(&(M::table().to_string(), pk));
    }

    /// Mark `model` dirty so its current field values are flushed as
    /// an UPDATE on next `flush`.
    pub fn touch<M: SessionModel>(&mut self, model: Arc<M>) {
        let table = M::table().to_string();
        let pk = model.pk();
        let values = model.to_values();
        self.staging.push(UowEntry::Update {
            table: table.clone(),
            pk,
            values,
        });
        self.identity_map
            .insert((table, pk), model as Arc<dyn Any + Send + Sync>);
    }

    /// Fetch the row with `pk` from the database, caching the result
    /// in the identity map. Repeat calls return the same `Arc`.
    pub async fn get<M: SessionModel>(&mut self, pk: i64) -> Result<Option<Arc<M>>> {
        let key = (M::table().to_string(), pk);
        if let Some(existing) = self.identity_map.get(&key) {
            return Arc::clone(existing).downcast::<M>().map(Some).map_err(|_| {
                DataBridgeError::Internal(format!(
                    "identity-map type mismatch for ({}, {})",
                    M::table(),
                    pk
                ))
            });
        }
        let row = Row::find_by_id(self.conn.pool(), M::table(), pk).await?;
        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };
        let model = M::from_row(&row)?;
        let arc = Arc::new(model);
        self.identity_map
            .insert(key, Arc::clone(&arc) as Arc<dyn Any + Send + Sync>);
        Ok(Some(arc))
    }

    /// Start a query whose results are routed through the identity
    /// map.
    pub fn query<M: SessionModel>(&mut self) -> Result<SessionQuery<'a, '_, M>> {
        let builder = QueryBuilder::new(M::table())?;
        Ok(SessionQuery {
            session: self,
            builder,
            _marker: PhantomData,
        })
    }

    /// Build a [`SessionQuery`] with a pre-populated builder. Used by
    /// the blocking facade to thread chainable-API state through the
    /// `block_on` boundary.
    pub(crate) fn query_with_builder<M: SessionModel>(
        &mut self,
        builder: QueryBuilder,
    ) -> SessionQuery<'a, '_, M> {
        SessionQuery {
            session: self,
            builder,
            _marker: PhantomData,
        }
    }

    // ── dyn surface (used by the pgkit-binding binding) ──────────

    /// Stage an INSERT against `table` with no compile-time model.
    /// Returns a slot the caller can read after `flush` to recover the
    /// assigned primary key.
    pub fn add_dyn(
        &mut self,
        table: &str,
        values: Vec<(String, ExtractedValue)>,
    ) -> Arc<Mutex<i64>> {
        let slot = Arc::new(Mutex::new(0i64));
        self.staging.push(UowEntry::Insert {
            table: table.to_string(),
            values,
            slot_pk: Some(Arc::clone(&slot)),
        });
        slot
    }

    /// Stage a DELETE of `(table, pk)`.
    pub fn delete_dyn(&mut self, table: &str, pk: i64) {
        self.staging.push(UowEntry::Delete {
            table: table.to_string(),
            pk,
        });
        self.dyn_identity_map.remove(&(table.to_string(), pk));
    }

    /// Stage an UPDATE of `(table, pk)` with the given values.
    pub fn touch_dyn(&mut self, table: &str, pk: i64, values: Vec<(String, ExtractedValue)>) {
        self.staging.push(UowEntry::Update {
            table: table.to_string(),
            pk,
            values,
        });
    }

    /// Fetch a single row by `(table, pk)`, caching the dict in the
    /// dyn identity map.
    pub async fn get_dyn(
        &mut self,
        table: &str,
        pk: i64,
    ) -> Result<Option<HashMap<String, ExtractedValue>>> {
        let key = (table.to_string(), pk);
        if let Some(existing) = self.dyn_identity_map.get(&key) {
            return Ok(Some(existing.clone()));
        }
        let row = Row::find_by_id(self.conn.pool(), table, pk).await?;
        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };
        let dict = row.columns_map().clone();
        self.dyn_identity_map.insert(key, dict.clone());
        Ok(Some(dict))
    }

    /// Fetch all rows matching the given equality filter.
    pub async fn query_all_dyn(
        &mut self,
        table: &str,
        filter: &[(String, ExtractedValue)],
    ) -> Result<Vec<HashMap<String, ExtractedValue>>> {
        let mut qb = QueryBuilder::new(table)?;
        for (col, val) in filter {
            qb = qb.where_clause(col, Operator::Eq, val.clone())?;
        }
        let rows = Row::find_many(self.conn.pool(), table, Some(&qb)).await?;
        Ok(rows.into_iter().map(|r| r.columns_map().clone()).collect())
    }

    /// Fetch the first row matching the given equality filter.
    pub async fn query_first_dyn(
        &mut self,
        table: &str,
        filter: &[(String, ExtractedValue)],
    ) -> Result<Option<HashMap<String, ExtractedValue>>> {
        let mut qb = QueryBuilder::new(table)?;
        for (col, val) in filter {
            qb = qb.where_clause(col, Operator::Eq, val.clone())?;
        }
        qb = qb.limit(1);
        let rows = Row::find_many(self.conn.pool(), table, Some(&qb)).await?;
        Ok(rows.into_iter().next().map(|r| r.columns_map().clone()))
    }

    // ── Test-only accessors (also useful for diagnostics) ─────────

    /// Number of staged unit-of-work entries.
    pub fn staging_len(&self) -> usize {
        self.staging.len()
    }

    /// Total entries in both typed and dyn identity maps.
    pub fn identity_map_len(&self) -> usize {
        self.identity_map.len() + self.dyn_identity_map.len()
    }

    /// True iff `begin` has been called and no terminal commit/rollback
    /// has fired since.
    pub fn in_transaction(&self) -> bool {
        self.tx.is_some()
    }
}

/// Builder returned from [`Session::query`]. Terminators register
/// results in the identity map keyed by `(M::table(), m.pk())`.
pub struct SessionQuery<'a, 'sess, M: SessionModel> {
    session: &'sess mut Session<'a>,
    builder: QueryBuilder,
    _marker: PhantomData<M>,
}

impl<'a, 'sess, M: SessionModel> SessionQuery<'a, 'sess, M> {
    /// Append a `WHERE` predicate.
    pub fn filter(mut self, field: &str, op: Operator, value: ExtractedValue) -> Result<Self> {
        self.builder = self.builder.where_clause(field, op, value)?;
        Ok(self)
    }

    /// Append a `LIMIT` clause.
    pub fn limit(mut self, n: i64) -> Self {
        self.builder = self.builder.limit(n);
        self
    }

    /// Materialise all matching rows.
    pub async fn all(self) -> Result<Vec<Arc<M>>> {
        let table = M::table().to_string();
        let pool = self.session.conn.pool().clone();
        let rows = Row::find_many(&pool, &table, Some(&self.builder)).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let model = M::from_row(row)?;
            let pk = model.pk();
            let key = (table.clone(), pk);
            let arc = match self.session.identity_map.get(&key) {
                Some(existing) => Arc::clone(existing).downcast::<M>().map_err(|_| {
                    DataBridgeError::Internal(format!(
                        "identity-map type mismatch for ({table}, {pk})"
                    ))
                })?,
                None => {
                    let arc = Arc::new(model);
                    self.session
                        .identity_map
                        .insert(key, Arc::clone(&arc) as Arc<dyn Any + Send + Sync>);
                    arc
                }
            };
            out.push(arc);
        }
        Ok(out)
    }

    /// Materialise the first matching row.
    pub async fn first(self) -> Result<Option<Arc<M>>> {
        let limited = self.limit(1);
        Ok(limited.all().await?.into_iter().next())
    }
}

// ── Flush drain helpers ──────────────────────────────────────────

async fn drain_all(
    tx: &mut SqlxTx<'static, Postgres>,
    inserts: Vec<UowEntry>,
    updates: Vec<UowEntry>,
    deletes: Vec<UowEntry>,
) -> Result<()> {
    for entry in inserts {
        if let UowEntry::Insert {
            table,
            values,
            slot_pk,
        } = entry
        {
            if values.is_empty() {
                return Err(DataBridgeError::Query(
                    "Session: cannot INSERT empty row".into(),
                ));
            }
            let row = Row::insert(&mut **tx, &table, &values).await?;
            if let Some(slot) = slot_pk {
                if let Ok(ExtractedValue::BigInt(pk)) = row.get(DEFAULT_PK_COLUMN) {
                    if let Ok(mut g) = slot.lock() {
                        *g = *pk;
                    }
                }
            }
        }
    }
    for entry in updates {
        if let UowEntry::Update { table, pk, values } = entry {
            if values.is_empty() {
                continue;
            }
            let mut qb = QueryBuilder::new(&table)?;
            qb = qb.where_clause(DEFAULT_PK_COLUMN, Operator::Eq, ExtractedValue::BigInt(pk))?;
            let (sql, params) = qb.build_update(&values)?;
            let mut args = PgArguments::default();
            for p in &params {
                p.bind_to_arguments(&mut args)?;
            }
            sqlx::query_with(&sql, args)
                .execute(&mut **tx)
                .await
                .map_err(|e| DataBridgeError::Database(format!("session update: {e}")))?;
        }
    }
    for entry in deletes {
        if let UowEntry::Delete { table, pk } = entry {
            let mut qb = QueryBuilder::new(&table)?;
            qb = qb.where_clause(DEFAULT_PK_COLUMN, Operator::Eq, ExtractedValue::BigInt(pk))?;
            let (sql, params) = qb.build_delete();
            let mut args = PgArguments::default();
            for p in &params {
                p.bind_to_arguments(&mut args)?;
            }
            sqlx::query_with(&sql, args)
                .execute(&mut **tx)
                .await
                .map_err(|e| DataBridgeError::Database(format!("session delete: {e}")))?;
        }
    }
    Ok(())
}

// HANDWRITE-END
