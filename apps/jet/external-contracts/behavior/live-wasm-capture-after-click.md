---
id: live-wasm-capture-after-click
summary: External contract for Live Wasm Capture After Click.
fill_sections: [e2e-test]
---

# EC: Live Wasm Capture After Click

---
id: live-wasm-capture-after-click
summary: External contract for Live Wasm Capture After Click.
fill_sections: [e2e-test]
---

# EC: Live Wasm Capture After Click

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: live-wasm-capture-after-click
    capability_id: browser-trace-parity
    claim_id: browser-automation-diagnostics
    contract_id: live-wasm-capture-after-click
    category: behavior
    command: "cargo test -p jet --test browser_cli_smoke -- --nocapture"
    assertions:
      - "bundle.schema_version == jet.browser.observation.v1"
      - "bundle.layout_tree is non-empty"
      - "bundle.paint_ops is non-empty"
      - "bundle.hook_values includes the post-click counter value"
      - "no python static server is used"
```
