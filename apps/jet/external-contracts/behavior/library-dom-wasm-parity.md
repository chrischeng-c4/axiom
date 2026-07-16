---
id: library-dom-wasm-parity
summary: External contract for Library Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Library Dom Wasm Parity

---
id: library-dom-wasm-parity
summary: External contract for Library Dom Wasm Parity.
fill_sections: [e2e-test]
---

# EC: Library Dom Wasm Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: library-dom-wasm-parity
    capability_id: browser-trace-parity
    claim_id: library-dom-wasm-parity-fixtures
    contract_id: library-dom-wasm-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance library_dom_wasm_parity -- --nocapture"
    assertions:
      - "Each fixture renders in React DOM and Jet WASM DOM."
      - "Initial visible form state matches for executable form-control fixtures."
      - "Post-input or post-click visible state matches when a fixture declares an interaction."
      - "Failures identify library id, fixture id, phase, expected state, and actual state."
      - "The test uses Jet browser observation helpers and no Python test server."
```
