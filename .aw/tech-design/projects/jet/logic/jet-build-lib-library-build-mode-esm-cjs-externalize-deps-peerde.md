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
initial: ParseLibInvocation
---
stateDiagram-v2
    [*] --> ParseLibInvocation : jet build --lib
    ParseLibInvocation --> LoadLibConfig : merge --lib flags + jet.toml [lib]
    LoadLibConfig --> ReadPackageJson : resolve formats, out_dir, preserve_modules
    ReadPackageJson --> DiscoverEntries : read exports / module / main
    ReadPackageJson --> CollectExternals : read dependencies + peerDependencies
    DiscoverEntries --> BuildPerEntry : entries resolved (>=1)
    CollectExternals --> BuildPerEntry : externals set = deps + peerDeps
    BuildPerEntry --> EmitEsm : format includes esm
    BuildPerEntry --> EmitCjs : format includes cjs
    EmitEsm --> PreserveExternalImports : external specifiers kept as bare import
    EmitCjs --> PreserveExternalRequires : external specifiers kept as require()
    PreserveExternalImports --> WriteOutputs
    PreserveExternalRequires --> WriteOutputs
    WriteOutputs --> RecordMetadata : one output file per entry x format
    RecordMetadata --> [*] : resolved entry to output map returned
    LoadLibConfig --> AppModeUnchanged : --lib absent
    AppModeUnchanged --> [*] : existing app build path stays byte-stable
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] The logic state diagram correctly scopes the library build flow: config merge, package.json read (entries from exports/module/main, externals from dependencies+peerDependencies), per-entry build, ESM/CJS emission preserving external imports/requires, and the `--lib`-absent branch that leaves the existing app build byte-stable. Applicability is sound for this TD.
