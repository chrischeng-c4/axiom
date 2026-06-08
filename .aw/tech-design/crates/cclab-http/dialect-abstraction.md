---
id: dialect-abstraction
type: spec
title: "Dialect and Database Abstraction"
version: 1
spec_type: algorithm
created_at: 2026-01-28T18:10:49.421608+00:00
updated_at: 2026-01-28T18:10:49.421608+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Dialect and Database Abstraction Architecture"
history:
  - timestamp: 2026-01-28T18:10:49.421608+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Dialect and Database Abstraction

## Overview

This specification defines the abstraction layer required to support multiple database dialects (PostgreSQL, SQLite, MySQL) in cclab-titan. It replaces the hardcoded PostgreSQL types and logic with traits and dialect-specific implementations.

## Requirements

### R1 - Dialect Trait

```yaml
id: R1
priority: medium
status: draft
```

Define a Dialect trait that abstracts SQL generation details like identifier quoting, parameter placeholders, and specific SQL syntax (e.g., RETURNING, LIMIT/OFFSET).

### R2 - Database Abstraction

```yaml
id: R2
priority: medium
status: draft
```

Define a Database trait that abstracts connection pooling and transaction management across different sqlx drivers.

### R3 - Dialect Implementations

```yaml
id: R3
priority: medium
status: draft
```

Provide concrete implementations for PostgreSQL, SQLite, and MySQL.

### R4 - QueryBuilder Refactoring

```yaml
id: R4
priority: medium
status: draft
```

Refactor QueryBuilder to use the Dialect trait for all SQL generation tasks.

## Acceptance Criteria

### Scenario: SQL Generation for SQLite

- **GIVEN** A QueryBuilder instance configured with SqliteDialect
- **WHEN** build_select() is called
- **THEN** Generated SQL should use SQLite-specific syntax (e.g., '?' placeholders instead of '$1').

### Scenario: MySQL Connection Initialization Logic

- **GIVEN** A database connection URI for MySQL
- **WHEN** Connection::new() is called with mysql:// URI
- **THEN** The system correctly identifies and initializes a MySQL connection pool using MysqlPool wrapper.

### Scenario: PostgreSQL Schema-Qualified Identifier Logic

- **GIVEN** A PostgreSQL dialect
- **WHEN** quote_identifier("public.users") is called
- **THEN** The output is correctly quoted as "public"."users".

## Diagrams

### Dialect and Database Abstraction Architecture

```mermaid
flowchart TB
    Connection[Connection]
    DatabasePool[DatabasePool (Trait)]
    PostgresPool[PostgresPool (Wrapper)]
    SqlitePool[SqlitePool (Wrapper)]
    MysqlPool[MysqlPool (Wrapper)]
    QueryBuilder[QueryBuilder]
    Dialect[Dialect (Trait)]
    PostgresDialect[PostgresDialect (Impl)]
    SqliteDialect[SqliteDialect (Impl)]
    MysqlDialect[MysqlDialect (Impl)]
    Connection --> DatabasePool
    QueryBuilder --> Dialect
    DatabasePool --> PostgresPool
    DatabasePool --> SqlitePool
    DatabasePool --> MysqlPool
    Dialect --> PostgresDialect
    Dialect --> SqliteDialect
    Dialect --> MysqlDialect
```

</spec>
