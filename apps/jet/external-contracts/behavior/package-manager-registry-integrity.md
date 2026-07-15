---
id: package-manager-registry-integrity
summary: External contract for Package Manager Registry Integrity.
fill_sections: [e2e-test]
---

# EC: Package Manager Registry Integrity

---
id: package-manager-registry-integrity
summary: External contract for Package Manager Registry Integrity.
fill_sections: [e2e-test]
---

# EC: Package Manager Registry Integrity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: package-manager-registry-integrity
    capability_id: package-manager
    claim_id: package-manager-registry-integrity
    contract_id: package-manager-registry-integrity
    category: behavior
    command: "cargo test -p jet --lib pkg_manager -- --nocapture"
    assertions:
      - "Jet package registry resolution preserves package metadata and integrity guarantees."
```
