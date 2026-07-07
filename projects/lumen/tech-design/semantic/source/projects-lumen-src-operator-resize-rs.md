<!-- HANDWRITE-BEGIN gap="missing-generator:source:4b636bc9" tracker="pending-tracker" reason="New SPEC-MANAGED rust-source-unit tech-design doc for resize.rs, mirroring the format of the other projects-lumen-src-operator-*-rs.md docs." -->
---
id: projects-lumen-src-operator-resize-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: partial
    rationale: >
      Hand-written module (#809), thinned to an adapter by #970: the
      pure quantity-parsing (`parse_storage_bytes`), the grow/no-op/shrink
      decision (`ResizeAction`/`decide`), the `PvcResizeOutcome` shape, and
      the impure PVC-list + `StorageClass`-gated patch driver
      (`resize_instance`) all moved to the shared `libs/operator::resize`
      module (universal to every sharded-StatefulSet operator, e.g. keep).
      This file keeps only the Lumen-specific glue: fetch the named `Lumen`
      CR for `spec.serving.raftStorage`, then delegate to the lib with the
      `app.kubernetes.io/instance=<name>` label selector and the
      `raft-<name>-` PVC name filter. No generator primitive exists yet for
      this shape, so it stays HANDWRITE per CLAUDE.md ("no skip state for
      source ownership") until a generator primitive covers it.
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/resize.rs

## Overview
<!-- type: overview lang: markdown -->

`lumen k8s operator resize-storage` (#809) support module. StatefulSet
`volumeClaimTemplates` are immutable after creation, so bumping
`spec.serving.raftStorage` on a live `Lumen` CR and letting the operator
reconcile does **not** resize anything — the rendered StatefulSet's `apply`
is a silent no-op for that field. The generic detect-and-patch tool (quantity
parsing, grow/no-op/shrink decision, PVC listing + `StorageClass`-gated
patch) now lives in the shared `operator::resize` module (#970, universal to
every sharded-StatefulSet operator); this file is the thin Lumen-specific
adapter that fetches the CR's declared `spec.serving.raftStorage` and scopes
the PVC listing to the instance's `raft-<name>-<n>` volumes via a label
selector + name filter passed into `operator::resize::resize_instance`.
Gated `#[cfg(feature = "operator")]`; the default (no-feature) build links no
kube client.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `resize_instance` | projects/lumen/src/operator/resize.rs | function | pub async | 28 | resize_instance(client: kube::Client, namespace: &str, name: &str, dry_run: bool) -> Result<Vec<PvcResizeOutcome>> |

`decide`, `parse_storage_bytes`, `PvcResizeOutcome`, and `ResizeAction` are
re-exported (`pub use operator::resize::{...}`) from `libs/operator::resize`,
not defined in this file — see
`projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-lease-rs.md`
for the same re-export convention.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// HANDWRITE-BEGIN gap="missing-generator:logic:7b95a80b" tracker="pending-tracker" reason="Thin Lumen-specific adapter over the shared `operator::resize` module (#970): fetches the named Lumen CR for spec.serving.raftStorage and delegates PVC listing/quantity-compare/patch logic (parse_storage_bytes, ResizeAction/decide, PvcResizeOutcome, resize_instance) to libs/operator, supplying the `app.kubernetes.io/instance=<name>` label selector and the `raft-<name>-` PVC name filter. No generator primitive exists yet for this shape, so it stays HANDWRITE per CLAUDE.md until one covers it."
//! `lumen k8s operator resize-storage` (#809) support module.
//!
//! StatefulSet `volumeClaimTemplates` are immutable after creation, so
//! bumping `spec.serving.raftStorage` on a live `Lumen` CR and letting the
//! operator reconcile does **not** resize anything — the rendered
//! StatefulSet's `apply` is a silent no-op for that field. The generic
//! detect-and-patch tool (quantity parsing, grow/no-op/shrink decision, PVC
//! listing + `StorageClass`-gated patch) lives in the shared
//! [`operator::resize`] module (#970); this module is the thin Lumen-specific
//! adapter that fetches the CR's declared `spec.serving.raftStorage` and
//! scopes the PVC listing to the instance's `raft-<name>-<n>` volumes.

use anyhow::{Context, Result};
use kube::api::Api;

use super::crd::Lumen;

pub use operator::resize::{decide, parse_storage_bytes, PvcResizeOutcome, ResizeAction};

/// List the live `raft-<name>-<n>` PVCs for the named `Lumen` instance,
/// compare each to the CR's declared `spec.serving.raftStorage`, and patch
/// `spec.resources.requests.storage` on PVCs that are safe to grow (bound
/// `StorageClass.allowVolumeExpansion == true`, `dry_run == false`).
/// No other PVC field is touched, and the CR itself is never mutated — the
/// deployer edits `spec.serving.raftStorage` separately, the same source of
/// truth `render()` already reads.
pub async fn resize_instance(
    client: kube::Client,
    namespace: &str,
    name: &str,
    dry_run: bool,
) -> Result<Vec<PvcResizeOutcome>> {
    let lumens: Api<Lumen> = Api::namespaced(client.clone(), namespace);
    let cr = lumens
        .get(name)
        .await
        .with_context(|| format!("get Lumen '{namespace}/{name}'"))?;
    let desired = cr.spec.serving.raft_storage.clone();

    let label_selector = format!("app.kubernetes.io/instance={name}");
    let prefix = format!("raft-{name}-");

    operator::resize::resize_instance(
        client,
        namespace,
        &label_selector,
        |pvc_name| pvc_name.starts_with(&prefix),
        |_pvc_name| desired.clone(),
        dry_run,
    )
    .await
}
// HANDWRITE-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/resize.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      Thinned to an adapter (#970): parse_storage_bytes, ResizeAction/decide,
      PvcResizeOutcome, and the PVC-list + quantity-compare + conditional-patch
      driver (resize_instance) moved to the shared libs/operator::resize module
      (reusable by any sharded-StatefulSet operator, e.g. keep). This file
      keeps only the Lumen-specific glue: fetch the named Lumen CR for
      spec.serving.raftStorage, then delegate to
      operator::resize::resize_instance with the
      app.kubernetes.io/instance=<name> label selector and the raft-<name>-
      PVC name filter. Byte-compatible parsing/patch semantics — no behavior
      change, same --dry-run contract.
  - path: libs/operator/src/resize.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      New shared module (#970), lifted from lumen's operator::resize: pure
      quantity parsing (parse_storage_bytes), grow/no-op/shrink decision
      (ResizeAction/decide, moved unit tests included), the PvcResizeOutcome
      shape, and the impure resize_instance driver — now parameterized by a
      PVC label selector, a PVC-name filter closure, and a desired-storage
      accessor closure instead of a Lumen CR type, so no libs/operator caller
      needs a Lumen dependency. Parsing and patch semantics unchanged from the
      lumen-private implementation.
```
<!-- HANDWRITE-END -->
