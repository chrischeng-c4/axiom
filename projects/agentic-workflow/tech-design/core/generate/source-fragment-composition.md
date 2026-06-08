---
id: sdd-source-fragment-composition
fill_sections: [changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Source Fragment Composition

## Overview
<!-- type: overview lang: markdown -->

Large claimed source files can exceed the spec markdown hard limit when a
single `type: source` section owns the whole file. The `source` template must
therefore support multiple codegen fragments for the same Rust module without
inventing narrower language-specific section types.

This spec adds two `replaces:` sentinel symbols for module-edge ownership and
extends ordinary symbol replacement to `mod` blocks:

- `<module-preamble>` replaces the module prefix before the first top-level
  item, including a whole-file HANDWRITE opener, module docs, attributes, and
  imports.
- `<module-trailer>` replaces the suffix after the last top-level item,
  including a whole-file HANDWRITE closer.
- `<handwrite-gap:...>` replaces a tracked HANDWRITE region by matching its
  `gap` attribute, including the opener and closer marker lines.
- `mod <name>` blocks are replaceable through the existing bare symbol form,
  so `replaces: [tests]` can own `#[cfg(test)] mod tests { ... }`.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/apply.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - special_module_replace_range
      - special_replace_range
      - find_handwrite_gap_range
      - strip_handwrite_comment_lead
      - extract_handwrite_attr
      - is_handwrite_open
      - is_handwrite_close
      - find_module_preamble_range
      - find_module_trailer_range
      - is_module_item_start
      - match_item_start
      - apply_replaces_handles_module_blocks
      - apply_replaces_handles_module_preamble_and_trailer
      - apply_replaces_handles_handwrite_gap_blocks
    description: |
      Allow `source` specs to split a large Rust module across several
      fragments by replacing module-edge ranges, tracked HANDWRITE regions,
      and test modules through the existing `replaces:` mechanism. This keeps
      `section: source` as the cross-language raw template while adding
      Rust-aware placement rules only at apply time.
```

## Reviews
<!-- type: review lang: markdown -->

**Verdict:** approved

- [changes] The change strengthens template composition instead of adding a
  language-specific section taxonomy. It is scoped to `apply.rs` replacement
  mechanics and has direct unit coverage for the new sentinels and `mod`
  replacement.
