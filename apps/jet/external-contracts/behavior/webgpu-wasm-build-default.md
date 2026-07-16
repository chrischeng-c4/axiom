---
id: webgpu-wasm-build-default
summary: External contract for Webgpu Wasm Build Default.
fill_sections: [e2e-test]
---

# EC: Webgpu Wasm Build Default

---
id: webgpu-wasm-build-default
summary: External contract for Webgpu Wasm Build Default.
fill_sections: [e2e-test]
---

# EC: Webgpu Wasm Build Default

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: webgpu-wasm-build-default
    capability_id: browser-trace-parity
    claim_id: webgpu-wasm-build-default
    contract_id: webgpu-wasm-build-default
    category: behavior
    command: "cargo test -p jet --test wasm_build_end_to_end wasm_build_selects_webgpu_scaffold_by_default -- --nocapture"
    assertions:
      - "Jet WASM build selects the WebGPU scaffold by default."
```
