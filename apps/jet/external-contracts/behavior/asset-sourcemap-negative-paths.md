---
id: asset-sourcemap-negative-paths
summary: External contract for Asset Sourcemap Negative Paths.
fill_sections: [e2e-test]
---

# EC: Asset Sourcemap Negative Paths

---
id: asset-sourcemap-negative-paths
summary: External contract for Asset Sourcemap Negative Paths.
fill_sections: [e2e-test]
---

# EC: Asset Sourcemap Negative Paths

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: asset-sourcemap-negative-paths
    capability_id: bundler-production-build
    claim_id: asset-sourcemap-negative-paths
    contract_id: asset-sourcemap-negative-paths
    category: behavior
    command: "cargo test -p jet --lib asset -- --nocapture"
    assertions:
      - "Invalid asset source-map inputs report a safe failure without producing malformed output."
```
