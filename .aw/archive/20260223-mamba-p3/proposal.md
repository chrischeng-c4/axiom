---
id: mamba-p3
type: proposal
version: 2
created_at: 2026-02-23T01:15:07.780372+00:00
updated_at: 2026-02-23T01:15:07.780372+00:00
iteration: 1
scope: minor
spec_plan:
  - id: mamba-stdlib-subprocess
    title: "subprocess module — external command execution"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/subprocess_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: mamba-stdlib-csv
    title: "csv module — CSV file reading/writing"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/csv_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-argparse
    title: "argparse module — CLI argument parsing"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/argparse_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-logging
    title: "logging module — structured log output"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/logging_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-typing
    title: "typing module — runtime type construct sentinels"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-type-system"]
    affected_code: ["crates/mamba/src/runtime/stdlib/typing_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-threading
    title: "threading module — Thread, Lock, Event"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/class.rs"]
      spec: ["mamba-oop-model"]
    affected_code: ["crates/mamba/src/runtime/stdlib/threading_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-socket-http
    title: "socket and http/urllib modules — networking"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/socket_mod.rs", "crates/mamba/src/runtime/stdlib/http_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-unittest
    title: "unittest module — test framework"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/class.rs"]
      spec: ["mamba-oop-model"]
    affected_code: ["crates/mamba/src/runtime/stdlib/unittest_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-builtin-eval-exec
    title: "eval/exec/compile/globals/locals builtins"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/builtins.rs", "crates/mamba/src/parser/"]
    affected_code: ["crates/mamba/src/runtime/builtins.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: mamba-stdlib-pickle
    title: "pickle module — object serialization"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/rc.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/pickle_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-sqlite3
    title: "sqlite3 module — database interface"
    depends: []
    context_refs:
      codebase: ["crates/mamba/Cargo.toml"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/sqlite3_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs", "crates/mamba/Cargo.toml"]
  - id: mamba-stdlib-compression
    title: "gzip, zipfile, tarfile modules — compression/archives"
    depends: []
    context_refs:
      codebase: ["crates/mamba/Cargo.toml"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/gzip_mod.rs", "crates/mamba/src/runtime/stdlib/zipfile_mod.rs", "crates/mamba/src/runtime/stdlib/tarfile_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs", "crates/mamba/Cargo.toml"]
  - id: mamba-stdlib-text-utils
    title: "pprint, textwrap, string modules — text utilities"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/string_ops.rs"]
      spec: ["mamba-string-runtime"]
    affected_code: ["crates/mamba/src/runtime/stdlib/pprint_mod.rs", "crates/mamba/src/runtime/stdlib/textwrap_mod.rs", "crates/mamba/src/runtime/stdlib/string_constants_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-xml-html
    title: "xml.etree.ElementTree and html.parser modules"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/stdlib/mod.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/xml_mod.rs", "crates/mamba/src/runtime/stdlib/html_parser_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-stdlib-array
    title: "array module — efficient typed arrays"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/rc.rs"]
      spec: ["mamba-stdlib-core"]
    affected_code: ["crates/mamba/src/runtime/stdlib/array_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
  - id: mamba-complex-ops
    title: "Complex number full operations and cmath module"
    depends: []
    context_refs:
      codebase: ["crates/mamba/src/runtime/rc.rs", "crates/mamba/src/parser/ast.rs"]
      spec: ["mamba-gc-runtime"]
    affected_code: ["crates/mamba/src/runtime/rc.rs", "crates/mamba/src/runtime/string_ops.rs", "crates/mamba/src/runtime/class.rs", "crates/mamba/src/runtime/gc.rs", "crates/mamba/src/runtime/builtins.rs", "crates/mamba/src/runtime/stdlib/cmath_mod.rs", "crates/mamba/src/runtime/stdlib/mod.rs"]
history:
  - timestamp: 2026-02-23T01:15:07.780372+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: mamba-p3

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((mamba-p3))  
    System & CLI Modules
      subprocess (#397)
      argparse (#399)
      logging (#400)
    Data Formats
      csv (#398)
      pickle (#442)
      xml/html (#449)
    Networking & I/O
      socket/http (#418)
      sqlite3 (#444)
      compression (#445)
    Concurrency
      threading (#417)
    Type System & Testing
      typing (#401)
      unittest (#419)
    Text Utilities
      pprint (#446)
      textwrap (#448)
      string (#452)
    Numeric & Data
      complex (#453)
      array (#451)
    Dynamic Execution
      eval/exec (#441)
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  mamba_stdlib_subprocess["mamba-stdlib-subprocess\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_csv["mamba-stdlib-csv\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_argparse["mamba-stdlib-argparse\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_logging["mamba-stdlib-logging\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_typing["mamba-stdlib-typing\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_threading["mamba-stdlib-threading\n codebase: crates/mamba/src/runtime/class.rs"]
  mamba_stdlib_socket_http["mamba-stdlib-socket-http\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_unittest["mamba-stdlib-unittest\n codebase: crates/mamba/src/runtime/class.rs"]
  mamba_builtin_eval_exec["mamba-builtin-eval-exec\n codebase: crates/mamba/src/runtime/builtins.rs, crates/mamba/src/parser/"]
  mamba_stdlib_pickle["mamba-stdlib-pickle\n codebase: crates/mamba/src/runtime/rc.rs"]
  mamba_stdlib_sqlite3["mamba-stdlib-sqlite3\n codebase: crates/mamba/Cargo.toml"]
  mamba_stdlib_compression["mamba-stdlib-compression\n codebase: crates/mamba/Cargo.toml"]
  mamba_stdlib_text_utils["mamba-stdlib-text-utils\n codebase: crates/mamba/src/runtime/string_ops.rs"]
  mamba_stdlib_xml_html["mamba-stdlib-xml-html\n codebase: crates/mamba/src/runtime/stdlib/mod.rs"]
  mamba_stdlib_array["mamba-stdlib-array\n codebase: crates/mamba/src/runtime/rc.rs"]
  mamba_complex_ops["mamba-complex-ops\n codebase: crates/mamba/src/runtime/rc.rs, crates/mamba/src/parser/ast.rs"]

```

## Spec Execution Order

1. **mamba-builtin-eval-exec** — eval/exec/compile/globals/locals builtins
   - code: crates/mamba/src/runtime/builtins.rs, crates/mamba/src/runtime/symbols.rs
2. **mamba-complex-ops** — Complex number full operations and cmath module
   - code: crates/mamba/src/runtime/rc.rs, crates/mamba/src/runtime/string_ops.rs, crates/mamba/src/runtime/class.rs, crates/mamba/src/runtime/gc.rs, crates/mamba/src/runtime/builtins.rs, crates/mamba/src/runtime/stdlib/cmath_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
3. **mamba-stdlib-argparse** — argparse module — CLI argument parsing
   - code: crates/mamba/src/runtime/stdlib/argparse_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
4. **mamba-stdlib-array** — array module — efficient typed arrays
   - code: crates/mamba/src/runtime/stdlib/array_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
5. **mamba-stdlib-compression** — gzip, zipfile, tarfile modules — compression/archives
   - code: crates/mamba/src/runtime/stdlib/gzip_mod.rs, crates/mamba/src/runtime/stdlib/zipfile_mod.rs, crates/mamba/src/runtime/stdlib/tarfile_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs, crates/mamba/Cargo.toml
6. **mamba-stdlib-csv** — csv module — CSV file reading/writing
   - code: crates/mamba/src/runtime/stdlib/csv_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
7. **mamba-stdlib-logging** — logging module — structured log output
   - code: crates/mamba/src/runtime/stdlib/logging_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
8. **mamba-stdlib-pickle** — pickle module — object serialization
   - code: crates/mamba/src/runtime/stdlib/pickle_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
9. **mamba-stdlib-socket-http** — socket and http/urllib modules — networking
   - code: crates/mamba/src/runtime/stdlib/socket_mod.rs, crates/mamba/src/runtime/stdlib/http_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
10. **mamba-stdlib-sqlite3** — sqlite3 module — database interface
   - code: crates/mamba/src/runtime/stdlib/sqlite3_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs, crates/mamba/Cargo.toml
11. **mamba-stdlib-subprocess** — subprocess module — external command execution
   - code: crates/mamba/src/runtime/stdlib/subprocess_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs, crates/mamba/src/runtime/symbols.rs
12. **mamba-stdlib-text-utils** — pprint, textwrap, string modules — text utilities
   - code: crates/mamba/src/runtime/stdlib/pprint_mod.rs, crates/mamba/src/runtime/stdlib/textwrap_mod.rs, crates/mamba/src/runtime/stdlib/string_constants_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
13. **mamba-stdlib-threading** — threading module — Thread, Lock, Event
   - code: crates/mamba/src/runtime/stdlib/threading_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
14. **mamba-stdlib-typing** — typing module — runtime type construct sentinels
   - code: crates/mamba/src/runtime/stdlib/typing_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
15. **mamba-stdlib-unittest** — unittest module — test framework
   - code: crates/mamba/src/runtime/stdlib/unittest_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs
16. **mamba-stdlib-xml-html** — xml.etree.ElementTree and html.parser modules
   - code: crates/mamba/src/runtime/stdlib/xml_mod.rs, crates/mamba/src/runtime/stdlib/html_parser_mod.rs, crates/mamba/src/runtime/stdlib/mod.rs

</proposal>
