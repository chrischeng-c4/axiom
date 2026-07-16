# Dependency Graph: Analyzed Project

## Overview

- **Internal Modules**: 8
- **External Dependencies**: 10
- **Total Relationships**: 22

## Module Graph

```mermaid
flowchart TD
    subgraph Internal["Internal Modules"]
        llm["llm\n(2 symbols, 2 public)"]
        courier["courier\n(16 symbols, 0 public)"]
        lib["lib\n(1 symbols, 1 public)"]
        openapi["openapi\n(3 symbols, 2 public)"]
        auth["auth\n(8 symbols, 8 public)"]
        mod["mod\n(11 symbols, 9 public)"]
        routes["routes\n(10 symbols, 5 public)"]
        github["github\n(15 symbols, 10 public)"]
    end
    subgraph External["External Dependencies"]
        clap[/"clap"/]
        service_http[/"service_http"/]
        anyhow[/"anyhow"/]
        axum[/"axum"/]
        std[/"std"/]
        serde_json[/"serde_json"/]
        utoipa[/"utoipa"/]
        reqwest[/"reqwest"/]
        service_auth[/"service_auth"/]
        serde[/"serde"/]
    end
    courier --> anyhow
    courier --> clap
    courier --> courier
    courier --> llm
    openapi --> utoipa
    auth --> std
    auth --> anyhow
    auth --> service_auth
    mod --> std
    mod --> axum
    mod --> service_auth
    mod --> service_http
    mod --> http
    routes --> axum
    routes --> serde
    routes --> serde_json
    routes --> service_auth
    routes --> service_http
    routes --> http
    github --> anyhow
    github --> reqwest
    github --> serde_json
```

## Internal Modules

| Module | Path | Symbols | Public |
|--------|------|---------|--------|
| llm | apps/courier/src/llm.rs | 2 | 2 |
| courier | apps/courier/src/bin/courier.rs | 16 | 0 |
| lib | apps/courier/src/lib.rs | 1 | 1 |
| openapi | apps/courier/src/http/openapi.rs | 3 | 2 |
| auth | apps/courier/src/http/auth.rs | 8 | 8 |
| mod | apps/courier/src/http/mod.rs | 11 | 9 |
| routes | apps/courier/src/http/routes.rs | 10 | 5 |
| github | apps/courier/src/http/github.rs | 15 | 10 |

## External Dependencies

| Dependency |
|------------|
| clap |
| service_http |
| anyhow |
| axum |
| std |
| serde_json |
| utoipa |
| reqwest |
| service_auth |
| serde |

## Dependency Details

| From | To | Type |
|------|-----|------|
| courier | anyhow | import |
| courier | clap | import |
| courier | courier | import |
| courier | llm | import |
| openapi | utoipa | import |
| auth | std | import |
| auth | anyhow | import |
| auth | service_auth | import |
| mod | std | import |
| mod | axum | import |
| mod | service_auth | import |
| mod | service_http | import |
| mod | http | import |
| routes | axum | import |
| routes | serde | import |
| routes | serde_json | import |
| routes | service_auth | import |
| routes | service_http | import |
| routes | http | import |
| github | anyhow | import |
| github | reqwest | import |
| github | serde_json | import |
