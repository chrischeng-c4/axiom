---
id: projects-jet-logic-jet-build-lib-library-build-mode-esm-cjs-externalize-deps-peerde-md
fill_sections: [logic]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "jet build --lib is the library build mode that emits externalized, multi-entry ESM/CJS — the foundation of the library-build-publishing capability."
---

# jet build --lib: Library Build Mode

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-build-lib-flow
entry: parse_inv
nodes:
  parse_inv: { kind: start,    label: "jet build --lib invoked" }
  load_cfg:  { kind: process,  label: "merge --lib/--format/--out-dir flags + jet.toml [lib]" }
  is_lib:    { kind: decision, label: "lib mode requested?" }
  app_mode:  { kind: terminal, label: "existing app build path (byte-stable, unchanged)" }
  read_pkg:  { kind: process,  label: "read package.json: exports/module/main + dependencies+peerDependencies" }
  entries:   { kind: decision, label: ">=1 library entry resolved?" }
  no_entry:  { kind: terminal, label: "error: no library entry found" }
  externals: { kind: process,  label: "externals = deps + peerDeps -> resolver externalize" }
  build:     { kind: process,  label: "build module graph per entry, tree-shake" }
  fmt:       { kind: decision, label: "emit format" }
  emit_esm:  { kind: process,  label: "emit ESM: external specifiers kept as bare import" }
  emit_cjs:  { kind: process,  label: "emit CJS: external specifiers kept as require()" }
  write:     { kind: process,  label: "write one output file per (entry x format) under out_dir" }
  result:    { kind: terminal, label: "LibBuildResult { entries: Vec<EntryOutput> } returned" }
edges:
  - { from: parse_inv, to: load_cfg }
  - { from: load_cfg, to: is_lib }
  - { from: is_lib,    to: app_mode,  label: "no" }
  - { from: is_lib,    to: read_pkg,  label: "yes" }
  - { from: read_pkg,  to: entries }
  - { from: entries,   to: no_entry,  label: "no" }
  - { from: entries,   to: externals, label: "yes" }
  - { from: externals, to: build }
  - { from: build,     to: fmt }
  - { from: fmt,       to: emit_esm,  label: "esm" }
  - { from: fmt,       to: emit_cjs,  label: "cjs" }
  - { from: emit_esm,  to: write }
  - { from: emit_cjs,  to: write }
  - { from: write,     to: result }
---
flowchart TD
    parse_inv([jet build --lib invoked]) --> load_cfg[merge flags + jet.toml lib]
    load_cfg --> is_lib{lib mode requested?}
    is_lib -->|no| app_mode([existing app build, byte-stable])
    is_lib -->|yes| read_pkg[read package.json exports + deps/peerDeps]
    read_pkg --> entries{>=1 library entry?}
    entries -->|no| no_entry([error: no library entry])
    entries -->|yes| externals[externals = deps + peerDeps]
    externals --> build[build graph per entry, tree-shake]
    build --> fmt{emit format}
    fmt -->|esm| emit_esm[emit ESM, external bare import]
    fmt -->|cjs| emit_cjs[emit CJS, external require]
    emit_esm --> write[write one file per entry x format]
    emit_cjs --> write
    write --> result([LibBuildResult entries returned])
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic is a valid Mermaid Plus block (id jet-build-lib-flow). The flow is complete and deterministic: parse the `jet build --lib` invocation, merge `--lib`/`--format`/`--out-dir` flags with `jet.toml [lib]`, branch on lib-mode (absent -> existing app build path stays byte-stable; present -> proceed). In lib mode it reads package.json (entries from exports/module/main; externals from dependencies+peerDependencies), validates >=1 entry (else a terminal no-entry error), externalizes deps via the resolver, builds+tree-shakes per entry, emits per format (ESM keeps external specifiers as bare `import`, CJS as `require()`), writes one output file per (entry x format) under out_dir, and returns LibBuildResult. Every node is reachable, both decisions (is_lib, entries) and the format branch carry labeled edges, and all three terminals (app_mode, no_entry, result) are real ends. Scope is correct: `.d.ts` emission (A2 #171) and publish/registry (A3 #172) are downstream and out of scope here.
