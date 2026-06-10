---
id: meter-capture-vitals-contract
summary: Single-knob meter.toml (level + [gate]) measurement contract, L1 vitals in capture (getrusage cpu/RSS + wall), until-exit sampling window with opaque --drive composition seam, collapsed artifact output, and removal of dead stress/load-test residue.
fill_sections: [logic, config, cli, unit-test]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: capture-vitals-and-measurement-contract
    claim: capture-vitals-and-measurement-contract
    coverage: full
    rationale: "Closes the README known-limit 'Memory/RSS are not public gates yet' via L1 vitals findings plus declarative meter.toml [gate] adjudication."
  - id: legacy-carried-internals
    role: contributes
    gap: stress-residue-prune
    claim: stress-residue-prune
    coverage: full
    rationale: "Removes dead load-test residue (StressMetrics, TestType::Stress, reporter RPS table, orphaned fuzz_http) so the codebase stops advertising a capability meter must not have."
---

# TD: meter capture vitals + single-knob measurement contract
