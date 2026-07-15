---
id: antd-visual-table-dom-wasm-parity
summary: External contract for Antd Visual Table Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Antd Visual Table Dom Wasm Parity

---
id: antd-visual-table-dom-wasm-parity
summary: External contract for Antd Visual Table Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Antd Visual Table Dom Wasm Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: antd-visual-table-dom-wasm-parity
    capability_id: browser-trace-parity
    claim_id: antd-visual-table-dom-wasm-parity
    contract_id: antd-visual-table-dom-wasm-parity
    category: behavior
    command: "cargo test -p jet --test mui_visual_regression antd_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture"
    assertions:
      - "Ant Design visual-table rendering stays equivalent between React DOM and Jet WASM."
```
