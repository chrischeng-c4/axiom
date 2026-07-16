---
id: package-manager-lockfile-parity
summary: External contract for Package Manager Lockfile Parity.
fill_sections: [e2e-test]
---

# EC: Package Manager Lockfile Parity

---
id: package-manager-lockfile-parity
summary: External contract for Package Manager Lockfile Parity.
fill_sections: [e2e-test]
---

# EC: Package Manager Lockfile Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: package-manager-lockfile-parity
    capability_id: package-manager
    claim_id: package-manager-lockfile-parity
    contract_id: package-manager-lockfile-parity
    category: behavior
    command: "cargo test -p jet --lib pkg_manager::lockfile -- --nocapture"
    assertions:
      - "Jet lockfile parsing and hydration preserve package integrity and dependency resolution."
```
