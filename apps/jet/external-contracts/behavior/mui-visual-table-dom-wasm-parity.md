---
id: mui-visual-table-dom-wasm-parity
summary: External contract for Mui Visual Table Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Mui Visual Table Dom Wasm Parity

---
id: mui-visual-table-dom-wasm-parity
summary: External contract for Mui Visual Table Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Mui Visual Table Dom Wasm Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: mui-visual-table-dom-wasm-parity
    capability_id: browser-trace-parity
    claim_id: mui-visual-table-dom-wasm-parity
    contract_id: mui-visual-table-dom-wasm-parity
    category: behavior
    command: "cargo test -p jet --test mui_visual_regression mui_visual_fixture_renders_on_react_dom_and_jet_wasm -- --nocapture"
    assertions:
      - "Material UI visual-table rendering stays equivalent between React DOM and Jet WASM."
```
