---
id: counter-dom-wasm-parity-after-click
summary: External contract for Counter Dom Wasm Parity After Click.
fill_sections: [e2e-test]
---

# EC: Counter Dom Wasm Parity After Click

---
id: counter-dom-wasm-parity-after-click
summary: External contract for Counter Dom Wasm Parity After Click.
fill_sections: [e2e-test]
---

# EC: Counter Dom Wasm Parity After Click

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: counter-dom-wasm-parity-after-click
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: counter-dom-wasm-parity-after-click
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance counter_demo_matches_react_dom_oracle_initial_and_after_click -- --nocapture"
    assertions:
      - "initial DOM tree equals normalized Jet WASM element tree"
      - "post-click DOM tree equals normalized Jet WASM element tree"
      - "WASM observation includes hook value 1 after click"
      - "mismatch output includes concrete DOM and WASM JSON"
```
