---
id: dev-server-proxy-contract
summary: External contract for Dev Server Proxy Contract.
fill_sections: [e2e-test]
---

# EC: Dev Server Proxy Contract

---
id: dev-server-proxy-contract
summary: External contract for Dev Server Proxy Contract.
fill_sections: [e2e-test]
---

# EC: Dev Server Proxy Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dev-server-proxy-contract
    capability_id: dev-server-hmr
    claim_id: dev-server-proxy-contract
    contract_id: dev-server-proxy-contract
    category: behavior
    command: "cargo test -p jet --lib dev_server::proxy -- --nocapture"
    assertions:
      - "Configured dev-server proxy routes requests to the upstream target with preserved contract behavior."
```
