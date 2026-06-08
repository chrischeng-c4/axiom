---
id: score-td-review-counter-reset
summary: Reset issue CRRR transient counters when entering TD lifecycle.
fill_sections: [logic, test-plan, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# TD Review Counter Reset

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: score-td-review-counter-reset-logic
entry: start
nodes:
  start:
    kind: start
    label: "aw td create <slug>"
  load_issue:
    kind: process
    label: "load open issue"
  enter_td:
    kind: process
    label: "activate TD workspace branch"
  reset_transient:
    kind: process
    label: "apply phase=td_inited active branch clear_transient=true"
  review_brief:
    kind: process
    label: "aw td review computes round from reset review_count"
  first_review:
    kind: decision
    label: "first TD review verdict"
  revise:
    kind: process
    label: "needs-revision dispatches aw td revise"
  gen_code:
    kind: process
    label: "approved dispatches aw td gen-code"
  done:
    kind: terminal
    label: "TD lifecycle continues"
edges:
  - from: start
    to: load_issue
    label: invoke
  - from: load_issue
    to: enter_td
    label: issue is open
  - from: enter_td
    to: reset_transient
    label: initialize TD state
  - from: reset_transient
    to: review_brief
    label: issue review_count cleared
  - from: review_brief
    to: first_review
    label: Round 1
  - from: first_review
    to: revise
    label: needs-revision
  - from: first_review
    to: gen_code
    label: approved
  - from: revise
    to: done
    label: normal revise loop
  - from: gen_code
    to: done
    label: normal codegen loop
---
flowchart TD
    start([aw td create slug]) --> load_issue[load open issue]
    load_issue --> enter_td[activate TD workspace branch]
    enter_td --> reset_transient[phase td_inited + active branch + clear transient counters]
    reset_transient --> review_brief[aw td review computes Round 1]
    review_brief --> first_review{first TD review verdict}
    first_review -- needs-revision --> revise[dispatch aw td revise]
    first_review -- approved --> gen_code[dispatch aw td gen-code]
    revise --> done([TD lifecycle continues])
    gen_code --> done
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: score-td-review-counter-reset-tests
requirements:
  td_init_resets_issue_review_count:
    id: TD-RC-1
    text: "aw td create clears an inherited issue review_count while preserving phase and branch"
    kind: functional
    risk: high
    verify: test
  td_init_resets_review_labels:
    id: TD-RC-2
    text: "serialized lifecycle issue file after TD init no longer carries review_count, flagged_sections, or fill_retry_count"
    kind: functional
    risk: medium
    verify: inspection
  first_td_review_starts_at_one:
    id: TD-RC-3
    text: "first TD review round is derived from the reset TD state, not the prior issue CRRR state"
    kind: functional
    risk: high
    verify: test
elements:
  inplace_mode_test:
    kind: test
    type: "rs/cargo-test"
relations:
  - { from: inplace_mode_test, verifies: td_init_resets_issue_review_count }
  - { from: inplace_mode_test, verifies: td_init_resets_review_labels }
  - { from: inplace_mode_test, verifies: first_td_review_starts_at_one }
---
requirementDiagram
    requirement td_init_resets_issue_review_count {
        id: TD-RC-1
        text: "aw td create clears an inherited issue review_count while preserving phase and branch"
        risk: high
        verifymethod: test
    }

    requirement td_init_resets_review_labels {
        id: TD-RC-2
        text: "serialized lifecycle issue file after TD init no longer carries review_count, flagged_sections, or fill_retry_count"
        risk: medium
        verifymethod: inspection
    }

    requirement first_td_review_starts_at_one {
        id: TD-RC-3
        text: "first TD review round is derived from the reset TD state, not the prior issue CRRR state"
        risk: high
        verifymethod: test
    }

    element inplace_mode_test {
        type: file
    }

    td_init_resets_issue_review_count - satisfies -> inplace_mode_test
    td_init_resets_review_labels - satisfies -> inplace_mode_test
    first_td_review_starts_at_one - satisfies -> inplace_mode_test
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/td.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: Clear transient issue CRRR fields when provisioning TD workspace state.
  - path: projects/agentic-workflow/tests/inplace_mode_test.rs
    action: modify
    section: test-plan
    impl_mode: hand-written
    description: Cover TD init with an inherited issue review_count and assert the active branch lifecycle issue file resets it.
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] TD init reset is the correct lifecycle boundary because the bug is inherited issue CRRR transient state leaking into TD review routing.
- [test-plan] The proposed in-place lifecycle regression should reproduce the issue by seeding review_count before `aw td create`, then asserting the active branch lifecycle issue file has no inherited review counter.
