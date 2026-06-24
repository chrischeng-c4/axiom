---
id: projects-jet-logic-jet-build-lib-cjs-re-export-and-renamed-alias-edge-cases-md
fill_sections: [logic]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "Correct CJS rewriting for re-exports and renamed aliases completes the CJS library output of library-build-publishing."
---

# jet build --lib CJS: Re-export and Renamed-Alias Edge Cases

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-cjs-reexport
entry: line
nodes:
  line:    { kind: start,    label: "esm_to_cjs processes an export/import line" }
  cls:     { kind: decision, label: "line shape" }
  ext:     { kind: process,  label: "export {x} from pkg: exports.x = require(pkg).x" }
  intl:    { kind: process,  label: "export {a as b} from ./m: exports.b = require(./m).a" }
  star:    { kind: process,  label: "export * from m: Object.assign(exports, require(m))" }
  alias:   { kind: process,  label: "local export {a as b}: exports.b = a" }
  other:   { kind: process,  label: "other line: existing rewrite (unchanged)" }
  done:    { kind: terminal, label: "CJS line emitted" }
edges:
  - { from: line,  to: cls }
  - { from: cls,   to: ext,   label: "export-from-pkg" }
  - { from: cls,   to: intl,  label: "export-from-relative" }
  - { from: cls,   to: star,  label: "export-star" }
  - { from: cls,   to: alias, label: "local-alias" }
  - { from: cls,   to: other, label: "other" }
  - { from: ext,   to: done }
  - { from: intl,  to: done }
  - { from: star,  to: done }
  - { from: alias, to: done }
  - { from: other, to: done }
---
flowchart TD
    line([esm_to_cjs line]) --> cls{line shape}
    cls -->|export-from-pkg| ext[exports.x = require pkg .x]
    cls -->|export-from-relative| intl[exports.b = require ./m .a]
    cls -->|export-star| star[Object.assign exports require m]
    cls -->|local-alias| alias[exports.b = a]
    cls -->|other| other[existing rewrite]
    ext --> done([CJS line])
    intl --> done
    star --> done
    alias --> done
    other --> done
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicability sound: classify each export/import line and rewrite re-export-from-pkg, re-export-from-relative, export-star, and local renamed-alias to correct CJS exports/require; other lines unchanged. Extends A1 CJS; ESM/preserve/IIFE out of scope.
