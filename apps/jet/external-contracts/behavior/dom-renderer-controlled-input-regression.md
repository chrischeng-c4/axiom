---
id: dom-renderer-controlled-input-regression
summary: External contract for Dom Renderer Controlled Input Regression.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Input Regression

---
id: dom-renderer-controlled-input-regression
summary: External contract for Dom Renderer Controlled Input Regression.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Input Regression

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dom-renderer-controlled-input-regression
    capability_id: browser-trace-parity
    claim_id: dom-renderer-controlled-input-parity
    contract_id: dom-renderer-controlled-input-regression
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture"
    assertions:
      - "The controlled-input parity regression remains covered by the React DOM oracle scenario."
```
