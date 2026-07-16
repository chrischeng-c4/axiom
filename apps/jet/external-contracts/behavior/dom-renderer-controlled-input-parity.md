---
id: dom-renderer-controlled-input-parity
summary: External contract for Dom Renderer Controlled Input Parity.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Input Parity

---
id: dom-renderer-controlled-input-parity
summary: External contract for Dom Renderer Controlled Input Parity.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Input Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dom-renderer-controlled-input-parity
    capability_id: browser-trace-parity
    claim_id: dom-renderer-controlled-input-parity
    contract_id: dom-renderer-controlled-input-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_input_parity -- --nocapture"
    assertions:
      - "React DOM and Jet WASM DOM renderer initial host trees match."
      - "React DOM and Jet WASM DOM renderer initial input states match."
      - "After replacing text, input values match."
      - "After replacing text, derived label text matches."
```
