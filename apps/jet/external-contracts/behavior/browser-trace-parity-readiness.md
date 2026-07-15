---
id: browser-trace-parity-readiness
summary: External contract for Browser Trace Parity Readiness.
fill_sections: [e2e-test]
---

# EC: Browser Trace Parity Readiness

---
id: browser-trace-parity-readiness
summary: External contract for Browser Trace Parity Readiness.
fill_sections: [e2e-test]
---

# EC: Browser Trace Parity Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: browser-trace-parity-readiness
    capability_id: browser-trace-parity
    claim_id: browser-trace-parity-readiness
    contract_id: browser-trace-parity-readiness
    category: behavior
    command: "apps/jet/scripts/verify-basic-dom-gates.sh --phase browser"
    assertions:
      - "The browser verification phase produces the required trace and parity evidence."
```
