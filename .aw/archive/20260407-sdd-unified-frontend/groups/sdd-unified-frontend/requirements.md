---
change: sdd-unified-frontend
group: sdd-unified-frontend
date: 2026-04-07
---

# Requirements

Unify Score viewer and Conductor frontend into a single React app under crates/sdd/packages/@score/. (1) Move @cclab/{ui,spec-viewer,pipeline} → crates/sdd/packages/@score/{ui,spec-viewer,pipeline}, rename to @score/*. (2) Create @score/core with SddDataSource adapter interface (LocalDataSource for filesystem, RemoteDataSource for REST API). (3) Create @score/app as unified React SPA with React Router, DataSource via Context. (4) Replace crates/sdd/src/ui/viewer/assets/ vanilla JS with @score/app build output, keep axum server + include_str!() embedding. (5) Update Conductor FE to import @score/app as sub-routes.
