---
id: build-script-version-stamp-contract
summary: External contract for Build Script Version Stamp Contract.
fill_sections: [e2e-test]
---

# EC: Build Script Version Stamp Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: build-script-version-stamp-contract
    capability_id: build-script-version-stamp
    claim_id: build-script-version-stamp-contract
    contract_id: build-script-version-stamp-contract
    category: behavior
    command: "cargo test -p build-stamp"
    assertions:
      - "stamp(prefix) emits git SHA, build timestamp, target triple, and rerun hints through Cargo build-script directives."
      - "fallback behavior remains best-effort when git metadata or TARGET are unavailable."
```
