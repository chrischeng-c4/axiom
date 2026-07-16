---
id: multi-fixture-dom-wasm-canvas-paint-parity
summary: External contract for Multi Fixture Dom Wasm Canvas Paint Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Canvas Paint Parity

---
id: multi-fixture-dom-wasm-canvas-paint-parity
summary: External contract for Multi Fixture Dom Wasm Canvas Paint Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Canvas Paint Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-fixture-dom-wasm-canvas-paint-parity
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: multi-fixture-dom-wasm-canvas-paint-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_canvas_paint_parity -- --nocapture"
    assertions:
      - "initial paintOps-derived method sequence is observed in browser canvas calls"
      - "after-click paintOps-derived method sequence is observed in browser canvas calls for interactive fixtures"
      - "mismatch output is machine-readable JSON"
```
