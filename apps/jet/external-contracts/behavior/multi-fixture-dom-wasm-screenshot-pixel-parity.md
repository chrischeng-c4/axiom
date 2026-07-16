---
id: multi-fixture-dom-wasm-screenshot-pixel-parity
summary: External contract for Multi Fixture Dom Wasm Screenshot Pixel Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Screenshot Pixel Parity

---
id: multi-fixture-dom-wasm-screenshot-pixel-parity
summary: External contract for Multi Fixture Dom Wasm Screenshot Pixel Parity.
fill_sections: [e2e-test]
---

# EC: Multi Fixture Dom Wasm Screenshot Pixel Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-fixture-dom-wasm-screenshot-pixel-parity
    capability_id: browser-trace-parity
    claim_id: parity-corpus-gates
    contract_id: multi-fixture-dom-wasm-screenshot-pixel-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance multi_fixture_dom_wasm_screenshot_pixel_parity -- --nocapture"
    assertions:
      - "React DOM and Jet WASM screenshots decode as PNG."
      - "Viewport dimensions match."
      - "Foreground bounds match within bounds_css_px."
      - "Foreground pixel counts match within foreground_count_ratio."
```
