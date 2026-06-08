//! Driver-backed execution helpers for QueryBuilder.

use crate::driver::{Connection, Row};
use crate::Result;

use super::builder::QueryBuilder;

impl QueryBuilder {
    /// Builds this SELECT query and fetches all rows through the driver layer.
    pub async fn fetch_rows(&self, conn: &Connection) -> Result<Vec<Row>> {
        let (sql, params) = self.build();
        conn.fetch_rows(&sql, &params).await
    }

    /// Builds this SELECT query and fetches at most one row through the driver layer.
    pub async fn fetch_optional_row(&self, conn: &Connection) -> Result<Option<Row>> {
        let (sql, params) = self.build();
        conn.fetch_optional_row(&sql, &params).await
    }
}
