---
id: jet-project-architecture-and-authoring-clarity
summary: >
  Jet moves the project-root uppercase meta doc projects/jet/LAYOUT.md into
  the scoped project doc projects/jet/docs/architecture/layout.md, preserving
  the top-level path map and crate/package naming conventions, and relinks
  the Jet README so the layout guide stays discoverable without a
  project-root uppercase meta doc.
capability_refs:
  - id: jet-project-architecture-and-authoring-clarity
    role: primary
    gap: move-root-layout-meta-doc-into-scoped-architecture-documentation
    claim: move-root-layout-meta-doc-into-scoped-architecture-documentation
    coverage: full
    rationale: "Defines the bounded root-layout-meta-doc-to-scoped-doc migration requested by WI #1169, pinned to the 'Move root layout meta doc into scoped architecture documentation' work root."
fill_sections: [logic, config, unit-test, changes]
---

# jet: move root layout meta doc into scoped architecture documentation
