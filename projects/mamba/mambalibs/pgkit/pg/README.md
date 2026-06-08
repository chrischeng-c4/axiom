# pgkit

Native PostgreSQL toolkit core for Mamba-facing applications.

## Overview

`projects/mamba/mambalibs/pgkit/pg` owns pgkit's source/logic layer: PostgreSQL driver
access, ORM/query primitives, schema migration, validation, metrics, and the
blocking facade used by language interfaces. Mamba-facing bindings live in the
sibling `projects/mamba/mambalibs/pgkit/binding` crate and expose this core as
`mambalibs.pg`.

- **Zero Mamba heap pressure**: SQL generation and serialization stay in Rust
- **Native async I/O**: Tokio + SQLx drive PostgreSQL access
- **Sync facade**: Binding layers reuse explicit blocking wrappers
- **Parallel processing**: For bulk operations using Rayon
- **Security-first**: Input validation to prevent SQL injection
- **Type safety**: Full type mapping between Mamba values and PostgreSQL

## Architecture

```
Mamba source layer (models, queries, migrations)
          |
     Mamba interface layer (mambalibs.pg)
          |
   Native pgkit core (projects/pgkit/pg/src/)
          |
        SQLx (PostgreSQL driver)
```

## Modules

- **connection**: Connection pooling and management
- **types**: Type mapping between Mamba values and PostgreSQL
- **row**: Row representation for query results
- **query**: Type-safe SQL query builder
- **transaction**: Transaction management with ACID guarantees
- **migration**: Database migration management
- **schema**: Schema introspection utilities

## Type Mapping

| Mamba Type | PostgreSQL Type | ExtractedValue Variant |
|------------|-----------------|------------------------|
| None | NULL | Null |
| bool | BOOLEAN | Bool |
| int (small) | INTEGER | Int |
| int (large) | BIGINT | BigInt |
| float | DOUBLE PRECISION | Double |
| str | TEXT | String |
| bytes | BYTEA | Bytes |
| uuid.UUID | UUID | Uuid |
| datetime.date | DATE | Date |
| datetime.time | TIME | Time |
| datetime.datetime (naive) | TIMESTAMP | Timestamp |
| datetime.datetime (aware) | TIMESTAMPTZ | TimestampTz |
| dict/list | JSONB | Json |
| list[T] | ARRAY | Array |
| Decimal | NUMERIC | Decimal |

## Features

- Connection pooling with configurable min/max connections
- Parameterized queries to prevent SQL injection
- Transaction support with savepoints
- Schema introspection (tables, columns, indexes, foreign keys)
- Migration management (up/down migrations)
- Comprehensive error handling with type conversion

## Status

**Version**: 0.1.0-alpha (Pre-release)

### Implemented
- Connection pooling and management ✅
- Type mapping between Mamba values and PostgreSQL ✅
- Basic CRUD operations (insert, fetch, update, delete) ✅
- Query builder with WHERE, ORDER BY, LIMIT, OFFSET ✅
- Transaction support with isolation levels ✅
- Raw SQL execution ✅
- Schema introspection (tables, columns, indexes, foreign keys) ✅
- Migration management (up/down migrations) ✅
- Advanced query features (JOINs) ✅

### Roadmap (Future Releases)
- Subquery support
- Bulk operations optimization with Rayon
- Connection pool metrics and monitoring
- Prepared statement caching
- Advanced migration features (rollback chains, dry-run)

## Usage

Rust callers depend on the core crate:

```toml
[dependencies]
cclab-pg = { path = "projects/pgkit/pg" }
```

Mamba callers import through the interface crate:

```python
from mambalibs.pg import connect, execute, Session
from mambalibs.pg.migrate import MigrationRunner, Migration
```

## Quick Start

```rust
use cclab_pg::{Connection, PoolConfig, QueryBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection pool
    let conn = Connection::new(
        "postgresql://user:pass@localhost/db",
        PoolConfig::default(),
    )
    .await?;

    // Insert data
    let query = QueryBuilder::new("users")
        .insert(vec![
            ("name", "Alice".into()),
            ("email", "alice@example.com".into()),
        ]);
    let _ = query;

    // Query data
    let query = QueryBuilder::new("users")
        .where_eq("name", "Alice")
        .limit(10);
    let _ = query;
    let rows: Vec<()> = Vec::new();

    println!("Found {} users", rows.len());
    Ok(())
}
```

## Dependencies

- **sqlx** 0.8: Async PostgreSQL driver with connection pooling
- **tokio**: Async runtime
- **serde**: Serialization framework
- **uuid**: UUID support
- **chrono**: Date/time handling
- **rust_decimal**: Precise decimal arithmetic

## Development

```bash
# Check compilation
cargo check -p cclab-pg

# Run unit tests (no database required)
cargo test -p cclab-pg

# Run integration tests (requires PostgreSQL)
cargo test -p cclab-pg --test test_transaction

# Run all tests including ignored ones
cargo test -p cclab-pg -- --ignored

# Lint
cargo clippy -p cclab-pg
```

### PostgreSQL Setup (macOS)

Using Homebrew:

```bash
# Install PostgreSQL
brew install postgresql@15

# Start PostgreSQL service
brew services start postgresql@15

# Create test database
createdb test_db

# Verify connection
psql -d test_db -c "SELECT version();"
```

Set the database URL environment variable:

```bash
# Default connection (local socket)
export DATABASE_URL="postgresql://localhost/test_db"

# Or with explicit credentials
export DATABASE_URL="postgresql://username:password@localhost:5432/test_db"
```

### PostgreSQL Setup (Docker)

```bash
# Start PostgreSQL container
docker run -d --name postgres-test \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=test_db \
  postgres:15

# Set connection URL
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/test_db"

# Run tests
cargo test -p cclab-pg --test test_transaction

# Cleanup
docker stop postgres-test && docker rm postgres-test
```

### Running Migration Tests

The migration integration tests require a running PostgreSQL database. They are marked with `#[ignore]` to prevent them from running in CI without a database.

```bash
# Start PostgreSQL (using Docker)
docker run -d --name postgres-test -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15

# Create test database
docker exec -it postgres-test psql -U postgres -c "CREATE DATABASE test_db;"

# Run migration tests
export POSTGRES_URL="postgresql://postgres:postgres@localhost/test_db"
cargo test -p cclab-pg --test test_migration -- --ignored

# Clean up
docker stop postgres-test && docker rm postgres-test
```

Each test creates its own migration table with a unique name (e.g., `_test_migrations_apply`) to avoid conflicts when running tests in parallel. Tables are automatically cleaned up after each test.

## License

MIT
