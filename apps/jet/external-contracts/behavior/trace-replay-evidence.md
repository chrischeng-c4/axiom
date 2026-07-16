---
id: trace-replay-evidence
summary: External contract for Trace Replay Evidence.
fill_sections: [e2e-test]
---

# EC: Trace Replay Evidence

---
id: trace-replay-evidence
summary: External contract for Trace Replay Evidence.
fill_sections: [e2e-test]
---

# EC: Trace Replay Evidence

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: trace-replay-evidence
    capability_id: native-test-product-flow-e2e
    claim_id: trace-replay-evidence
    contract_id: trace-replay-evidence
    category: behavior
    command: "cargo test -p jet --lib trace -- --nocapture"
    assertions:
      - "Recorded Jet trace evidence replays into the expected observable outcome."
```
