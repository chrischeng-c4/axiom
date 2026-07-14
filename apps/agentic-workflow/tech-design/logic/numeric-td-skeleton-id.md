---
id: numeric-td-skeleton-id
summary: Preserve generated TD skeleton ids as YAML strings so numeric GitHub WI ids can enter the applicability section-apply loop without manual frontmatter edits.
fill_sections: [logic, unit-test]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: numeric-td-skeleton-ids
    claim: numeric-td-skeleton-ids
    coverage: full
    rationale: "The TD create producer must emit a validator-compatible skeleton for both numeric issue ids and non-numeric slugs."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: committed-td-skeleton-lifecycle
    claim: committed-td-skeleton-lifecycle
    coverage: full
    rationale: "The TD create producer owns, safely recovers, canonicalizes, and commits its exact empty skeleton as part of the queue-start lifecycle."
---

# Numeric TD skeleton IDs

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: numeric-td-skeleton-id
entry: start
nodes:
  start: { kind: start, label: "initialize_td_spec_skeleton(path, slug)" }
  exists: { kind: decision, label: "spec already exists?" }
  preserve: { kind: terminal, label: "return false; preserve authored bytes" }
  exact_untracked: { kind: decision, label: "sole exact ?? target with known empty bytes?" }
  canonicalize: { kind: process, label: "canonicalize exact historical empty skeleton" }
  encode: { kind: process, label: "serde_yaml::to_string(slug) as a string scalar" }
  write: { kind: process, label: "write skeleton frontmatter with encoded id" }
  queue_start: { kind: process, label: "stage skeleton in Td-Queue-Start" }
  apply: { kind: process, label: "merge first applicability section" }
  validate: { kind: decision, label: "frontmatter id parses with as_str()?" }
  success: { kind: terminal, label: "section apply validates" }
  reject: { kind: terminal, label: "serialization or validation error" }
edges:
  - { from: start, to: exists }
  - { from: exists, to: exact_untracked, label: "yes" }
  - { from: exists, to: encode, label: "no" }
  - { from: exact_untracked, to: canonicalize, label: "yes" }
  - { from: exact_untracked, to: preserve, label: "no" }
  - { from: canonicalize, to: queue_start }
  - { from: encode, to: write }
  - { from: write, to: queue_start }
  - { from: queue_start, to: apply }
  - { from: apply, to: validate }
  - { from: validate, to: success, label: "yes" }
  - { from: validate, to: reject, label: "no" }
---
flowchart TD
  start([initialize skeleton]) --> exists{spec exists?}
  exists -->|yes| exact_untracked{sole exact untracked known-empty skeleton?}
  exists -->|no| encode[serialize slug as YAML string]
  exact_untracked -->|yes| canonicalize[canonicalize historical skeleton]
  exact_untracked -->|no| preserve([preserve authored bytes])
  encode --> write[write skeleton]
  canonicalize --> queue_start[stage skeleton in queue-start commit]
  write --> queue_start
  queue_start --> apply[merge first applicability section]
  apply --> validate{id is string?}
  validate -->|yes| success([section apply validates])
  validate -->|no| reject([error])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: numeric-td-skeleton-id-tests
requirements:
  numeric_string_id:
    id: R1
    text: "A generated skeleton for numeric WI id 1487 parses its top-level id as the string 1487."
    kind: functional
    risk: high
    verify: test
  first_section_apply:
    id: R2
    text: "Merging the first applicability logic payload into the numeric-id skeleton passes section-apply validation."
    kind: functional
    risk: high
    verify: test
  preserve_existing_behavior:
    id: R3
    text: "A non-numeric slug remains a valid id and repeated initialization does not overwrite authored content."
    kind: functional
    risk: medium
    verify: test
  exact_untracked_only:
    id: R4
    text: "Recovery accepts only the sole exact untracked target with byte-identical known empty skeleton content; tracked, staged, renamed, authored, symlink, and sibling-dirty states are rejected without mutation."
    kind: functional
    risk: high
    verify: test
  queue_start_owns_skeleton:
    id: R5
    text: "Fresh and recovered skeletons are canonicalized and staged by exactly one Td-Queue-Start commit, leaving the checkout clean and repeated brief calls history-idempotent."
    kind: functional
    risk: high
    verify: test
  rebased_recovery_preserves_candidate:
    id: R6
    text: "An unreachable Td-Init may carry only the admitted skeleton through Td-Reset and fresh Td-Init before queue start; a reachable non-td_inited phase (including td_created), post-gen, and terminal phases reject it without mutation."
    kind: functional
    risk: high
    verify: test
