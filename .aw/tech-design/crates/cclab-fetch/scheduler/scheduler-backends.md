---
id: scheduler-backends-gcp
main_spec_ref: "crates/cclab-fetch/scheduler/scheduler-backends.md"
merge_strategy: new
---

# Scheduler Backends Gcp

## Overview

Extends the scheduler-backends module to register `CloudSchedulerBackend` as a third backend variant alongside `IonSchedulerBackend` (production, distributed locking via cclab-ion) and `MemorySchedulerBackend` (testing). The Cloud Scheduler backend offloads cron/interval scheduling to GCP Cloud Scheduler service, eliminating the self-hosted leader election loop. Module registration adds `#[cfg(feature = "cloud-scheduler")] pub mod cloud_scheduler_backend` and corresponding re-exports to `scheduler/mod.rs`, following the same conditional compilation pattern as the existing `#[cfg(feature = "scheduler")] pub mod ion_backend`. The existing `SchedulerBackend` trait (R1), Ion backend (R2), and InMemory backend (R3) are unchanged — this change adds R4 for the Cloud Scheduler variant. Implementation details of `CloudSchedulerBackend` are specified in the companion `cloud-scheduler-backend` change spec.

Source: `crates/cclab-queue/src/scheduler/mod.rs` (module registration), `crates/cclab-queue/src/scheduler/backend.rs` (SchedulerBackend trait)
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R4 | Cloud Scheduler Backend registration | P0 | `scheduler/mod.rs` declares `#[cfg(feature = "cloud-scheduler")] pub mod cloud_scheduler_backend` and re-exports `CloudSchedulerBackend`, `CloudSchedulerConfig`. Module is only compiled when `cloud-scheduler` Cargo feature is enabled. Follows the same pattern as `#[cfg(feature = "scheduler")] pub mod ion_backend` |
| R5 | SchedulerBackend trait compatibility | P0 | `CloudSchedulerBackend` implements the existing `SchedulerBackend` trait defined in `backend.rs` without modifications to the trait itself. All methods (`acquire_leader`, `renew_leader`, `release_leader`, `get_task_state`, `set_task_state`) are implemented. No changes to `TaskScheduleState` struct |
| R6 | Feature isolation | P1 | Enabling `cloud-scheduler` feature does not affect `scheduler` (Ion) or default (InMemory) backends. All three backends can coexist when their respective features are enabled. No GCP dependencies are pulled unless `cloud-scheduler` feature is active |

### Constraints

- Existing R1 (SchedulerBackend trait), R2 (Ion backend), R3 (InMemory backend) are unchanged
- The `cloud-scheduler` feature is independent of the `scheduler` feature (Ion)
- `CloudSchedulerBackend` module file is `cloud_scheduler_backend.rs` (snake_case per Rust convention)
## Scenarios

### S1: Cloud Scheduler backend is available when feature is enabled (R4, R6)

**GIVEN** a Cargo.toml with `features = ["cloud-scheduler"]` enabled
**WHEN** the crate is compiled
**THEN** `CloudSchedulerBackend` and `CloudSchedulerConfig` are importable from `cclab_queue::scheduler`; the `cloud_scheduler_backend` module is included in compilation

### S2: Cloud Scheduler backend is excluded when feature is disabled (R4, R6)

**GIVEN** a Cargo.toml without `cloud-scheduler` feature
**WHEN** the crate is compiled
**THEN** `CloudSchedulerBackend` and `CloudSchedulerConfig` are not available; no GCP-related dependencies (reqwest, base64) are pulled; existing Ion and InMemory backends compile normally

### S3: Cloud Scheduler backend implements SchedulerBackend trait (R5)

**GIVEN** a `CloudSchedulerBackend` instance
**WHEN** used as `Box<dyn SchedulerBackend>` in `PeriodicScheduler`
**THEN** all trait methods are callable: `acquire_leader(ttl)`, `renew_leader(ttl)`, `release_leader()`, `get_task_state(name)`, `set_task_state(name, state)`; the scheduler operates without changes to its scheduling loop

### S4: All three backends coexist (R6)

**GIVEN** a Cargo.toml with both `scheduler` and `cloud-scheduler` features enabled
**WHEN** the crate is compiled
**THEN** `IonSchedulerBackend`, `MemorySchedulerBackend`, and `CloudSchedulerBackend` are all available; application can select backend at runtime based on configuration
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
changes:
  - file: crates/cclab-queue/src/scheduler/mod.rs
    action: modify
    description: |
      Add conditional module declaration and re-export for cloud_scheduler_backend.
      Follows existing pattern of #[cfg(feature = "scheduler")] pub mod ion_backend.
    additions:
      - '#[cfg(feature = "cloud-scheduler")] pub mod cloud_scheduler_backend;'
      - '#[cfg(feature = "cloud-scheduler")] pub use cloud_scheduler_backend::{CloudSchedulerBackend, CloudSchedulerConfig};'
    location: |
      Module declaration after line 11 (after ion_backend declaration).
      Re-export after line 18 (after IonSchedulerBackend re-export).

  - file: crates/cclab-queue/Cargo.toml
    action: modify
    description: |
      Add cloud-scheduler feature flag and ensure conditional dependencies are activated.
    additions:
      - 'cloud-scheduler feature: ["dep:reqwest", "dep:base64"]'
    notes: |
      reqwest and base64 may already be optional dependencies for the cloudtasks feature.
      If so, cloud-scheduler should share these deps (both features activate the same optional deps).
      Do NOT duplicate dependency entries — add cloud-scheduler to existing activation lists.
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->