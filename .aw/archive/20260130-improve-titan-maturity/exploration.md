---
id: improve-titan-maturity
type: exploration
created_at: 2026-01-28T08:01:39.445228+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: Upgrade cclab-titan Maturity

## Architecture Overview

`cclab-titan` is currently a PostgreSQL-centric ORM layer. It is used as the backend for `cclab-nucleus` Python library via PyO3. The current implementation is heavily tied to PostgreSQL-specific types (`PgPool`, `PgRow`, `PgArguments`) and SQL syntax (e.g., `::jsonb`, `ARRAY[]`, `DISTINCT ON`).

The Python layer (`python/cclab/titan/session.py`) currently handles the "Unit of Work" logic (identity map, dirty tracking), which contradicts the "Zero Python byte handling" principle and limits performance.

## Key Findings

1.  **Dialect Coupling**: Core structures like `QueryBuilder`, `SchemaInspector`, `Executor`, and `ExtractedValue` are hardcoded for PostgreSQL.
2.  **Missing Unit of Work in Rust**: The session management logic resides in Python, leading to unnecessary overhead and inconsistency across the "sandwich" architecture.
3.  **No Hook System**: There is no existing infrastructure for lifecycle events like `before_insert` or `after_update` in the Rust core.
4.  **Hybrid Properties**: `QueryBuilder` lacks the ability to treat SQL expressions as model attributes, which is a key feature of SQLAlchemy.
5.  **Test Gaps**: Existing tests focus on basic CRUD and migrations for Postgres. Rollback logic and isolation level guarantees are not thoroughly verified.

## Impact Analysis

- **Refactoring for Multi-Dialect**: This is a major architectural change. Introducing a `Dialect` trait will require modifying almost every file in `src/query`, `src/schema`, and `src/executor`.
- **Moving Session to Rust**: This will simplify the Python layer but requires careful design of thread-safe state management in Rust (e.g., using `Arc` and `DashMap` or similar).
- **Breaking Changes**: Moving the session logic to Rust might change some internal Python APIs, but the goal is to maintain SQLAlchemy-like developer experience.

## Proposed Technical Design

### 1. Dialect Abstraction
- Introduce `trait Dialect` in `src/dialect/mod.rs`.
- Implement `PostgresDialect`, `SqliteDialect`, and `MysqlDialect`.
- Move SQL generation logic from `QueryBuilder` to `Dialect`.

### 2. Database Abstraction
- Introduce `trait Database` to abstract over `sqlx::Pool`.
- Implement wrappers for `sqlx::PgPool`, `sqlx::SqlitePool`, and `sqlx::MySqlPool`.

### 3. Rust-based Session & Unit of Work
- Create `src/session.rs` and `src/uow.rs`.
- Implement `IdentityMap` using weak references where possible.
- Implement `DirtyTracker` by snapshotting `Row` data.
- Implement `UnitOfWork` to batch operations.

### 4. Hook System
- Add `hooks` module to `src/session.rs`.
- Support `before_insert`, `after_insert`, `before_update`, `after_update`, `before_delete`, `after_delete`.

### 5. Hybrid Properties
- Extend `QueryBuilder` to allow registering virtual columns that map to SQL expressions.

## Spec Recommendations

1.  `dialect-abstraction.md`: Define the `Dialect` and `Database` traits and their implementations.
2.  `session-unit-of-work.md`: Design the identity map, dirty tracking, and batching logic in Rust.
3.  `hook-system.md`: Define the event lifecycle and registry.
4.  `hybrid-properties.md`: Design the SQL-expression attribute support in `QueryBuilder`.

