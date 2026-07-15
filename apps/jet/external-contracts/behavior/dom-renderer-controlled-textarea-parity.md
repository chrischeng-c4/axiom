---
id: dom-renderer-controlled-textarea-parity
summary: External contract for Dom Renderer Controlled Textarea Parity.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Textarea Parity

---
id: dom-renderer-controlled-textarea-parity
summary: External contract for Dom Renderer Controlled Textarea Parity.
fill_sections: [e2e-test]
---

# EC: Dom Renderer Controlled Textarea Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dom-renderer-controlled-textarea-parity
    capability_id: browser-trace-parity
    claim_id: dom-renderer-controlled-textarea-parity
    contract_id: dom-renderer-controlled-textarea-parity
    category: behavior
    command: "cargo test -p jet --test react_dom_oracle_conformance dom_renderer_controlled_textarea_parity -- --nocapture"
    assertions:
      - "Controlled textarea values remain equal between React DOM and Jet WASM after updates."
```
