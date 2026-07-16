---
id: prebundle-importmap-parity
summary: External contract for Prebundle Importmap Parity.
fill_sections: [e2e-test]
---

# EC: Prebundle Importmap Parity

---
id: prebundle-importmap-parity
summary: External contract for Prebundle Importmap Parity.
fill_sections: [e2e-test]
---

# EC: Prebundle Importmap Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: prebundle-importmap-parity
    capability_id: dev-server-hmr
    claim_id: prebundle-importmap-parity
    contract_id: prebundle-importmap-parity
    category: behavior
    command: "cargo test -p jet --lib dev_server::prebundle -- --nocapture"
    assertions:
      - "Dev-server prebundling produces an import map that resolves configured dependencies."
```
