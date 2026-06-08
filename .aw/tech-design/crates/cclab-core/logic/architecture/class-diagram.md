---
id: core-class-diagram
type: spec
title: "Core Utilities Class Diagram"
version: 1
spec_type: utility
created_at: 2026-01-31T12:45:00+00:00
updated_at: 2026-01-31T12:45:00+00:00
main_spec_ref: "cclab-core/logic/architecture/class-diagram.md"
fill_sections: [overview, dependency, changes]
design_elements:
  has_mermaid: true
  diagrams:
    - type: class
      title: "Core Types and Traits"
---

# Core Utilities Architecture

## Overview
<!-- type: overview lang: markdown -->

cclab-core provides shared utilities, types, and abstractions used across all
crates.

## Core Types
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cclab-core-core-types
title: Core Types
---
classDiagram
    class Result~T, E~ {
        <<enum>>
        Ok(T)
        Err(E)
    }

    class Error {
        +kind: ErrorKind
        +message: String
        +source: Option~Box~dyn Error~~
        +context() String
        +chain() Iterator
    }

    class ErrorKind {
        <<enum>>
        Io
        Parse
        Validation
        Network
        Database
        Internal
    }

    class Config {
        +load(path: Path) Config
        +get~T~(key: str) Option~T~
        +set~T~(key: str, value: T)
    }

    class Context {
        +config: Config
        +logger: Logger
        +metrics: Metrics
    }

    Error --> ErrorKind
    Result --> Error
    Context --> Config
```

## Async Traits
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cclab-core-async-traits
title: Async Traits
---
classDiagram
    class AsyncRead {
        <<trait>>
        +poll_read(buf) Poll~Result~usize~~
    }

    class AsyncWrite {
        <<trait>>
        +poll_write(buf) Poll~Result~usize~~
        +poll_flush() Poll~Result~()~~
    }

    class AsyncStream {
        <<trait>>
    }

    class Connection {
        <<trait>>
        +connect(addr) Future~Self~
        +close() Future~()~
    }

    AsyncStream --|> AsyncRead
    AsyncStream --|> AsyncWrite
    Connection --|> AsyncStream
```

## Serialization
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cclab-core-serialization
title: Serialization
---
classDiagram
    class Serialize {
        <<trait>>
        +serialize~S~(serializer: S) Result
    }

    class Deserialize {
        <<trait>>
        +deserialize~D~(deserializer: D) Result~Self~
    }

    class JsonValue {
        <<enum>>
        Null
        Bool(bool)
        Number(Number)
        String(String)
        Array(Vec)
        Object(Map)
    }

    class Codec {
        +encode~T~(value: T) Vec~u8~
        +decode~T~(bytes: bytes) Result~T~
    }

    Codec --> Serialize
    Codec --> Deserialize
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/crates/cclab-core/logic/architecture/class-diagram.md
    action: modify
    section: dependency
    impl_mode: hand-written
    description: "Maintain high-level cclab-core utility architecture diagrams."
  - path: .aw/tech-design/crates/cclab-core/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: "Link to the normalized class diagram spec."
```
