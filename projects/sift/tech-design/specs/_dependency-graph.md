# Dependency Graph: Analyzed Project

## Overview

- **Internal Modules**: 2
- **External Dependencies**: 8
- **Total Relationships**: 13

## Module Graph

```mermaid
flowchart TD
    subgraph Internal["Internal Modules"]
        sift["sift\n(24 symbols, 0 public)"]
        lib["lib\n(40 symbols, 22 public)"]
    end
    subgraph External["External Dependencies"]
        anyhow[/"anyhow"/]
        utoipa[/"utoipa"/]
        std[/"std"/]
        chrono[/"chrono"/]
        serde[/"serde"/]
        serde_json[/"serde_json"/]
        clap[/"clap"/]
        axum[/"axum"/]
    end
    sift --> std
    sift --> anyhow
    sift --> axum
    sift --> clap
    sift --> sift
    lib --> std
    lib --> anyhow
    lib --> axum
    lib --> chrono
    lib --> clap
    lib --> serde
    lib --> serde_json
    lib --> utoipa
```

## Internal Modules

| Module | Path | Symbols | Public |
|--------|------|---------|--------|
| sift | projects/sift/src/bin/sift.rs | 24 | 0 |
| lib | projects/sift/src/lib.rs | 40 | 22 |

## External Dependencies

| Dependency |
|------------|
| anyhow |
| utoipa |
| std |
| chrono |
| serde |
| serde_json |
| clap |
| axum |

## Dependency Details

| From | To | Type |
|------|-----|------|
| sift | std | import |
| sift | anyhow | import |
| sift | axum | import |
| sift | clap | import |
| sift | sift | import |
| lib | std | import |
| lib | anyhow | import |
| lib | axum | import |
| lib | chrono | import |
| lib | clap | import |
| lib | serde | import |
| lib | serde_json | import |
| lib | utoipa | import |
