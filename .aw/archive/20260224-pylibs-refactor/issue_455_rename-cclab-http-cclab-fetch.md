---
number: 455
title: "Rename cclab-http → cclab-fetch"
state: open
labels: [enhancement, crate:http, P1, crate:fetch]
---

# #455 — Rename cclab-http → cclab-fetch

## Summary

Rename `cclab-http` to `cclab-fetch` to avoid confusion with Python's stdlib `http` module.

`cclab.http` as a Python module name conflicts with `import http` (stdlib). The name `fetch` is modern, inspired by the JS Fetch API, and clearly conveys HTTP client functionality.

## Scope

### Rust side
- [ ] `git mv crates/cclab-http crates/cclab-fetch`
- [ ] Update `Cargo.toml`: package name `cclab-fetch`
- [ ] Update root `Cargo.toml`: workspace member
- [ ] Update all downstream `Cargo.toml` deps (`cclab-agent`, `cclab-nucleus`, `cclab-cli`, etc.)
- [ ] Replace all `use cclab_http::` → `use cclab_fetch::` across workspace
- [ ] Update `cclab-nucleus` feature name: `http` → `fetch`

### Python side
- [ ] `git mv python/cclab/http python/cclab/fetch`
- [ ] Update `python/cclab/__init__.py`: `from . import fetch`
- [ ] Update all Python imports: `from cclab.http` → `from cclab.fetch`

### Docs
- [ ] README.md module table
- [ ] CLAUDE.md references
- [ ] GitHub label: `crate:http` → `crate:fetch`
