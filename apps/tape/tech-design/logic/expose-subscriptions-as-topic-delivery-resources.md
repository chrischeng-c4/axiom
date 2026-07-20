---
id: "1254"
summary: (fill)
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-subscription-resource-contract
entry: create
nodes:
  create:
    kind: start
    label: "tape subscription create TOPIC NAME creates one caller-driven pull cursor"
  validate:
    kind: decision
    label: "the topic/name resource does not already exist"
  invalid:
    kind: terminal
    label: "nonzero CLI result names the existing resource; the journal stays unchanged"
  persist:
    kind: process
    label: "persist Subscription in the existing file-backed TapeJournal; create has no side effect on checkpoints"
  pull_view:
    kind: process
    label: "a pull subscription exposes the current optional checkpoint at topic/name so existing checkpoint get/put remains the cursor API"
  resource_ops:
    kind: process
    label: "list/show/delete address subscriptions by topic/name; delete removes only resource metadata and does not delete the consumer checkpoint"
  api_inventory:
    kind: terminal
    label: "spec inventory declares POST/GET /topics/{topic}/subscriptions and GET/DELETE /topics/{topic}/subscriptions/{subscription}; schemas contain no delivery-mode switch"
edges:
  - { from: create, to: validate }
  - { from: validate, to: invalid, label: "invalid" }
  - { from: validate, to: persist, label: "valid" }
  - { from: persist, to: pull_view }
  - { from: pull_view, to: resource_ops }
  - { from: resource_ops, to: api_inventory }
---
flowchart TD
    create["subscription create TOPIC NAME"] --> validate{"unique topic/name resource?"}
    validate -->|invalid| invalid(["nonzero; journal unchanged"])
    validate -->|valid| persist["persist Subscription in the file-backed journal; do not advance checkpoint"]
    persist --> pull_view["checkpoint remains topic/name cursor API"]
    pull_view --> resource_ops["list/show/delete by topic/name; delete preserves checkpoint"]
    resource_ops --> api_inventory(["declare create/list/show/delete routes and delivery schemas in spec inventory"])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-subscription-resource-verification
requirements:
  pull_checkpoint_compatibility:
    id: R3
    text: "Creating or deleting a pull subscription does not advance or remove the existing topic/name consumer checkpoint; checkpoint get/put remains the durable pull cursor interface."
    kind: regression
    risk: high
    verify: cargo test -p tape tests::pull_subscription_preserves_checkpoint_compatibility --lib -- --exact
  pull_only_boundary:
    id: R5
    text: "Subscription creation is intrinsically pull-only; the CLI and schema expose no push endpoint, lease, or consumer-group mode."
    kind: negative
    risk: medium
    verify: cargo test -p tape --test cli_contract subscription_cli_surface -- --exact
  spec_inventory:
    id: R4
    text: "The offline routes, OpenAPI, and JSON Schema inventories declare the topic-scoped subscription collection/item routes and delivery configuration schemas."
    kind: contract
    risk: medium
    verify: cargo test -p tape --test cli_contract subscription_spec_inventory -- --exact
  subscription_cli_surface:
    id: R1
    text: "tape --help and tape subscription create --help expose pull-only subscription creation, inspection, and deletion without delivery-mode flags."
    kind: functional
    risk: medium
    verify: cargo test -p tape --test cli_contract subscription_cli_surface -- --exact
  subscription_local_lifecycle:
    id: R2
    text: "A named pull subscription can be created, listed, shown, and deleted in one file-backed journal without changing unrelated entries."
    kind: functional
    risk: high
    verify: cargo test -p tape --test cli_contract subscription_resource_roundtrip -- --exact
---
flowchart TD
    r1[R1 subscription cli surface] --> cargo_test_p_tape_test_cli_contract_subscription_cli_surface_exact[cargo test -p tape --test cli_contract subscription_cli_surface -- --exact]
    r2[R2 subscription local lifecycle] --> cargo_test_p_tape_test_cli_contract_subscription_resource_roundtrip_exact[cargo test -p tape --test cli_contract subscription_resource_roundtrip -- --exact]
    r5[R5 pull-only boundary] --> cargo_test_p_tape_test_cli_contract_subscription_cli_surface_exact
    r3[R3 pull checkpoint compatibility] --> cargo_test_p_tape_tests_pull_subscription_preserves_checkpoint_compatibility_lib_exact[cargo test -p tape tests::pull_subscription_preserves_checkpoint_compatibility --lib -- --exact]
    r4[R4 spec inventory] --> cargo_test_p_tape_test_cli_contract_subscription_spec_inventory_exact[cargo test -p tape --test cli_contract subscription_spec_inventory -- --exact]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the file-backed pull-only Subscription model, keyed CRUD on TapeJournal, duplicate/not-found errors, and a regression test proving resource lifecycle leaves the existing topic/name checkpoint untouched. Keep this within the existing journal ownership boundary; generator gap: missing-generator:logic:tape-subscription-resource (#1254)."
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add pull-only `tape subscription create|list|show|delete`; commands load/save the existing `--store` journal and emit JSON plus a runnable `next:` marker. generator gap: missing-generator:cli:tape-subscription-resource (#1254)."
  - path: apps/tape/src/spec.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Declare the topic-scoped pull-subscription collection and item routes in routes_json/OpenAPI, define name-only create/resource schemas, and keep the served HTTP contract aligned. Create/delete mutations use the Raft command path in replica mode. generator gap: missing-generator:openapi:tape-subscription-resource (#1254)."
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add CLI-level deterministic coverage for help, pull-only create/list/show/delete round-trip, push-flag rejection, and routes/OpenAPI/schema inventory. generator gap: missing-generator:test:tape-subscription-resource (#1254)."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Document subscription resources as a pull-only Tape core capability with checkpoint compatibility; do not expose push, leases, consumer groups, or raft-backed executor ownership."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-lib-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Refresh the source snapshot from apps/tape/src/lib.rs after the subscription journal model changes."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Refresh the source snapshot from apps/tape/src/bin/tape.rs after CLI changes."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-spec-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Refresh the source snapshot from apps/tape/src/spec.rs after contract inventory changes."
  - path: apps/tape/tech-design/semantic/source/apps-tape-tests-cli-contract-rs.md
    action: modify
    section: source
    impl_mode: hand-written
    description: "Refresh the source snapshot from apps/tape/tests/cli_contract.rs after subscription contract tests."
```