elements:
  initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply:
    kind: test
    type: "rs/#[test]"
  initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent:
    kind: test
    type: "rs/#[test]"
  td_skeleton_recovery_requires_exact_untracked_status_and_clean_siblings:
    kind: test
    type: "rs/#[test]"
  td_skeleton_recovery_accepts_only_exact_historical_empty_bytes:
    kind: test
    type: "rs/#[test]"
  td_skeleton_recovery_rejects_untracked_symlink:
    kind: test
    type: "rs/#[test]"
  td_create_commits_fresh_numeric_skeleton_once:
    kind: test
    type: "rs/#[test]"
  td_create_recovers_reachable_locked_legacy_skeleton_once:
    kind: test
    type: "rs/#[test]"
  td_create_rebased_lifecycle_reprovisions_untracked_legacy_skeleton:
    kind: test
    type: "rs/#[test]"
  td_create_rejects_authored_tracked_staged_and_sibling_skeleton_states:
    kind: test
    type: "rs/#[test]"
  td_create_post_gen_and_terminal_phases_reject_untracked_skeleton:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply, verifies: numeric_string_id }
  - { from: initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply, verifies: first_section_apply }
  - { from: initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent, verifies: preserve_existing_behavior }
  - { from: td_skeleton_recovery_requires_exact_untracked_status_and_clean_siblings, verifies: exact_untracked_only }
  - { from: td_skeleton_recovery_accepts_only_exact_historical_empty_bytes, verifies: exact_untracked_only }
  - { from: td_skeleton_recovery_rejects_untracked_symlink, verifies: exact_untracked_only }
  - { from: td_create_rejects_authored_tracked_staged_and_sibling_skeleton_states, verifies: exact_untracked_only }
  - { from: td_create_commits_fresh_numeric_skeleton_once, verifies: queue_start_owns_skeleton }
  - { from: td_create_recovers_reachable_locked_legacy_skeleton_once, verifies: queue_start_owns_skeleton }
  - { from: td_create_rebased_lifecycle_reprovisions_untracked_legacy_skeleton, verifies: rebased_recovery_preserves_candidate }
  - { from: td_create_post_gen_and_terminal_phases_reject_untracked_skeleton, verifies: rebased_recovery_preserves_candidate }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "numeric skeleton id is a YAML string"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "first applicability section validates"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "slug and idempotence remain valid"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "only exact untracked empty skeleton recovers"
      risk: high
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "queue-start owns skeleton exactly once"
      risk: high
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: "rebased recovery preserves only admitted candidate"
      risk: high
      verifymethod: test
    }
    element initialize_td_spec_skeleton_numeric_id_accepts_first_section_apply {
      type: "rs/#[test]"
    }
    element initialize_td_spec_skeleton_writes_frontmatter_and_is_idempotent {
      type: "rs/#[test]"
    }
    element td_skeleton_recovery_requires_exact_untracked_status_and_clean_siblings {
      type: "rs/#[test]"
    }
    element td_skeleton_recovery_accepts_only_exact_historical_empty_bytes {
      type: "rs/#[test]"
    }
    element td_skeleton_recovery_rejects_untracked_symlink {
      type: "rs/#[test]"
    }
    element td_create_commits_fresh_numeric_skeleton_once {
      type: "rs/#[test]"
    }
    element td_create_recovers_reachable_locked_legacy_skeleton_once {
      type: "rs/#[test]"
    }
    element td_create_rebased_lifecycle_reprovisions_untracked_legacy_skeleton {
      type: "rs/#[test]"
    }
    element td_create_rejects_authored_tracked_staged_and_sibling_skeleton_states {
      type: "rs/#[test]"
    }
    element td_create_post_gen_and_terminal_phases_reject_untracked_skeleton {
      type: "rs/#[test]"
    }
```
