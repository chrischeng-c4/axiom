---
id: semantic-lumen-ec-gates
summary: Semantic coverage for Lumen EC gate inventory, vat dispatch, and production claim closure artifacts.
capability_refs:
  - id: "ec-gates-configured"
    role: primary
    gap: "aw-ec-generated-inventory-and-dispatch"
    claim: "aw-ec-generated-inventory-and-dispatch"
    coverage: full
    rationale: "The project-local aw.toml EC inventory is the generated dispatch catalog consumed by AW health."
  - id: "ec-gates-configured"
    role: primary
    gap: "vat-managed-meter-and-rig-runners"
    claim: "vat-managed-meter-and-rig-runners"
    coverage: full
    rationale: "The project-local vat.toml owns the meter and rig runner bindings used by Lumen EC gates."
  - id: "ec-gates-configured"
    role: primary
    gap: "external-contract-claim-closure-evidence"
    claim: "external-contract-claim-closure-evidence"
    coverage: full
    rationale: "The production claim-closure EC document maps README promises to executable evidence."
  - id: "replica-sync-bootstrap"
    role: primary
    gap: "external-backup-disaster-recovery-seed"
    claim: "external-backup-disaster-recovery-seed"
    coverage: partial
    rationale: "The claim-closure EC inventory ties the external backup seed promise to backup/restore executable evidence."
fill_sections: [schema, changes]
---

# Semantic TD: Lumen EC Gates

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/ec-gates"
  source_group: "apps/lumen/ec-gates"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/aw.toml"
        language: "toml"
        ownership_state: "codegen"
        generator_primitives: ["ec_inventory"]
        source_evidence_node:
          layer: "configuration"
          ecosystem: "aw"
          role: "ec-inventory"
          section_type: "schema"
          domain: "apps/lumen/ec-gates"
      - path: "apps/lumen/vat.toml"
        language: "toml"
        ownership_state: "handwrite"
        generator_primitives: ["external_runner_manifest"]
        source_evidence_node:
          layer: "configuration"
          ecosystem: "vat"
          role: "ec-runner-dispatch"
          section_type: "schema"
          domain: "apps/lumen/ec-gates"
      - path: "apps/lumen/external-contracts/claim-closure/production-claims.md"
        language: "markdown"
        ownership_state: "codegen"
        generator_primitives: ["ec_claim_closure"]
        source_evidence_node:
          layer: "verification"
          ecosystem: "aw"
          role: "production-claim-closure"
          section_type: "schema"
          domain: "apps/lumen/ec-gates"
      - path: "apps/lumen/external-contracts/ec.lock"
        language: "toml"
        ownership_state: "codegen"
        generator_primitives: ["ec_lock"]
        source_evidence_node:
          layer: "verification"
          ecosystem: "aw"
          role: "ec-ir-lock"
          section_type: "schema"
          domain: "apps/lumen/ec-gates"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/lumen/aw.toml"
    action: modify
    section: schema
    description: |
      Generated EC inventory and dispatch commands are covered by this semantic TD.
    impl_mode: codegen
  - path: "apps/lumen/vat.toml"
    action: modify
    section: schema
    description: |
      Vat-managed meter and rig runner dispatch is covered by this semantic TD.
    impl_mode: hand-written
  - path: "apps/lumen/external-contracts/claim-closure/production-claims.md"
    action: modify
    section: schema
    description: |
      Production claim closure mappings are covered by this semantic TD.
    impl_mode: codegen
  - path: "apps/lumen/external-contracts/ec.lock"
    action: modify
    section: schema
    description: |
      EC IR lock freshness is covered by this semantic TD.
    impl_mode: codegen
```
