---
id: library-wasm-lowering-fixtures
summary: External contract for Library Wasm Lowering Fixtures.
fill_sections: [e2e-test]
---

# EC: Library Wasm Lowering Fixtures

---
id: library-wasm-lowering-fixtures
summary: External contract for Library Wasm Lowering Fixtures.
fill_sections: [e2e-test]
---

# EC: Library Wasm Lowering Fixtures

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: library-wasm-lowering-fixtures
    capability_id: browser-trace-parity
    claim_id: library-wasm-lowering-fixtures
    contract_id: library-wasm-lowering-fixtures
    category: behavior
    command: "cargo test -p jet --test tsx_to_rust_imports -- --nocapture"
    assertions:
      - "TSX-to-Rust lowering preserves supported import forms in library fixtures."
```
