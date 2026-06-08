//! Blocking facade over [`crate::orm::Session`].
//!
//! @spec
//!   .aw/tech-design/projects/pg/specs/pg-orm-session-uow.md#changes

// HANDWRITE-BEGIN reason: facade-mirror codegen gap — same as
//   #2086. Closes when score grows a `facade-mirror` section type
//   that emits `rt.block_on(async_fn(...))` wrappers from an async
//   signature.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use crate::driver::blocking::Connection as BlockingConnection;
use crate::driver::ExtractedValue;
use crate::orm::query::{Operator, QueryBuilder};
use crate::orm::session::{Session as AsyncSession, SessionModel};
use crate::Result;

/// Blocking facade over [`AsyncSession`].
pub struct Session<'a> {
    inner: AsyncSession<'a>,
    rt: Arc<Runtime>,
}

impl<'a> Session<'a> {
    /// Build a blocking Session bound to `conn`. No IO.
    pub fn new(conn: &'a BlockingConnection) -> Self {
        Self {
            inner: AsyncSession::new(conn.as_async()),
            rt: conn.runtime(),
        }
    }

    /// The underlying async session (for borrow plumbing on the dyn
    /// surface).
    pub fn as_async(&self) -> &AsyncSession<'a> {
        &self.inner
    }

    /// Open a long-running transaction.
    pub fn begin(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.begin())
    }

    /// Drain staging and commit.
    pub fn commit(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.commit())
    }

    /// Roll back outer tx (if any) and clear staging + identity map.
    pub fn rollback(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.rollback())
    }

    /// Drain the staging vector in canonical order.
    pub fn flush(&mut self) -> Result<()> {
        self.rt.block_on(self.inner.flush())
    }

    /// Stage an INSERT. Sync — no IO.
    pub fn add<M: SessionModel>(&mut self, model: Arc<M>) {
        self.inner.add::<M>(model)
    }

    /// Stage a DELETE. Sync — no IO.
    pub fn delete<M: SessionModel>(&mut self, pk: i64) {
        self.inner.delete::<M>(pk)
    }

    /// Mark `model` dirty for UPDATE on next flush. Sync — no IO.
    pub fn touch<M: SessionModel>(&mut self, model: Arc<M>) {
        self.inner.touch::<M>(model)
    }

    /// Fetch + cache via the identity map.
    pub fn get<M: SessionModel>(&mut self, pk: i64) -> Result<Option<Arc<M>>> {
        self.rt.block_on(self.inner.get::<M>(pk))
    }

    /// Returns a blocking query builder bound to this session.
    pub fn query<M: SessionModel>(&mut self) -> Result<SessionQuery<'a, '_, M>> {
        let builder = QueryBuilder::new(M::table())?;
        Ok(SessionQuery {
            session: self,
            builder,
            _marker: std::marker::PhantomData,
        })
    }

    // ── dyn surface (used by the pgkit-binding binding) ──────────

    /// Stage an INSERT against `table` (untyped). Returns a slot the
    /// caller can read after `flush` to recover the assigned pk.
    pub fn add_dyn(
        &mut self,
        table: &str,
        values: Vec<(String, ExtractedValue)>,
    ) -> Arc<Mutex<i64>> {
        self.inner.add_dyn(table, values)
    }

    /// Stage a DELETE of `(table, pk)`.
    pub fn delete_dyn(&mut self, table: &str, pk: i64) {
        self.inner.delete_dyn(table, pk)
    }

    /// Stage an UPDATE.
    pub fn touch_dyn(
        &mut self,
        table: &str,
        pk: i64,
        values: Vec<(String, ExtractedValue)>,
    ) {
        self.inner.touch_dyn(table, pk, values)
    }

    /// Fetch a single row by `(table, pk)`.
    pub fn get_dyn(
        &mut self,
        table: &str,
        pk: i64,
    ) -> Result<Option<HashMap<String, ExtractedValue>>> {
        self.rt.block_on(self.inner.get_dyn(table, pk))
    }

    /// Fetch all rows matching the given equality filter.
    pub fn query_all_dyn(
        &mut self,
        table: &str,
        filter: &[(String, ExtractedValue)],
    ) -> Result<Vec<HashMap<String, ExtractedValue>>> {
        self.rt.block_on(self.inner.query_all_dyn(table, filter))
    }

    /// Fetch the first row matching the given equality filter.
    pub fn query_first_dyn(
        &mut self,
        table: &str,
        filter: &[(String, ExtractedValue)],
    ) -> Result<Option<HashMap<String, ExtractedValue>>> {
        self.rt.block_on(self.inner.query_first_dyn(table, filter))
    }

    /// Number of staged unit-of-work entries.
    pub fn staging_len(&self) -> usize {
        self.inner.staging_len()
    }

    /// Total entries in both typed and dyn identity maps.
    pub fn identity_map_len(&self) -> usize {
        self.inner.identity_map_len()
    }

    /// True iff `begin` has been called and no terminal commit/rollback
    /// has fired since.
    pub fn in_transaction(&self) -> bool {
        self.inner.in_transaction()
    }
}

/// Blocking sibling of [`crate::orm::session::SessionQuery`].
pub struct SessionQuery<'a, 'sess, M: SessionModel> {
    session: &'sess mut Session<'a>,
    builder: QueryBuilder,
    _marker: std::marker::PhantomData<M>,
}

impl<'a, 'sess, M: SessionModel> SessionQuery<'a, 'sess, M> {
    pub fn filter(
        mut self,
        field: &str,
        op: Operator,
        value: ExtractedValue,
    ) -> Result<Self> {
        self.builder = self.builder.where_clause(field, op, value)?;
        Ok(self)
    }

    pub fn limit(mut self, n: i64) -> Self {
        self.builder = self.builder.limit(n);
        self
    }

    pub fn all(self) -> Result<Vec<Arc<M>>> {
        let rt = self.session.rt.clone();
        let inner = &mut self.session.inner;
        let builder = self.builder;
        rt.block_on(async move {
            let sq = inner.query_with_builder::<M>(builder);
            sq.all().await
        })
    }

    pub fn first(self) -> Result<Option<Arc<M>>> {
        let rt = self.session.rt.clone();
        let inner = &mut self.session.inner;
        let builder = self.builder;
        rt.block_on(async move {
            let sq = inner.query_with_builder::<M>(builder);
            sq.first().await
        })
    }
}

// HANDWRITE-END
