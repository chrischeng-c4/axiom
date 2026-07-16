---
id: webgpu-large-table-smoke
summary: External contract for Webgpu Large Table Smoke.
fill_sections: [e2e-test]
---

# EC: Webgpu Large Table Smoke

---
id: webgpu-large-table-smoke
summary: External contract for Webgpu Large Table Smoke.
fill_sections: [e2e-test]
---

# EC: Webgpu Large Table Smoke

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: webgpu-large-table-smoke
    capability_id: browser-trace-parity
    claim_id: webgpu-large-table-smoke
    contract_id: webgpu-large-table-smoke
    category: behavior
    command: "cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_and_visual_probe_when_available -- --nocapture"
    assertions:
      - "Default WebGPU renderer reports runtime status and captures the required large-table visual probe."
```
