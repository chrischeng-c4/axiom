---
title: data-bridge Documentation
status: implemented
component: general
type: index
---

# data-bridge Documentation

Welcome to the documentation for **data-bridge**.

## Mission
<!-- type: overview lang: markdown -->

**data-bridge** is a collection of high-performance Rust solutions for Python scenarios.
The goal is to provide consistent, safe, and extremely fast implementations for common Python bottlenecks by offloading work to Rust.

## Solutions
<!-- type: doc lang: markdown -->

### [1. MongoDB Solution](../cclab-titan/README.md)
A high-performance ORM/ODM that replaces Beanie/Motor.
- **Components**: [Rust Core Engine](../cclab-titan/01-core-engine/README.md), [Python API](../cclab-titan/02-python-api/README.md)
- **Key Features**: Zero-copy BSON, Rayon parallelism, Connection pooling.

### [2. HTTP Solution](../cclab-ion/README.md)
A high-performance async HTTP client.
- **Replaces**: `httpx`, `aiohttp` (for specific use cases)
- **Key Features**: GIL-free request processing, Error sanitization.

### [3. Test Runner](../cclab-probe/README.md)
A specialized test runner for mixed Rust/Python projects.
- **Replaces**: `pytest` (for discovery/execution, not assertions)
- **Key Features**: Parallel test execution, Fast discovery via Rust `walkdir`.

### [4. PostgreSQL Solution](../cclab-orbit/README.md)
A high-performance async PostgreSQL ORM with Rust backend.
- **Components**: [Rust Core Engine](../cclab-orbit/01-core-engine/README.md), [Python API](../cclab-orbit/02-python-api/README.md)
- **Key Features**: CRUD operations, Transactions, Migrations, Connection pooling.
- **Driver**: sqlx with compile-time query validation.

### Future Solutions
- **Redis**: Async cache/queue interface.
- **MySQL**: Async driver.

## Getting Started
<!-- type: doc lang: markdown -->

### Architecture
- [Architecture Principles](./logic/architecture/principles.md)
- [Class Diagram](./logic/architecture/class-diagram.md)
- [Structured Error Handling](./interfaces/error/structured-handling.md)
- [Core Safety Standards](./logic/safety/core-safety-standards.md)
- [Roadmap](./logic/roadmap/feature-roadmap.md)

### Development
- See `GEMINI.md` in the root directory for the active development context and workflows.
- [Global TODOs](./logic/roadmap/global-todos.md)
