---
id: cclab-grid-db-spec-index
main_spec_ref: "cclab-grid-db/README.md"
fill_sections: [overview, doc]
---

# cclab-grid-db Specs

## Overview
<!-- type: overview lang: markdown -->

`cclab-grid-db` tracks the database layer inside `crates/cclab-grid/src/db`.
The layer stores spreadsheet cells with Morton-encoded keys, protects writes
with the shared WAL crate, supports rectangular range scans, and persists Yrs
updates and snapshots for collaborative editing.

## Specs
<!-- type: doc lang: markdown -->

- [Grid DB Architecture](./logic/architecture/grid-db-architecture.md)
