---
id: improve-quasar-maturity
type: exploration
created_at: 2026-01-28T07:23:24.363121+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: Improve Quasar Maturity

## Architecture Overview
`cclab-quasar` is a high-performance API framework built on Hyper 1.0 and Tokio. It follows a two-phase GIL pattern to maximize concurrency for Python handlers while using Rust for routing, validation, and serialization.

The core components include:
- `Router`: Uses `matchit` for radix-tree based routing.
- `Server`: Wraps the router and handles HTTP/1.1 connections via Hyper.
- `DependencyContainer`: Provides a DI system with topological resolution.
- `LifecycleManager`: Manages startup and shutdown hooks.
- `OpenApiSpec`: Generates OpenAPI 3.1 documentation.

## Relevant Files
- `crates/cclab-quasar/src/dependency.rs`: DI implementation.
- `crates/cclab-quasar/src/openapi.rs`: OpenAPI spec generation.
- `crates/cclab-quasar/src/server.rs`: HTTP server implementation.
- `crates/cclab-quasar/src/lifecycle.rs`: Lifecycle hooks.
- `crates/cclab-quasar/src/handler.rs` & `router.rs`: Handler definitions and routing.

## Impact Analysis
- **Dependency Injection**: Updating the handler system to support auto-resolution will affect how routes are registered and how handlers are invoked. This is a significant but necessary change for developer experience.
- **Interactive Docs**: Adding documentation routes is low impact but provides high value. It will involve adding static asset serving or embedding the UI files.
- **Lifespan Events**: Integration with `Server` will ensure better reliability of startup/shutdown logic.
- **Test Client**: New utility for testing, primarily affecting `dev-dependencies` and providing a new API for tests.

## Technical Considerations
- **DI Resolution**: We should leverage the existing `DependencyContainer` but extend the `HandlerFn` to handle resolution. Alternatively, we can use a trait-based approach for extractors that includes DI.
- **Interactive Docs**: We can use `utoipa-swagger-ui` or similar crates, or manually serve the static files for Swagger/ReDoc.
- **Test Client**: Should support both sync and async testing, and ideally allow testing without binding to a real TCP port.

## Recommendation
Proceed with creating a proposal that focuses on integrating the DI system with handlers, adding documentation UIs, and providing a robust test client. The proposal should also include tasks for the missing tests and documentation guides.
