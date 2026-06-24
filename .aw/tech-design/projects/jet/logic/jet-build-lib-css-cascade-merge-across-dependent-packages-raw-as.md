---
id: projects-jet-logic-jet-build-lib-css-cascade-merge-across-dependent-packages-raw-as-md
fill_sections: [logic, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: library-build-mode
    coverage: partial
    rationale: "jet build --lib CSS cascade-merge + raw asset copy"
---

# jet build --lib CSS cascade-merge + raw asset copy

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-lib-css-merge
entry: post
nodes:
  post: { kind: start,    label: "after lib entry/css emit" }
  merge: { kind: decision, label: "[lib].css_merge configured?" }
  concat: { kind: process,  label: "concat dependent packages style.css in declared order -> style.css" }
  copy: { kind: decision, label: "[lib].raw_copy configured?" }
  rawcopy: { kind: process,  label: "copy raw asset dirs verbatim into out_dir (deep-import paths)" }
  done: { kind: terminal, label: "lib output incl merged css + raw assets" }
edges:
  - { from: post,   to: merge }
  - { from: merge,  to: concat, label: "yes" }
  - { from: merge,  to: copy,   label: "no" }
  - { from: concat, to: copy }
  - { from: copy,   to: rawcopy, label: "yes" }
  - { from: copy,   to: done,    label: "no" }
  - { from: rawcopy, to: done }
---
flowchart TD
    post([after lib emit]) --> merge{css_merge configured?}
    merge -->|yes| concat[concat dep style.css in order]
    merge -->|no| copy{raw_copy configured?}
    concat --> copy
    copy -->|yes| rawcopy[copy raw asset dirs verbatim]
    copy -->|no| done([lib output])
    rawcopy --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/task_runner/config.rs"
    action: modify
    section: logic
    description: |
      Add [lib] config: css_merge (ordered list of dependent style.css to concatenate) and raw_copy (src dir -> out dir verbatim copies).
    impl_mode: hand-written
  - path: "projects/jet/src/bundler/lib_build.rs"
    action: modify
    section: logic
    description: |
      After lib emit: when css_merge configured, concatenate the dependent packages style.css into the output style.css in declared cascade order; when raw_copy configured, copy the raw asset directories verbatim into out_dir preserving paths for deep imports.
    impl_mode: hand-written
  - path: "projects/jet/tests/build/lib_css_merge.rs"
    action: create
    section: unit-test
    description: |
      Tests: css_merge produces style.css with dep CSS in declared order; raw_copy lands icons/images/audio verbatim at deep-import paths; neither configured = unchanged.
    impl_mode: hand-written
```

