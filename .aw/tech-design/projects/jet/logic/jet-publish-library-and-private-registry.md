---
id: projects-jet-logic-jet-publish-library-and-private-registry-md
fill_sections: [logic]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: publish-and-private-registry
    coverage: partial
    rationale: "jet publish builds the library, validates package metadata, and publishes to public or private registries — the publish leg of library-build-publishing."
---

# jet publish: Build, Validate, and Publish to Public/Private Registries

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-publish-lib-flow
entry: invoke
nodes:
  invoke:    { kind: start,    label: "jet publish [--build] or jet pack" }
  build_q:   { kind: decision, label: "--build or [lib] config present?" }
  build_lib: { kind: process,  label: "run build_library -> dist outputs (A1/A2)" }
  read_pkg:  { kind: process,  label: "read + transform package.json (workspace:* -> versions)" }
  validate:  { kind: process,  label: "validate main/module/exports/types resolve to real files; auto-fill from build output" }
  val_ok:    { kind: decision, label: "all metadata paths exist?" }
  val_err:   { kind: terminal, label: "error: metadata points at missing file" }
  identity:  { kind: process,  label: "require name + version (publish identity)" }
  registry:  { kind: process,  label: "resolve registry via .npmrc scope + auth token" }
  pack:      { kind: process,  label: "create tarball: built files + transformed package.json" }
  pack_only: { kind: decision, label: "publish or pack-only?" }
  write_tgz: { kind: terminal, label: "pack: write <name>-<version>.tgz" }
  put:       { kind: process,  label: "PUT base64 tarball to registry with Bearer auth" }
  done:      { kind: terminal, label: "published <name>@<version> to <registry>" }
edges:
  - { from: invoke,    to: build_q }
  - { from: build_q,   to: build_lib, label: "yes" }
  - { from: build_q,   to: read_pkg,  label: "no" }
  - { from: build_lib, to: read_pkg }
  - { from: read_pkg,  to: validate }
  - { from: validate,  to: val_ok }
  - { from: val_ok,    to: val_err,   label: "no" }
  - { from: val_ok,    to: identity,  label: "yes" }
  - { from: identity,  to: registry }
  - { from: registry,  to: pack }
  - { from: pack,      to: pack_only }
  - { from: pack_only, to: write_tgz, label: "pack" }
  - { from: pack_only, to: put,       label: "publish" }
  - { from: put,       to: done }
---
flowchart TD
    invoke([jet publish / pack]) --> build_q{--build or lib config?}
    build_q -->|yes| build_lib[build_library dist outputs]
    build_q -->|no| read_pkg[read + transform package.json]
    build_lib --> read_pkg
    read_pkg --> validate[validate + auto-fill main/module/exports/types]
    validate --> val_ok{all paths exist?}
    val_ok -->|no| val_err([error: missing file])
    val_ok -->|yes| identity[require name + version]
    identity --> registry[resolve registry via .npmrc + auth]
    registry --> pack[create tarball]
    pack --> pack_only{publish or pack?}
    pack_only -->|pack| write_tgz([write name-version.tgz])
    pack_only -->|publish| put[PUT base64 tarball, Bearer auth]
    put --> done([published to registry])
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract logic (id jet-publish-lib-flow) is complete and deterministic: optional build_library, read+transform package.json, validate+auto-fill main/module/exports/types (terminal val_err on missing), require identity, resolve registry via .npmrc scope+auth, pack, then branch pack-only (.tgz) vs publish (PUT base64 Bearer). All nodes reachable; decisions (build_q, val_ok, pack_only) carry labeled branches; terminals (val_err, write_tgz, done) are real ends. Private-registry routing reuses the hardened npmrc path; scope correct (build is A1/A2).
