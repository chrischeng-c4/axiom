---
id: loom-security-hardening-ec
summary: Loom security evidence is guard-owned and vat-isolated, with meter evidence attached for control-plane smoke.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening

loom's security gate is the guard report over its HTTP / Kubernetes / container
surface. guard scans the source (the control-plane API, the raft peer transport,
the k8s operator/RBAC render, the Dockerfiles) and attaches meter evidence for
the behaviour suite so request-boundary security cannot be marked complete
without runtime proof. The gate runs inside vat so it never mutates the host
checkout.

Relevant surfaces:

- **Control API** (`/runs*`) — bounded, JSON-only control messages; payload bytes
  claim-check through keep and never reach loom.
- **Standard probes** (`/healthz` `/readyz` `/metrics` `/openapi.json` `/docs`)
  are auth-exempt and always-on by design (probes/scrape depend on them).
- **Operator RBAC** — the ClusterRole is scoped to the child objects the
  reconcile loop applies + the leader-election Lease, nothing broader.
- **Container** — the release image runs non-root (uid 65532) with a fetched,
  sha256-verified binary.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: loom-security-hardening-guard-scan
    capability_id: security-hardening
    claim_id: vat-guard-security-gate
    contract_id: loom-guard-security-report
    category: security
    test_path: projects/loom/src/controller.rs
    command: "cd projects/loom && ../../target/debug/vat run guard-security"
    assertions:
      - "guard owns the pass/fail evidence for loom's HTTP / k8s / container security surface."
      - "guard attaches meter evidence for the control-plane behaviour suite (runtime proof)."
      - "The gate runs inside vat so report artifacts and transient state do not mutate the host checkout."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: loom-guard-security
    tool: guard
    manifest: guard-loom-security.toml
    category: security
    delegate_command: "cd projects/loom && ../../target/debug/vat run guard-security"
```
