# FocusFlow Todo capabilities

## Brief

FocusFlow is a small full-stack Todo application used to dogfood a Python EC
and TD contract against a Rust (Axum + SQLite) codebase.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Todo HTTP API | — | Rust Axum + SQLite | EC behavior/security/stability/efficiency | dogfood | no | CRUD API and static web UI |

### Todo HTTP API

ID: todo-http-api

Type: DeveloperTool

Promise: Create, list, update, and delete persisted Todo records through a
stable browser-facing HTTP API.

Required Verification: Python EC case inventory in `external-contracts/`.
